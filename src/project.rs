//! Model-independent project snapshots, obligations and evidence.
use crate::{
    analysis::Selection,
    evidence::{input_identities, portable_inputs, read_evidence},
    fingerprint,
    model::*,
    process,
    store::{self, Store},
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: u32,
    pub verification: VerificationConfig,
    #[serde(default)]
    pub package: PackageConfig,
    #[serde(default)]
    pub obligations: Vec<Obligation>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationConfig {
    pub tool: Tool,
    /// Native verification argv, executed directly without shell interpolation.
    #[serde(default)]
    pub command: Vec<String>,
    /// Working directory relative to the project root.
    #[serde(default)]
    pub directory: Option<String>,
    /// Additional input files/directories, including external proof libraries.
    #[serde(default)]
    pub inputs: Vec<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageConfig {
    pub name: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub all_features: bool,
    #[serde(default)]
    pub no_default_features: bool,
    pub target: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    pub id: String,
    pub target: Option<String>,
    pub statement: Option<String>,
    pub theorem: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default)]
    pub directory: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub target: Option<String>,
    pub statement: Option<String>,
    pub theorem: Option<String>,
    pub description: String,
    pub specification: String,
    pub limitation: Option<String>,
    pub command: Vec<String>,
    pub directory: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source_input: String,
    pub environment_input: String,
    #[serde(default)]
    pub input_files: BTreeMap<String, String>,
    pub kind: String,
    pub recorded_at: u64,
    pub input: String,
    pub specification: String,
    pub accepted: bool,
    pub result: Option<Verification>,
    pub receipt: String,
    pub phase: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema: u32,
    pub tool: Tool,
    pub observed_at: u64,
    pub input: String,
    pub context_input: String,
    pub input_files: BTreeMap<String, String>,
    pub changed_inputs: Vec<String>,
    pub tool_version: String,
    pub scope: Scope,
    pub inventory: Vec<Target>,
    #[serde(default)]
    pub analysis_reused: bool,
    pub limitations: Vec<String>,
    pub goals: Vec<Goal>,
    pub evidence: BTreeMap<String, Vec<Evidence>>,
    #[serde(default)]
    pub proof_modules: Option<serde_json::Value>,
}
pub fn config(store: &Store) -> Result<Config> {
    let c: Config = toml::from_str(&fs::read_to_string(store.root.join("elicit.toml"))?)?;
    if c.schema != 3 {
        bail!("unsupported configuration schema; use a new project configuration (schema = 3)");
    }
    let mut ids = std::collections::BTreeSet::new();
    for goal in &c.obligations {
        if goal.id.is_empty() || !ids.insert(&goal.id) {
            bail!("obligation IDs must be nonempty and unique");
        }
        if goal.command.first().is_some_and(|p| p.trim().is_empty()) {
            bail!("command executable must not be empty for {}", goal.id);
        }
    }
    if c.verification
        .command
        .first()
        .is_some_and(|p| p.trim().is_empty())
    {
        bail!("verification.command executable must not be empty");
    }
    Ok(c)
}
fn load_cached(store: &Store) -> Result<Snapshot> {
    let value: serde_json::Value = serde_json::from_slice(
        &fs::read(store.root.join(".elicit/cache/snapshot.json"))
            .context("no project snapshot; run cargo elicit status")?,
    )?;
    if value["schema"] != 4 {
        bail!(
            "unsupported snapshot schema; existing data was preserved (native-command snapshot schema is 4)"
        );
    }
    Ok(serde_json::from_value(value)?)
}
pub fn load(store: &Store) -> Result<Snapshot> {
    let mut snapshot = load_cached(store)?;
    let (evidence, warnings) = read_evidence(store)?;
    snapshot.evidence = evidence;
    snapshot.limitations.extend(warnings);
    Ok(snapshot)
}
fn save(store: &Store, s: &Snapshot) -> Result<()> {
    store::atomic_write(
        &store.root.join(".elicit/cache/snapshot.json"),
        &serde_json::to_vec_pretty(s)?,
    )
}
pub fn resolve(targets: &[Target], selector: &str) -> Result<String> {
    let matches = targets
        .iter()
        .filter(|t| {
            t.id == selector || t.name == selector || t.aliases.iter().any(|a| a == selector)
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        bail!(
            "selector {selector:?} matched {} targets; use an exact ID from list",
            matches.len()
        );
    }
    Ok(matches[0].id.clone())
}
pub fn init(root: &Path, c: Config) -> Result<Snapshot> {
    let path = root.join("elicit.toml");
    if path.exists() {
        bail!("elicit.toml already exists; edit it and run status (no implicit migration)");
    }
    store::atomic_write(&path, toml::to_string_pretty(&c)?.as_bytes())?;
    let store = Store {
        root: root.to_path_buf(),
    };
    let _lock = store.lock("project")?;
    store.prepare_gitignore()?;
    refresh(&store)
}
/// Explicit observation: Cargo/discovery and fingerprints; no verifier compilation.
pub fn refresh(store: &Store) -> Result<Snapshot> {
    let config_identity = fingerprint::hash(fs::read(store.root.join("elicit.toml"))?);
    let config = config(store)?;
    let selection = Selection {
        package: config.package.name.clone(),
        features: config.package.features.clone(),
        all_features: config.package.all_features,
        no_default_features: config.package.no_default_features,
        target: config.package.target.clone(),
    };
    let observed = crate::observation::refresh(store, &config, &selection)?;
    let package = observed.package;
    let inventory = observed.inventory;
    let limitations = observed.limitations;
    let (input, context, raw_input_files) = observed.inputs;
    let input_files = portable_inputs(&store.root, &package.scope, &raw_input_files)?;
    let version = observed.version;
    let proof_modules = if config.verification.tool == Tool::Aeneas {
        Some(
            match crate::lean_modules::Modules::load(
                &store
                    .root
                    .join(config.verification.directory.as_deref().unwrap_or(".")),
            ) {
                Ok(modules) => modules.planning_view(),
                Err(e) => serde_json::json!({"error":format!("{e:#}"),"modules":[]}),
            },
        )
    } else {
        None
    };
    anyhow::ensure!(
        config_identity == fingerprint::hash(fs::read(store.root.join("elicit.toml"))?),
        "configuration changed during refresh; retry on stable files"
    );
    let definitions = if config.obligations.is_empty() {
        inventory
            .iter()
            .filter(|t| t.visibility == Visibility::Public && t.kind.proof())
            .map(|t| Obligation {
                id: t.id.clone(),
                target: Some(t.id.clone()),
                statement: None,
                theorem: None,
                description: String::new(),
                command: Vec::new(),
                directory: None,
            })
            .collect::<Vec<_>>()
    } else {
        config.obligations.clone()
    };
    let mut goals = vec![];
    for goal in definitions {
        let target = goal
            .target
            .as_ref()
            .map(|t| resolve(&inventory, t))
            .transpose()?;
        let command = if goal.command.is_empty() {
            config.verification.command.clone()
        } else {
            goal.command.clone()
        };
        let directory = goal
            .directory
            .clone()
            .or_else(|| config.verification.directory.clone())
            .unwrap_or_else(|| ".".into());
        let specification = fingerprint::hash(serde_json::to_vec(&(
            "native-command-v1",
            &target,
            &goal.statement,
            &goal.theorem,
            &command,
            &directory,
        ))?);
        let limitation = command.is_empty().then(|| "no verification command configured; set verification.command or obligations[].command in elicit.toml".into());
        goals.push(Goal {
            id: goal.id,
            target,
            statement: goal.statement,
            theorem: goal.theorem,
            description: goal.description,
            specification,
            limitation,
            command,
            directory,
        });
    }
    let old = if store.root.join(".elicit/cache/snapshot.json").exists() {
        load_cached(store).ok()
    } else {
        None
    };
    let changed_inputs = if let Some(old) = &old {
        old.input_files
            .keys()
            .chain(input_files.keys())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|key| old.input_files.get(*key) != input_files.get(*key))
            .cloned()
            .collect()
    } else {
        input_files.keys().cloned().collect()
    };
    let (evidence, warnings) = read_evidence(store)?;
    let mut limitations = limitations;
    limitations.extend(warnings);
    let snapshot = Snapshot {
        schema: 4,
        analysis_reused: observed.reused,
        proof_modules,
        tool: config.verification.tool,
        observed_at: now(),
        input,
        context_input: context,
        input_files,
        changed_inputs,
        tool_version: version,
        scope: package.scope,
        inventory,
        limitations,
        goals,
        evidence,
    };
    save(store, &snapshot)?;
    if !snapshot.changed_inputs.is_empty() {
        store.history(
            "inputs_observed",
            serde_json::json!({"input":snapshot.input,"changed_input_count":snapshot.changed_inputs.len(),"changed_inputs":snapshot.changed_inputs.iter().take(20).collect::<Vec<_>>()}),
        )?;
    }
    Ok(snapshot)
}
pub fn select<'a>(s: &'a Snapshot, selectors: &[String]) -> Result<Vec<&'a Goal>> {
    if selectors.is_empty() {
        return Ok(s.goals.iter().collect());
    }
    let mut goals = vec![];
    for selector in selectors {
        let matches = s
            .goals
            .iter()
            .filter(|g| {
                &g.id == selector
                    || g.target.as_ref().is_some_and(|id| {
                        id == selector
                            || s.inventory.iter().any(|t| {
                                &t.id == id && (&t.name == selector || t.aliases.contains(selector))
                            })
                    })
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            bail!(
                "obligation selector {selector:?} is missing or ambiguous; use an obligation ID from list"
            );
        }
        if !goals.iter().any(|g: &&Goal| g.id == matches[0].id) {
            goals.push(matches[0]);
        }
    }
    Ok(goals)
}
pub(crate) fn machine_success(r: &Verification) -> bool {
    r.outcome == Outcome::Success && r.exit_code == Some(0)
}
pub fn current_evidence<'a>(s: &'a Snapshot, g: &Goal) -> Option<&'a Evidence> {
    s.evidence
        .get(&g.id)?
        .iter()
        .rev()
        .find(|e| e.input == s.input && e.specification == g.specification)
}
fn evidence_view(e: &Evidence) -> serde_json::Value {
    let mut value = serde_json::json!(e);
    value.as_object_mut().unwrap().remove("input_files");
    value
}
pub fn view(s: &Snapshot) -> serde_json::Value {
    let (source, environment) = input_identities(&s.input_files).expect("serializable input maps");
    let goals = s.goals.iter().map(|g| {
        let current = current_evidence(s, g);
        let last = s.evidence.get(&g.id).and_then(|es| es.last());
        serde_json::json!({"id":g.id,"target":g.target,"description":g.description,
            "command":g.command,"directory":g.directory,
            "command_status":current.map(|e|if e.accepted{"succeeded"}else{"failed"}).unwrap_or("missing_or_stale"),
            "historical_result":last.map(|e|if e.accepted{"succeeded"}else{"failed_or_interrupted"}).unwrap_or("none"),
            "source_status":last.map(|e|if e.source_input==source && e.specification==g.specification{"unchanged"}else{"changed"}).unwrap_or("unknown"),
            "environment_status":last.map(|e|if e.environment_input==environment{"matching"}else{"different"}).unwrap_or("unknown"),
            "limitation":g.limitation,
            "last_evidence":s.evidence.get(&g.id).and_then(|e|e.last()).map(evidence_view),
            "next":"work with native tools; verify at useful milestones"})
    }).collect::<Vec<_>>();
    let complete = !s.goals.is_empty()
        && s.goals
            .iter()
            .all(|g| current_evidence(s, g).is_some_and(|e| e.accepted));
    let uncovered = s
        .inventory
        .iter()
        .filter(|t| {
            t.visibility == Visibility::Public
                && !s.goals.iter().any(|g| g.target.as_ref() == Some(&t.id))
        })
        .map(|t| t.id.clone())
        .collect::<Vec<_>>();
    serde_json::json!({"schema":4,"observation":"saved_snapshot","workspace_currentness":"not_observed_by_this_read; run status or verify","observed_at_unix_ms":s.observed_at,"snapshot":s.input,"scope":s.scope,"declared_commands_succeeded_at_snapshot":complete,"whole_package_claim":false,"evidence_semantics":"native_command_result; no theorem, axiom or Rust correspondence checks","uncovered_public_inventory":uncovered,"analysis_limitations":s.limitations,"analysis_reused":s.analysis_reused,"changed_inputs":s.changed_inputs,"goals":goals})
}
// A saved status bit alone is insufficient: require the durable receipt to agree
// with the exact observed inputs, specification, scope and accepted native result.
fn reusable_receipt(
    store: &Store,
    s: &Snapshot,
    g: &Goal,
    cache: &mut BTreeMap<String, Option<serde_json::Value>>,
) -> bool {
    if g.limitation.is_some() {
        return false;
    }
    let e = current_evidence(s, g);
    let Some(e) = e.filter(|e| e.accepted) else {
        return false;
    };
    let path = Path::new(&e.receipt);
    if path.is_absolute()
        || path
            .components()
            .any(|p| !matches!(p, std::path::Component::Normal(_)))
        || !e.receipt.starts_with(".elicit/evidence/")
    {
        return false;
    }
    let Some(receipt) = cache
        .entry(e.receipt.clone())
        .or_insert_with(|| {
            fs::read(store.root.join(path))
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        })
        .as_ref()
    else {
        return false;
    };
    if receipt["kind"] != e.kind
        || receipt["input_files"] != serde_json::json!(s.input_files)
        || receipt["report"]["stable"] != true
        || receipt["report"]["input"] != s.input
        || !receipt["goals"].as_array().is_some_and(|gs| {
            gs.iter()
                .any(|x| x["id"] == g.id && x["specification"] == g.specification)
        })
    {
        return false;
    }
    let Some(result) = e.result.as_ref().filter(|r| machine_success(r)) else {
        return false;
    };
    result.command == g.command
        && result.directory == g.directory
        && receipt["report"]["results"].as_array().is_some_and(|rs| {
            rs.iter().any(|r| {
                r["target"] == g.id
                    && *r == {
                        let mut value = serde_json::json!(result);
                        if let Some(d) = value["diagnostic"].as_object_mut() {
                            d.remove("message");
                        }
                        value
                    }
            })
        })
}

pub fn check(
    store: &Store,
    selectors: &[String],
    seconds: u64,
    fresh: bool,
) -> Result<(Snapshot, i32, usize)> {
    let started = std::time::Instant::now();
    if seconds == 0 {
        bail!("seconds must be positive");
    }
    let _lock = store.lock("project")?;
    store.prepare_gitignore()?;
    let mut snapshot = refresh(store)?;
    let goals = select(&snapshot, selectors)?
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    if goals.is_empty() {
        bail!("no goals; declare obligations in elicit.toml");
    }
    let mut receipt_cache = BTreeMap::new();
    let mut reused = Vec::new();
    let goals = if fresh {
        goals
    } else {
        goals
            .into_iter()
            .filter(|g| {
                let keep = !reusable_receipt(store, &snapshot, g, &mut receipt_cache);
                if !keep {
                    reused.push(g.id.clone());
                }
                keep
            })
            .collect::<Vec<_>>()
    };
    if goals.is_empty()
        && (fingerprint::tool_version(snapshot.tool)? != snapshot.tool_version
            || fingerprint::input(&snapshot.scope, snapshot.tool, &snapshot.tool_version)?
                != snapshot.input)
    {
        bail!("inputs changed while validating saved evidence; retry on stable files");
    }
    if !reused.is_empty() {
        store.history(
            "evidence_reused",
            serde_json::json!({"input":snapshot.input,"goals":reused,"elapsed_ms":started.elapsed().as_millis()}),
        )?;
    }
    if goals.is_empty() {
        return Ok((snapshot, 0, reused.len()));
    }
    let mut report = crate::runner::verify(&snapshot, &store.root, &goals, seconds)?;
    let receipt_dir = store.root.join(".elicit/evidence");
    fs::create_dir_all(&receipt_dir)?;
    let logs = store.root.join(".elicit/logs");
    fs::create_dir_all(&logs)?;
    let artifacts = tempfile::Builder::new()
        .prefix("run-")
        .tempdir_in(&logs)?
        .keep();
    let mut artifact_refs = Vec::new();
    let source_cache = std::path::Path::new(&report.cache);
    for entry in walkdir::WalkDir::new(source_cache)
        .into_iter()
        .filter_entry(|e| e.path() != source_cache.join("rust-build"))
    {
        let entry = entry?;
        if !entry.file_type().is_file() || entry.file_type().is_symlink() {
            continue;
        }
        let extension = entry
            .path()
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if !["lean", "log", "llbc"].contains(&extension) {
            continue;
        }
        let target = artifacts.join(entry.path().strip_prefix(source_cache)?);
        fs::create_dir_all(target.parent().unwrap())?;
        fs::copy(entry.path(), &target)?;
        let bytes = fs::read(&target)?;
        artifact_refs.push(serde_json::json!({
            "path":target.strip_prefix(&store.root)?.to_string_lossy(),
            "sha256":fingerprint::hash(&bytes),"bytes":bytes.len()
        }));
    }
    for result in &mut report.results {
        if let Some(diagnostic) = &mut result.diagnostic {
            diagnostic.cache = artifacts
                .strip_prefix(&store.root)?
                .to_string_lossy()
                .into_owned();
        }
    }
    report.cache = artifacts
        .strip_prefix(&store.root)?
        .to_string_lossy()
        .into_owned();

    let (source_input, environment_input) = input_identities(&snapshot.input_files)?;
    store::atomic_write(
        &artifacts.join("diagnostics.json"),
        &serde_json::to_vec(&report.results)?,
    )?;
    let mut published_report = serde_json::to_value(&report)?;
    // Raw diagnostic excerpts remain local; committed records reference logs.
    for result in published_report["results"].as_array_mut().unwrap() {
        if let Some(d) = result["diagnostic"].as_object_mut() {
            d.remove("message");
        }
    }
    let bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema":4,"kind":if fresh{"rerun"}else{"run"},"recorded_at":now(),
        "source_input":source_input,"environment_input":environment_input,
        "input_files":snapshot.input_files,"goals":goals,"report":published_report,"artifacts":artifact_refs
    }))?;
    let receipt = receipt_dir.join(format!("{}.json", fingerprint::hash(&bytes)));
    store::atomic_write(&receipt, &bytes)?;
    let all = goals.iter().all(|goal| {
        report.stable
            && report
                .results
                .iter()
                .find(|r| r.target == goal.id)
                .is_some_and(machine_success)
    });
    // Observation and evidence are distinct: a raced check never makes the old
    // input look current. Refresh is explicit on the next write/check operation.
    // Reload canonical records and optional local diagnostics.
    snapshot.evidence = read_evidence(store)?.0;
    save(store, &snapshot)?;
    let code = if process::interrupted() {
        130
    } else if report.environment_error {
        2
    } else if all {
        0
    } else {
        1
    };
    store.history("verification_finished", serde_json::json!({"input":snapshot.input,"fresh":fresh,"exit_code":code,"elapsed_ms":started.elapsed().as_millis(),
        "goals":snapshot.evidence.iter().filter_map(|(goal_id,es)|es.last().filter(|e|e.receipt==receipt.strip_prefix(&store.root).unwrap().to_string_lossy()).map(|e|serde_json::json!({"id":goal_id,"accepted":e.accepted,"receipt":e.receipt,"phase":e.phase,"reason":e.reason}))).collect::<Vec<_>>() }))?;
    Ok((snapshot, code, reused.len()))
}
pub fn show(s: &Snapshot, selector: &str) -> Result<serde_json::Value> {
    let goal = select(s, &[selector.into()])?[0];
    let target = goal
        .target
        .as_ref()
        .and_then(|id| s.inventory.iter().find(|t| &t.id == id));
    Ok(
        serde_json::json!({"schema":4,"observation":"saved_snapshot","workspace_currentness":"not_observed_by_this_read; run status or verify","observed_at_unix_ms":s.observed_at,"goal":goal,"target":target,"progress":view(s)["goals"].as_array().unwrap().iter().find(|g|g["id"]==goal.id),"evidence":s.evidence.get(&goal.id),"next":"Work with native tools; verify at useful milestones; verify --fresh to rerun the configured command."}),
    )
}
