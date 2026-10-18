//! Goal-set authoring followed by stable artifact collection.
use crate::{
    agent::Codex,
    analysis::Analyzer,
    cli, collection, fingerprint,
    model::*,
    plan, process,
    scheduler::{self, Limits},
    store::Store,
};
use anyhow::{Result, bail};
use serde_json::json;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Workflow {
    Autonomous,
    Planned,
}
fn collect(store: &Store, engine: &mut Analyzer, start: Instant, limits: &Limits) -> Result<State> {
    let s = cli::sync(store, engine)?;
    if matches!(collection::status(store, &s), "stale" | "invalid") {
        store.event(
            "review_rejected",
            json!({"reason":"staged review needs explicit restaging","input":s.input}),
        )?;
        return Ok(s);
    }
    let remaining = limits.max_seconds.saturating_sub(start.elapsed().as_secs());
    if remaining == 0 {
        return Ok(s);
    }
    collection::collect(store, engine, remaining.min(limits.verify_seconds))
}
pub fn run(store: &Store, engine: &mut Analyzer, retry: bool, limits: &Limits) -> Result<i32> {
    let start = Instant::now();
    if limits.max_seconds == 0
        || limits.max_tasks == 0
        || limits.verify_seconds == 0
        || limits.max_attempts == 0
        || limits.max_no_progress == 0
    {
        bail!("run limits must be positive");
    }
    let _run = store.lock("run")?;
    let config = store.config()?;
    if config.verification.tool != Tool::Aeneas {
        bail!(
            "autonomous artifact collection currently supports Aeneas; use --workflow planned for this backend"
        );
    }
    let mut s = cli::sync(store, engine)?;
    fingerprint::tool_version(s.tool)?;
    if !fingerprint::version("codex", &["--version"])?.contains("0.159.") {
        bail!("app-server adapter requires Codex 0.159.x");
    }
    store.transaction(None, |s| {
        if retry {
            s.run.attempts.clear();
            for t in s.targets.values_mut() {
                t.blocker = None;
            }
        }
        if let Some(active) = s.run.active.take() {
            if let Some(a) = s.run.attempts.get_mut(&active.task.key()) {
                a.failed = false;
                a.total = a.total.saturating_sub(1);
            }
            s.run.last_result = "recovered authoring; saved artifacts preserved".into();
        }
        Ok(())
    })?;
    let authoring = scheduler::authoring_seconds(limits, config.verification.required);
    store.event("run_budget",json!({"workflow":"autonomous","max_seconds":limits.max_seconds,"authoring_seconds":authoring,"finalization_reserved_seconds":limits.max_seconds-authoring}))?;
    s = collect(store, engine, start, limits)?;
    let mut codex = None;
    for _ in 0..limits.max_tasks {
        if process::interrupted() {
            return Ok(130);
        }
        if s.complete(config.verification.required) || start.elapsed().as_secs() >= authoring {
            break;
        }
        let task = Task {
            kind: TaskKind::Author,
            work: None,
            targets: plan::roots(&s),
        };
        if task.targets.is_empty() || !scheduler::eligible(&s, &task, limits) {
            break;
        }
        let before = s.clone();
        let input = scheduler::task_input(&s, &task);
        let key = task.key();
        store.transaction(None, |s| {
            s.run.active = Some(Active {
                task: task.clone(),
                input: input.clone(),
                started: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            });
            let a = s.run.attempts.entry(key.clone()).or_default();
            a.checked_work
                .extend(scheduler::known_checked_work(&before));
            a.total += 1;
            a.failed = true;
            a.last_input = input.clone();
            a.last_result = "authoring, not collected".into();
            Ok(())
        })?;
        store.event(
            "task_started",
            json!({"task":task,"input":s.input,"workflow":"autonomous"}),
        )?;
        let result = (|| -> Result<String> {
            let _worker = store.lock("worker")?;
            if start.elapsed().as_secs() >= authoring {
                return Err(crate::agent::WorkerDeadline.into());
            }
            if codex.is_none() {
                codex = Some(Codex::connect_autonomous(
                    store,
                    &s,
                    config.verification.required,
                )?);
                let c = codex.as_ref().unwrap();
                store.transaction(None, |s| {
                    s.run.start_thread(&c.thread);
                    Ok(())
                })?;
            }
            let remaining = authoring.saturating_sub(start.elapsed().as_secs());
            if remaining == 0 {
                return Err(crate::agent::WorkerDeadline.into());
            }
            store.event("worker_started", json!({"task":task,"input":s.input}))?;
            codex
                .as_mut()
                .unwrap()
                .run(&s, &task, Duration::from_secs(remaining), remaining)
        })();
        let deadline = result
            .as_ref()
            .err()
            .is_some_and(|e| e.is::<crate::agent::WorkerDeadline>());
        store.event(
            "worker_finished",
            json!({"task":task,"completed":result.is_ok(),"worker_deadline":deadline}),
        )?;
        store.transaction(None, |s| {
            if let Some(c) = &codex {
                s.run.record_usage(&c.thread, c.usage);
            }
            Ok(())
        })?;
        if process::interrupted() {
            return Ok(130);
        }
        if deadline {
            drop(codex.take());
        }
        if let Err(error) = result {
            if !deadline {
                store.transaction(None,|s|{s.run.active=None;s.run.last_result="author stopped on environment, approval, or connection failure; files preserved".into();Ok(())})?;
                return Err(error);
            }
        }
        s = collect(store, engine, start, limits)?;
        let progress = scheduler::advanced(&before, &s);
        let done = s.complete(config.verification.required);
        store.transaction(None, |state| {
            state.run.active = None;
            let a = state.run.attempts.entry(key.clone()).or_default();
            scheduler::record_checked_progress(a, scheduler::checked_work(&s, &task));
            a.failed = !done && !progress && !deadline;
            if deadline {
                a.total = a.total.saturating_sub(1);
            }
            a.no_progress = if progress {
                0
            } else if deadline {
                a.no_progress
            } else {
                a.no_progress + 1
            };
            a.last_input = scheduler::task_input(&s, &task);
            a.last_result = if done {
                "user goal set collected"
            } else if deadline {
                "authoring budget ended; stable artifacts collected"
            } else {
                "goal incomplete; collected available evidence"
            }
            .into();
            state.run.last_result = a.last_result.clone();
            state.run.elapsed_ms = start.elapsed().as_millis() as u64;
            Ok(())
        })?;
        store.event("task_finished",json!({"task":task,"success":done,"forward_progress":progress,"worker_deadline":deadline,"input":s.input}))?;
        s = store.load()?;
        if deadline || done {
            break;
        }
    }
    if process::interrupted() {
        return Ok(130);
    }
    cli::display(&s, &config, false, false, false)?;
    Ok(if s.complete(config.verification.required) {
        0
    } else {
        1
    })
}
