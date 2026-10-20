//! Portable immutable command records; local caches are never the history source.
use crate::{
    fingerprint,
    model::*,
    project::{Evidence, Goal, machine_success},
    store::Store,
};
use anyhow::{Context, Result};
use std::{collections::BTreeMap, fs, path::Path};
type History = BTreeMap<String, Vec<Evidence>>;

// Persist only portable project paths and opaque environment hashes. Native
// execution still uses the conservative full local fingerprint for reuse/races.
pub(crate) fn portable_inputs(
    root: &Path,
    scope: &Scope,
    files: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for (path, digest) in files {
        let key = if path.starts_with('@') {
            path.clone()
        } else if let Ok(relative) = Path::new(path).strip_prefix(root) {
            relative.to_string_lossy().replace('\\', "/")
        } else {
            format!("@external:{}", fingerprint::hash(path))
        };
        out.insert(key, digest.clone());
    }
    out.insert(
        "@project_scope".into(),
        fingerprint::hash(serde_json::to_vec(&(
            &scope.package,
            &scope.features,
            &scope.resolved_features,
            scope.all_features,
            scope.no_default_features,
            &scope.target,
            &scope.requested_target,
        ))?),
    );
    Ok(out)
}
fn source_key(key: &str) -> bool {
    !key.starts_with('@') || matches!(key, "@declarations" | "@project_scope")
}
pub(crate) fn input_identities(files: &BTreeMap<String, String>) -> Result<(String, String)> {
    let (source, environment): (BTreeMap<_, _>, BTreeMap<_, _>) =
        files.iter().partition(|(k, _)| source_key(k));
    Ok((
        fingerprint::hash(serde_json::to_vec(&source)?),
        fingerprint::hash(serde_json::to_vec(&environment)?),
    ))
}
pub(crate) fn read_evidence(store: &Store) -> Result<(History, Vec<String>)> {
    let dir = store.root.join(".elicit/evidence");
    let mut records = Vec::new();
    let mut warnings = Vec::new();
    if !dir.exists() {
        return Ok((BTreeMap::new(), warnings));
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let read = || -> Result<Vec<(String, Evidence)>> {
            anyhow::ensure!(entry.file_type()?.is_file(), "not a regular record");
            let bytes = fs::read(&path)?;
            anyhow::ensure!(
                path.file_stem().and_then(|s| s.to_str())
                    == Some(fingerprint::hash(&bytes).as_str()),
                "record hash mismatch"
            );
            let v: serde_json::Value = serde_json::from_slice(&bytes)?;
            anyhow::ensure!(v["schema"] == 4, "unsupported record schema");
            let files: BTreeMap<String, String> = serde_json::from_value(v["input_files"].clone())?;
            anyhow::ensure!(
                files.keys().all(|k| k.starts_with('@')
                    || Path::new(k)
                        .components()
                        .all(|c| matches!(c, std::path::Component::Normal(_)))),
                "nonportable project input path"
            );
            let (source, environment) = input_identities(&files)?;
            anyhow::ensure!(
                v["source_input"] == source && v["environment_input"] == environment,
                "input identity mismatch"
            );
            let mut report: crate::runner::Report = serde_json::from_value(v["report"].clone())?;
            let cache = Path::new(&report.cache);
            anyhow::ensure!(
                cache.starts_with(".elicit/logs")
                    && cache
                        .components()
                        .all(|c| matches!(c, std::path::Component::Normal(_))),
                "invalid log reference"
            );
            let local: Vec<Verification> =
                fs::read(store.root.join(cache).join("diagnostics.json"))
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok())
                    .unwrap_or_default();
            for r in &mut report.results {
                if let Some(d) = &mut r.diagnostic {
                    d.message = local
                        .iter()
                        .find(|x| x.target == r.target)
                        .and_then(|x| x.diagnostic.as_ref())
                        .and_then(|x| x.message.clone());
                }
            }
            let goals: Vec<Goal> = serde_json::from_value(v["goals"].clone())?;
            let mut out = Vec::new();
            for goal in goals {
                let result = report
                    .results
                    .iter()
                    .find(|r| {
                        r.target == goal.id
                            && r.command == goal.command
                            && r.directory == goal.directory
                    })
                    .cloned();
                let accepted = report.stable && result.as_ref().is_some_and(machine_success);
                out.push((
                    goal.id,
                    Evidence {
                        source_input: source.clone(),
                        environment_input: environment.clone(),
                        input_files: files.clone(),
                        kind: v["kind"].as_str().context("missing kind")?.into(),
                        recorded_at: v["recorded_at"].as_u64().context("missing timestamp")?,
                        input: report.input.clone(),
                        specification: goal.specification,
                        accepted,
                        reason: result
                            .as_ref()
                            .map(|r| r.summary.clone())
                            .unwrap_or_else(|| "command was not executed".into()),
                        result,
                        receipt: path
                            .strip_prefix(&store.root)?
                            .to_string_lossy()
                            .into_owned(),
                        phase: "command".into(),
                    },
                ));
            }
            Ok(out)
        };
        match read() {
            Ok(es) => records.extend(es),
            Err(e) => warnings.push(format!(
                "ignored invalid evidence {}: {e}",
                entry.file_name().to_string_lossy()
            )),
        }
    }
    records.sort_by(|a, b| (a.1.recorded_at, &a.1.receipt).cmp(&(b.1.recorded_at, &b.1.receipt)));
    let mut evidence: BTreeMap<String, Vec<Evidence>> = BTreeMap::new();
    for (id, e) in records {
        evidence.entry(id).or_default().push(e);
    }
    Ok((evidence, warnings))
}
