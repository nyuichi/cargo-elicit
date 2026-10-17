use crate::{
    agent::Codex,
    analysis::{self, Analyzer, Selection},
    fingerprint,
    model::*,
    process,
    scheduler::{self, Limits},
    store::{self, Store},
    verifier,
};
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use serde_json::json;
use std::{
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[derive(Parser)]
#[command(
    name = "cargo elicit",
    version,
    about = "Persistent Rust specification and verification with a single Codex worker"
)]
struct Cli {
    #[arg(long, global = true)]
    manifest_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Create configuration and reconcile definitions in one package/build configuration.
    Init {
        #[arg(long, value_enum)]
        tool: Option<Tool>,
        #[command(flatten)]
        scope: ScopeArgs,
        /// Verify only these API roots; accepts exact IDs, aliases, or unique names.
        #[arg(long = "api", conflicts_with = "whole_package")]
        apis: Vec<String>,
        /// Explicitly replace selected roots with the whole-package goal.
        #[arg(long)]
        whole_package: bool,
    },
    /// Resume work, bounded by task/time/attempt limits. Exit: 0 complete, 1 unresolved, 2 environment, 130 interrupted.
    Run {
        #[arg(long)]
        retry: bool,
        #[command(flatten)]
        limits: Limits,
    },
    Status {
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        public: bool,
    },
    /// Revision-checked tools for the specification worker.
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Human review: accept the current runtime implementation as a new baseline.
    AcceptImplementation {
        id: String,
        #[arg(long)]
        reason: String,
    },
}
#[derive(Args, Default)]
struct ScopeArgs {
    #[arg(short, long)]
    package: Option<String>,
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
    #[arg(long)]
    all_features: bool,
    #[arg(long)]
    no_default_features: bool,
    #[arg(long)]
    target: Option<String>,
}
#[derive(Subcommand)]
enum AgentCommand {
    /// Install a JSON proof plan {"steps": [...]} without accepting proof evidence.
    Plan {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        revision: u64,
    },
    Show {
        id: Option<String>,
    },
    Submit {
        id: String,
        #[arg(long)]
        revision: u64,
        #[arg(long)]
        note: String,
        #[arg(long = "depends")]
        depends: Vec<String>,
        #[arg(long = "unit")]
        units: Vec<String>,
    },
    Depend {
        id: String,
        dependency: String,
        #[arg(long)]
        revision: u64,
    },
    Issue {
        id: String,
        #[arg(long)]
        revision: u64,
        #[arg(long)]
        reason: String,
    },
    Verify {
        ids: Vec<String>,
        #[arg(long)]
        revision: u64,
        #[arg(long, default_value_t = 600)]
        seconds: u64,
    },
}
pub fn entry() -> i32 {
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|a| a == "elicit") {
        args.remove(1);
    }
    let cli = Cli::parse_from(args);
    let _ = process::install_interrupt();
    match execute(cli) {
        Ok(code) => {
            if process::interrupted() {
                130
            } else {
                code
            }
        }
        Err(e) => {
            if process::interrupted() {
                130
            } else {
                eprintln!("cargo-elicit: {e:#}");
                2
            }
        }
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn root(cli: &Cli) -> Result<PathBuf> {
    let path = cli
        .manifest_path
        .clone()
        .unwrap_or(std::env::current_dir()?.join("Cargo.toml"));
    Ok(path
        .canonicalize()
        .context("Cargo.toml not found; use --manifest-path")?
        .parent()
        .unwrap()
        .to_path_buf())
}
pub fn sync(store: &Store, engine: &mut Analyzer) -> Result<State> {
    if process::interrupted() {
        bail!("interrupted");
    }
    let _lock = store.state_lock()?;
    let mut s = store.load()?;
    let previous = serde_json::to_vec(&s)?;
    let package = analysis::package(
        PathBuf::from(&s.scope.manifest_path).parent().unwrap(),
        &Selection::from_scope(&s.scope),
    )?;
    let version =
        fingerprint::tool_version(s.tool).unwrap_or_else(|_| "verifier unavailable".into());
    let (input, context_input, input_files) =
        fingerprint::inputs(&package.scope, s.tool, &version)?;
    // The ledger caches definition identities, aliases, call edges and unresolved
    // analysis. Reuse it across CLI invocations only for identical complete inputs.
    if input == s.input && package.scope == s.scope {
        return Ok(s);
    }
    if process::interrupted() {
        bail!("interrupted");
    }
    let (targets, incomplete) = engine.analyze(&package, s.tool)?;
    if process::interrupted() {
        bail!("interrupted");
    }
    s.scope = package.scope;
    let changed: Vec<_> = input_files
        .keys()
        .chain(s.input_files.keys())
        .filter(|p| input_files.get(*p) != s.input_files.get(*p))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .cloned()
        .collect();
    store.event("input_changed", json!({"previous":s.input,"current":input,"previous_manifest_available":!s.input_files.is_empty(),"changed_files":changed.iter().filter(|k|!k.starts_with('@')).collect::<Vec<_>>(),"changed_context_keys":changed.iter().filter(|k|k.starts_with('@')).collect::<Vec<_>>(),"context_changed":s.context_input != context_input}))?;
    s.context_input = context_input;
    s.input_files = input_files;
    s.reconcile(targets, incomplete, input);
    if previous != serde_json::to_vec(&s)? {
        store.save(&mut s)?;
    }
    Ok(s)
}
fn execute(cli: Cli) -> Result<i32> {
    let root = root(&cli)?;
    let mut engine = Analyzer::default();
    if let Commands::Init {
        tool,
        scope,
        apis,
        whole_package,
    } = &cli.command
    {
        let store = Store { root };
        let _run = store.lock("run")?;
        let _state = store.state_lock()?;
        let config_path = store.root.join("elicit.toml");
        if !config_path.exists() {
            let tool =
                tool.context("--tool creusot or --tool kani is required for first initialization")?;
            let c = Config::new(tool);
            store::atomic_write(&config_path, toml::to_string_pretty(&c)?.as_bytes())?;
        }
        let config = store.config()?;
        if tool.is_some_and(|t| t != config.verification.tool) {
            bail!("verifier change is unsupported; config and state preserved");
        }
        let selection = Selection {
            package: scope.package.clone(),
            features: scope.features.clone(),
            all_features: scope.all_features,
            no_default_features: scope.no_default_features,
            target: scope.target.clone(),
        };
        let mut s = if store.root.join(".elicit/state.toml").exists() {
            let s = store.load()?;
            if selection.package.is_some() && selection.package.as_ref() != Some(&s.scope.package)
                || !selection.features.is_empty() && selection.features != s.scope.features
                || selection.target.is_some() && selection.target != s.scope.requested_target
                || selection.all_features && !s.scope.all_features
                || selection.no_default_features && !s.scope.no_default_features
            {
                bail!(
                    "changing persisted build scope requires a separate working directory; state preserved"
                );
            }
            s
        } else {
            State::new(
                config.verification.tool,
                analysis::package(&store.root, &selection)?.scope,
            )
        };
        let package = analysis::package(
            PathBuf::from(&s.scope.manifest_path).parent().unwrap(),
            &Selection::from_scope(&s.scope),
        )?;
        let (targets, incomplete) = engine.analyze(&package, s.tool)?;
        s.scope = package.scope;
        let v = fingerprint::tool_version(s.tool).unwrap_or_else(|_| "verifier unavailable".into());
        let (input, context_input, input_files) = fingerprint::inputs(&s.scope, s.tool, &v)?;
        s.context_input = context_input;
        s.input_files = input_files;
        s.reconcile(targets, incomplete, input);
        if !apis.is_empty() || *whole_package {
            let goal = if *whole_package {
                crate::plan::VerificationGoal::WholePackage
            } else {
                crate::plan::VerificationGoal::SelectedApis(crate::plan::resolve(&s, apis)?)
            };
            if goal != s.goal {
                s.goal = goal;
                s.proof_plan = Default::default();
                s.run.attempts.clear();
            }
        }
        store.save(&mut s)?;
        println!(
            "Initialized {} ({}, features {:?}, target {}). {} definitions. Existing source and results preserved.",
            s.scope.package,
            s.tool,
            s.scope.resolved_features,
            s.scope.target,
            s.targets.len()
        );
        return Ok(0);
    }
    let store = Store::discover(&root)?;
    match cli.command {
        Commands::Status { json: as_json } => {
            let s = sync(&store, &mut engine)?;
            display(&s, &store.config()?, as_json, false, false)?;
            Ok(0)
        }
        Commands::List {
            json: as_json,
            public,
        } => {
            let s = sync(&store, &mut engine)?;
            display(&s, &store.config()?, as_json, true, public)?;
            Ok(0)
        }
        Commands::Run { retry, limits } => run(&store, &mut engine, retry, &limits),
        Commands::Agent { command } => agent(&store, &mut engine, command),
        Commands::AcceptImplementation { id, reason } => {
            let _worker = store.lock("worker")?;
            let _run = store.lock("run")?;
            let s = sync(&store, &mut engine)?;
            store.transaction(Some(s.revision), |s| {
                let t = s.targets.get_mut(&id).context("unknown target")?;
                t.original_body = t.body_hash.clone();
                t.implementation_changed = false;
                t.stale
                    .push("human accepted implementation; specification review required".into());
                t.notes
                    .push(format!("Human accepted runtime change: {reason}"));
                Ok(())
            })?;
            Ok(0)
        }
        Commands::Init { .. } => unreachable!(),
    }
}
fn display(s: &State, c: &Config, as_json: bool, list: bool, public: bool) -> Result<()> {
    let targets:Vec<_>=s.targets.values().filter(|t|!public || t.visibility==Visibility::Public).map(|t|{let mut v=serde_json::to_value(t).unwrap();if t.kind.proof(){v["verified"]=json!(s.verified(&t.id));}v["unsupported"]=json!(s.unsupported(&t.id));v["verification_results"]=json!(s.results.iter().filter(|r|r.target==t.id).map(|r|json!({"result":r,"stale":r.input!=s.input || !s.dependencies_ready(&t.id)})).collect::<Vec<_>>());v}).collect();
    let active: Vec<_> = s
        .targets
        .values()
        .filter(|t| !t.removed && t.visibility != Visibility::Auxiliary)
        .collect();
    let types: Vec<_> = active.iter().filter(|t| !t.kind.proof()).collect();
    let functions: Vec<_> = active.iter().filter(|t| t.kind.proof()).collect();
    let summary = json!({"revision":s.revision,"scope":s.scope,"tool":s.tool,"required":c.verification.required,"goal":s.goal,"proof_plan":s.proof_plan,"checked_lemmas":s.proof_plan.steps.iter().filter(|p| p.kind==crate::plan::StepKind::Lemma && crate::plan::checked(s,p)).count(),"complete":s.complete(c.verification.required),"types_traits":{"total":types.len(),"specified":types.iter().filter(|t|t.ready()).count()},"functions_methods_impls":{"total":functions.len(),"specified":functions.iter().filter(|t|t.ready()).count(),"verified":functions.iter().filter(|t|s.verified(&t.id)).count()},"blocked":active.iter().filter(|t| t.blocker.is_some() || s.unsupported(&t.id).is_some()).map(|t| json!({"id":t.id,"reason":s.unsupported(&t.id).map(str::to_string).or_else(||t.blocker.as_ref().map(|b|b.reason.clone()))})).collect::<Vec<_>>(),"analysis_incomplete":s.analysis_incomplete,"run":s.run});
    if !as_json {
        println!("Goal: {}", serde_json::to_string(&s.goal)?);
        println!(
            "Proof plan: {} steps, {} current checked lemmas",
            s.proof_plan.steps.len(),
            s.proof_plan
                .steps
                .iter()
                .filter(|p| p.kind == crate::plan::StepKind::Lemma && crate::plan::checked(s, p))
                .count()
        );
    }
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&if list {
                json!({"summary":summary,"targets":targets})
            } else {
                summary
            })?
        );
    } else {
        println!(
            "{}: {} / target {}, features {:?}\nMode: {}. Revision {}. {}",
            s.scope.package,
            s.tool,
            s.scope.target,
            s.scope.resolved_features,
            if c.verification.required {
                "verification required"
            } else {
                "specification"
            },
            s.revision,
            if s.complete(c.verification.required) {
                "complete"
            } else {
                "unresolved"
            }
        );
        println!(
            "Types/traits specified: {}/{}; functions/methods/impls specified: {}/{}, fully verified: {}/{}",
            types.iter().filter(|t| t.ready()).count(),
            types.len(),
            functions.iter().filter(|t| t.ready()).count(),
            functions.len(),
            functions.iter().filter(|t| s.verified(&t.id)).count(),
            functions.len()
        );
        for t in &active {
            if let Some(reason) = s
                .unsupported(&t.id)
                .map(str::to_string)
                .or_else(|| t.blocker.as_ref().map(|b| b.reason.clone()))
            {
                println!("Blocked: {}: {reason}", t.id);
            }
        }
        for issue in &s.analysis_incomplete {
            println!("Analysis incomplete: {issue}");
        }
        if list {
            for v in targets {
                println!("{}", serde_json::to_string_pretty(&v)?);
            }
        }
        if !s.run.last_result.is_empty() {
            println!("Last run: {}", s.run.last_result);
        }
    }
    Ok(())
}
fn agent(store: &Store, engine: &mut Analyzer, command: AgentCommand) -> Result<i32> {
    let s = sync(store, engine)?;
    match command {
        AgentCommand::Plan { file, revision } => {
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Document {
                steps: Vec<crate::plan::ProofStep>,
            }
            let doc: Document = serde_json::from_slice(&std::fs::read(file)?)?;
            let _verification = store.lock("verification")?;
            store.transaction(Some(revision), |s| crate::plan::install(s, doc.steps))?;
            store.event(
                "plan_installed",
                json!({"steps":store.load()?.proof_plan.steps}),
            )?;
            println!("plan accepted; no proof evidence accepted");
        }
        AgentCommand::Show { id } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&if let Some(id) = id {
                    json!({"revision":s.revision,"input":s.input,"target":s.targets.get(&id).context("unknown target")?,"verified":s.verified(&id),"verification_results":s.results.iter().filter(|r| r.target == id).collect::<Vec<_>>(),"unsupported":s.unsupported(&id),"aeneas_contract":if s.tool == Tool::Aeneas && matches!(s.targets[&id].kind, Kind::Function | Kind::Method) {Some(crate::aeneas::contract_name(&id))} else {None}})
                } else {
                    serde_json::to_value(&s)?
                })?
            );
        }
        AgentCommand::Submit {
            id,
            revision,
            note,
            depends,
            units,
        } => {
            if revision != s.revision {
                bail!("revision conflict: current {}", s.revision);
            }
            if !s.targets.contains_key(&id) {
                bail!("unknown target");
            }
            if s.tool == Tool::Aeneas {
                crate::aeneas::check_submission(&s.scope, &s.targets[&id])?;
            }
            let input = verifier::syntax(store, &s, 600)?;
            store.transaction(Some(revision), |s| {
                if input != s.input {
                    bail!("input changed; reconcile and resubmit");
                }
                let mut dependencies = std::collections::BTreeMap::new();
                // Retain known dependencies unless explicitly superseded by a reasoned source review.
                for d in depends {
                    let t = s.targets.get(&d).context("unknown dependency")?;
                    if t.removed {
                        bail!("dependency was removed");
                    }
                    dependencies.insert(d, t.spec_hash.clone());
                }
                let hashes = s
                    .targets
                    .iter()
                    .filter(|(_, t)| !t.removed)
                    .map(|(id, t)| (id.clone(), t.spec_hash.clone()))
                    .collect::<std::collections::BTreeMap<_, _>>();
                let t = s.targets.get_mut(&id).unwrap();
                if t.removed {
                    bail!("cannot submit removed target");
                }
                if t.implementation_changed {
                    bail!("runtime implementation changed; human review required");
                }
                for (d, h) in &mut t.dependencies {
                    *h = hashes
                        .get(d)
                        .context("a known specification dependency was removed; report the issue")?
                        .clone();
                }
                t.dependencies.extend(dependencies);
                if !units.is_empty() {
                    if t.kind.proof() && !units.iter().any(|u| u == "contract") {
                        bail!("required contract unit cannot be removed");
                    }
                    t.required_units = units;
                }
                t.bind_models();
                t.specified = true;
                t.blocker = None;
                t.stale.clear();
                t.notes.push(note);
                let changed_id = t.id.clone();
                let changed_hash = t.spec_hash.clone();
                for target in s.targets.values_mut() {
                    if target
                        .dependencies
                        .get(&changed_id)
                        .is_some_and(|h| h != &changed_hash)
                    {
                        target
                            .stale
                            .push(format!("specification dependency changed: {changed_id}"));
                        target.stale.sort();
                        target.stale.dedup();
                    }
                }
                s.processing_input = input;
                Ok(())
            })?;
            println!("accepted; fetch current revision before the next submission");
        }
        AgentCommand::Depend {
            id,
            dependency,
            revision,
        } => {
            store.transaction(Some(revision), |s| {
                let h = s
                    .targets
                    .get(&dependency)
                    .context("unknown dependency")?
                    .spec_hash
                    .clone();
                let t = s.targets.get_mut(&id).context("unknown target")?;
                t.dependencies.insert(dependency, h);
                t.blocker = None;
                t.stale
                    .push("new specification dependency requires review".into());
                Ok(())
            })?;
        }
        AgentCommand::Issue {
            id,
            revision,
            reason,
        } => {
            store.transaction(Some(revision), |s| {
                if !s.targets.contains_key(&id) {
                    bail!("unknown target");
                }
                let input = s.target_input(&id);
                let t = s.targets.get_mut(&id).unwrap();
                t.blocker = Some(Blocker {
                    reason: reason.clone(),
                    input,
                });
                if t.notes.last() != Some(&reason) {
                    t.notes.push(reason);
                }
                Ok(())
            })?;
        }
        AgentCommand::Verify {
            ids,
            revision,
            seconds,
        } => {
            let ids = if ids.is_empty() {
                s.targets
                    .values()
                    .filter(|t| t.kind.proof() && !t.removed)
                    .map(|t| t.id.clone())
                    .collect()
            } else {
                ids
            };
            let r = verifier::verify(store, &ids, Some(revision), seconds)?;
            println!("{}", serde_json::to_string_pretty(&r)?);
            return Ok(if process::interrupted() {
                130
            } else if r.iter().all(|r| r.outcome == Outcome::Success) {
                0
            } else {
                1
            });
        }
    }
    Ok(0)
}
fn run(store: &Store, engine: &mut Analyzer, retry: bool, limits: &Limits) -> Result<i32> {
    if limits.max_tasks == 0
        || limits.max_seconds == 0
        || limits.task_seconds == 0
        || limits.plan_seconds == 0
        || limits.verify_seconds == 0
        || limits.max_attempts == 0
        || limits.max_no_progress == 0
    {
        bail!("run limits must be positive");
    }
    let _run = store.lock("run")?;
    let config = store.config()?;
    let mut s = sync(store, engine)?;
    fingerprint::tool_version(s.tool)?;
    let cv = fingerprint::version("codex", &["--version"])?;
    if !cv.contains("0.159.") {
        bail!("app-server adapter requires Codex 0.159.x; found {cv}");
    }
    store.transaction(None, |s| {
        if retry {
            s.run.attempts.clear();
            for t in s.targets.values_mut() {
                t.blocker = None;
            }
        }
        if s.run.active.take().is_some() {
            s.run.last_result =
                "recovered interrupted task; source and partial artifacts preserved".into();
        }
        Ok(())
    })?;
    let start = Instant::now();
    let mut codex: Option<Codex> = None;
    for _ in 0..limits.max_tasks {
        if process::interrupted() || start.elapsed().as_secs() >= limits.max_seconds {
            break;
        }
        s = sync(store, engine)?;
        if s.processing_input != s.input
            && s.targets
                .values()
                .filter(|t| !t.removed && t.visibility != Visibility::Auxiliary)
                .all(Target::ready)
        {
            let input = verifier::syntax(
                store,
                &s,
                limits
                    .verify_seconds
                    .min(limits.max_seconds.saturating_sub(start.elapsed().as_secs())),
            )?;
            store.transaction(Some(s.revision), |s| {
                s.processing_input = input;
                Ok(())
            })?;
            s = store.load()?;
        }
        if s.complete(config.verification.required) {
            break;
        }
        // Persist prepared proof work while it exists, before unrelated stale
        // specifications can consume the remaining budget. This does not ask an
        // AI to solve any API ahead of another; it only checks saved artifacts.
        if config.verification.required && s.tool == Tool::Aeneas {
            let ids = crate::aeneas::prepared_targets(&s)?;
            if !ids.is_empty() || crate::plan::checkpoint_needed(&s) {
                let remaining = limits.max_seconds.saturating_sub(start.elapsed().as_secs());
                if remaining == 0 {
                    break;
                }
                store.event(
                    "artifact_checkpoint",
                    json!({"targets":ids,"input":s.input}),
                )?;
                let results = verifier::verify(
                    store,
                    &ids,
                    Some(s.revision),
                    limits.verify_seconds.min(remaining),
                )?;
                if process::interrupted() {
                    return Ok(130);
                }
                s = sync(store, engine)?;
                if results
                    .iter()
                    .any(|r| r.outcome == Outcome::EnvironmentError)
                {
                    bail!(
                        "verification environment failed during artifact checkpoint; inspect the recorded phase/cache"
                    );
                }
            }
        }
        if start.elapsed().as_secs() >= limits.max_seconds {
            break;
        }
        let Some(task) = scheduler::next(&s, config.verification.required, limits) else {
            break;
        };
        let before = s.clone();
        let input = scheduler::task_input(&s, &task);
        let key = task.key();
        store.transaction(None, |s| {
            s.run.active = Some(Active {
                task: task.clone(),
                input: input.clone(),
                started: now(),
            });
            let a = s.run.attempts.entry(key.clone()).or_default();
            a.total += 1;
            a.last_input = input.clone();
            a.failed = true;
            a.last_result = "started, not finalized".into();
            Ok(())
        })?;
        s = store.load()?;
        store.event(
            "task_started",
            json!({"task":task,"input":s.input,"task_input":input,"revision":s.revision}),
        )?;
        // A fresh input with existing proof artifacts needs a checker, not an AI
        // turn. Failed proofs still enter the worker for actual repair.
        let refresh = task.kind == TaskKind::Verify
            && s.tool == Tool::Aeneas
            && store
                .root
                .join("elicit-proofs/aeneas/Proofs.lean")
                .is_file()
            && task.targets.iter().all(|id| {
                !s.results
                    .iter()
                    .any(|r| &r.target == id && r.input == s.input)
            });
        let mut refreshed = false;
        if refresh {
            let remaining = limits.max_seconds.saturating_sub(start.elapsed().as_secs());
            let results = verifier::verify(
                store,
                &task.targets,
                Some(s.revision),
                limits.verify_seconds.min(remaining),
            )?;
            if results
                .iter()
                .any(|r| r.outcome == Outcome::EnvironmentError)
            {
                bail!(
                    "verification environment failed before worker repair; inspect the recorded phase/cache"
                );
            }
            refreshed = task.targets.iter().all(|id| {
                results
                    .iter()
                    .any(|r| &r.target == id && r.outcome == Outcome::Success)
            });
            if process::interrupted() {
                return Ok(130);
            }
            s = sync(store, engine)?;
        }
        if process::interrupted() {
            return Ok(130);
        }
        if start.elapsed().as_secs() >= limits.max_seconds {
            store.event("budget_exhausted", json!({"task":task,"input":s.input}))?;
            break;
        }
        let result = (|| -> Result<String> {
            if refreshed {
                return Ok("checked existing proof artifacts without an AI turn".into());
            }
            let _worker = store.lock("worker")?;
            if codex.is_none() {
                codex = Some(Codex::connect(store, &s, config.verification.required)?);
                let c = codex.as_ref().unwrap();
                store.transaction(None, |s| {
                    s.run.thread = Some(c.thread.clone());
                    Ok(())
                })?;
                s = store.load()?;
            }
            let remaining = limits.max_seconds.saturating_sub(start.elapsed().as_secs());
            store.event("worker_started", json!({"task":task,"input":s.input}))?;
            codex.as_mut().unwrap().run(
                &s,
                &task,
                Duration::from_secs(limits.task_seconds.min(remaining).min(
                    if task.kind == TaskKind::Plan {
                        limits.plan_seconds
                    } else {
                        u64::MAX
                    },
                )),
                remaining,
            )
        })();
        store.event(
            "worker_finished",
            json!({"task":task,"called":!refreshed,"completed":result.is_ok()}),
        )?;
        let worker_deadline = result
            .as_ref()
            .is_err_and(|e| e.is::<crate::agent::WorkerDeadline>());
        // A failed/interrupted worker is killed before releasing work to a verifier.
        if result.is_err() {
            codex = None;
        }
        if process::interrupted() {
            store.transaction(None, |s| {
                s.run.last_result =
                    "interrupted; task checkpoint retained for source reconciliation on resume"
                        .into();
                Ok(())
            })?;
            return Ok(130);
        }
        let mut after = sync(store, engine)?;
        if (result.is_ok() || worker_deadline)
            && s.tool == Tool::Aeneas
            && task.kind != TaskKind::Plan
            && !refreshed
            && (task.work.is_some() || task.kind == TaskKind::Integrate)
            && !process::interrupted()
        {
            let remaining = limits.max_seconds.saturating_sub(start.elapsed().as_secs());
            if remaining > 0 {
                // Lemmas are kernel checkpoints, never API acceptance. This call
                // also collects every independently ready API proof in the cache.
                verifier::verify(
                    store,
                    &[],
                    Some(after.revision),
                    limits.verify_seconds.min(remaining),
                )?;
                after = sync(store, engine)?;
            }
        }
        let mut proof_ok = refreshed;
        let mut proof_environment_error = false;
        if result.is_ok()
            && !refreshed
            && task.kind == TaskKind::Verify
            && task.work.is_none()
            && !process::interrupted()
        {
            let remaining = limits.max_seconds.saturating_sub(start.elapsed().as_secs());
            if remaining > 0 {
                let results = verifier::verify(
                    store,
                    &task.targets,
                    Some(after.revision),
                    limits.verify_seconds.min(remaining),
                );
                match results {
                    Ok(r) => {
                        proof_ok = r.iter().all(|r| r.outcome == Outcome::Success);
                        proof_environment_error =
                            r.iter().any(|r| r.outcome == Outcome::EnvironmentError);
                    }
                    Err(_) => proof_environment_error = true,
                }
                if process::interrupted() {
                    return Ok(130);
                }
                after = sync(store, engine)?;
            }
        }
        let progress = scheduler::advanced(&before, &after);
        let success = result.is_ok()
            && if task.kind == TaskKind::Plan {
                task.targets.iter().all(|id| {
                    after.proof_plan.inspected.get(id)
                        == Some(&crate::plan::inspection_input(&after, id))
                })
            } else if let Some(work) = &task.work {
                after
                    .proof_plan
                    .steps
                    .iter()
                    .find(|p| &p.id == work)
                    .is_some_and(|p| crate::plan::checked(&after, p))
            } else if task.kind == TaskKind::Integrate {
                task.targets
                    .iter()
                    .all(|id| crate::plan::goal_done(&after, id))
            } else if task.kind == TaskKind::Verify {
                proof_ok
            } else {
                progress
                    && task
                        .targets
                        .iter()
                        .all(|id| after.targets.get(id).is_some_and(Target::ready))
            };
        let failure = result.is_err();
        store.transaction(None, |s| {
            s.run.active = None;
            let attempt = s.run.attempts.entry(key.clone()).or_default();
            attempt.failed = !success && !progress;
            attempt.no_progress = if progress { 0 } else { attempt.no_progress + 1 };
            attempt.last_input = scheduler::task_input(&after, &task);
            attempt.last_result = if success {
                "completed with accepted work"
            } else if worker_deadline {
                "worker task deadline reached; saved checkpoints retained"
            } else if failure {
                "worker failed, interrupted, approval needed, or disconnected; partial work retained"
            } else if proof_environment_error {
                "verification build/environment error; partial work retained"
            } else {
                "unresolved; same task/input suppressed"
            }.into();
            s.run.last_result = attempt.last_result.clone();
            s.run.elapsed_ms = start.elapsed().as_millis() as u64;
            if let Some(c) = &codex {
                s.run.input_tokens = c.usage.0;
                s.run.cached_input_tokens = c.usage.1;
                s.run.output_tokens = c.usage.2;
            }
            Ok(())
        })?;
        store.event("task_finished", json!({"task":task,"input":after.input,"success":success,"forward_progress":progress,"checker_only":refreshed,"worker_deadline":worker_deadline}))?;
        if (failure && !worker_deadline) || proof_environment_error {
            if process::interrupted() {
                break;
            }
            return Ok(2);
        }
    }
    drop(codex);
    if process::interrupted() {
        return Ok(130);
    }
    s = sync(store, engine)?;
    // Final batch refresh only for ready APIs whose current input has no formal
    // execution yet. This does not regenerate already stable specifications.
    if config.verification.required && !process::interrupted() {
        let ids = s
            .targets
            .values()
            .filter(|t| {
                t.kind.proof()
                    && t.visibility != Visibility::Auxiliary
                    && s.dependencies_ready(&t.id)
                    && !s.verified(&t.id)
                    && !s
                        .results
                        .iter()
                        .any(|r| r.target == t.id && r.input == s.input)
            })
            .map(|t| t.id.clone())
            .collect::<Vec<_>>();
        if !ids.is_empty() && start.elapsed().as_secs() < limits.max_seconds {
            let result = verifier::verify(
                store,
                &ids,
                Some(s.revision),
                limits
                    .verify_seconds
                    .min(limits.max_seconds - start.elapsed().as_secs()),
            );
            let environment_error = result.as_ref().map_or(true, |rs| {
                rs.iter().any(|r| r.outcome == Outcome::EnvironmentError)
            });
            s = sync(store, engine)?;
            if environment_error {
                display(&s, &config, false, false, false)?;
                return Ok(if process::interrupted() { 130 } else { 2 });
            }
        }
    }
    display(&s, &config, false, false, false)?;
    Ok(if process::interrupted() {
        130
    } else if s.complete(config.verification.required) {
        0
    } else {
        1
    })
}
