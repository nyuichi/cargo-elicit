//! Run user-selected native commands. No theorem or API admission policy.
use crate::{
    fingerprint,
    model::*,
    process,
    project::{Goal, Snapshot},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub results: Vec<Verification>,
    pub input: String,
    pub stable: bool,
    pub cache: String,
    pub elapsed_ms: u64,
    pub environment_error: bool,
}

pub fn verify(s: &Snapshot, root: &Path, goals: &[Goal], seconds: u64) -> Result<Report> {
    let start = Instant::now();
    let dir = root.join(".elicit/cache/run");
    // Only transient command logs live here; native builds remain tool-owned.
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    let before = fingerprint::input(&s.scope, s.tool, &s.tool_version)?;
    anyhow::ensure!(
        before == s.input,
        "inputs changed before command execution; retry on stable files"
    );
    let mut results = Vec::new();
    let mut environment_error = false;
    for (index, goal) in goals.iter().enumerate() {
        let log = format!("command-{index}.log");
        let mut result = Verification {
            target: goal.id.clone(),
            command: goal.command.clone(),
            directory: goal.directory.clone(),
            exit_code: None,
            outcome: Outcome::Unresolved,
            input: s.input.clone(),
            summary: String::new(),
            diagnostic: None,
            elapsed_ms: 0,
        };
        // One invocation per distinct command/directory in this request. Shared
        // project builds are user-declared associations, not API coverage proofs.
        if let Some(previous) = results
            .iter()
            .find(|r: &&Verification| r.command == goal.command && r.directory == goal.directory)
        {
            result = previous.clone();
            result.target = goal.id.clone();
            results.push(result);
            continue;
        }
        let execution = if let Some(program) = goal.command.first() {
            process::run(
                Command::new(program)
                    .args(&goal.command[1..])
                    .current_dir(root.join(&goal.directory)),
                Duration::from_secs(seconds),
            )
        } else {
            Err(anyhow::anyhow!(
                "no native command configured; set verification.command in elicit.toml"
            ))
        };
        let output = match execution {
            Ok(o) => {
                result.exit_code = o.code;
                result.elapsed_ms = o.elapsed_ms;
                result.outcome = if o.interrupted {
                    Outcome::Interrupted
                } else if o.timeout {
                    Outcome::Timeout
                } else if o.code == Some(0) {
                    Outcome::Success
                } else {
                    Outcome::Unresolved
                };
                result.summary = if o.interrupted {
                    "native command interrupted".into()
                } else if o.timeout {
                    "native command timed out".into()
                } else {
                    format!(
                        "native command exited {}; no theorem, axiom or API coverage checks",
                        o.code
                            .map_or_else(|| "without an exit code".into(), |c| c.to_string())
                    )
                };
                format!("{}\n{}", o.stdout, o.stderr)
            }
            Err(e) => {
                environment_error = true;
                result.outcome = Outcome::EnvironmentError;
                result.summary = format!("cannot run native command: {e:#}");
                result.summary.clone()
            }
        };
        fs::write(dir.join(&log), &output)?;
        if result.outcome != Outcome::Success {
            let message = output
                .lines()
                .find(|l| l.to_ascii_lowercase().contains("error"))
                .or_else(|| output.lines().find(|l| !l.trim().is_empty()))
                .unwrap_or(&result.summary);
            result.diagnostic = Some(Diagnostic {
                phase: "command".into(),
                cache: dir.to_string_lossy().into_owned(),
                message: Some(crate::ui::one_line(message, 300)),
                log: Some(log),
            });
        }
        results.push(result);
        if process::interrupted() {
            break;
        }
    }
    // Tool versions are re-observed as well as command executables and inputs.
    let version = fingerprint::tool_version(s.tool)?;
    let after = fingerprint::input(&s.scope, s.tool, &version);
    let stable = version == s.tool_version
        && after.as_ref().is_ok_and(|x| x == &before)
        && !process::interrupted();
    if !stable {
        for result in &mut results {
            if result.outcome == Outcome::Success {
                result.outcome = Outcome::InputChanged;
                result.summary =
                    "inputs changed during native command; result is not current".into();
            }
        }
    }
    if after.is_err() {
        environment_error = true;
    }
    Ok(Report {
        results,
        input: before,
        stable,
        cache: dir.to_string_lossy().into_owned(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        environment_error,
    })
}
