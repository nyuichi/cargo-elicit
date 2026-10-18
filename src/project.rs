//! Model-independent project snapshots, obligations, reviews and evidence.
use crate::{
    analysis::{self, Analyzer, Selection},
    fingerprint,
    model::*,
    process,
    store::{self, Store},
    verifier,
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
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub target: Option<String>,
    pub statement: Option<String>,
    pub theorem: Option<String>,
    pub description: String,
    pub specification: String,
    pub review_specification: String,
    pub limitation: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub specification: String,
    pub verdict: String,
    pub note: String,
    pub recorded_at: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
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
    pub limitations: Vec<String>,
    pub goals: Vec<Goal>,
    pub evidence: BTreeMap<String, Vec<Evidence>>,
    pub reviews: BTreeMap<String, Review>,
    #[serde(default)]
    pub proof_modules: Option<serde_json::Value>,
}
pub fn config(store: &Store) -> Result<Config> {
    let c: Config = toml::from_str(&fs::read_to_string(store.root.join("elicit.toml"))?)?;
    if c.schema != 2 {
        bail!("unsupported configuration schema; use a new project configuration (schema = 2)");
    }
    let mut ids = std::collections::BTreeSet::new();
    for goal in &c.obligations {
        if goal.id.is_empty() || !ids.insert(&goal.id) {
            bail!("obligation IDs must be nonempty and unique");
        }
        if goal.target.is_some() == goal.statement.is_some() {
            bail!(
                "obligation {} needs either target or statement/theorem",
                goal.id
            );
        }
        if goal.target.is_none()
            && (goal.statement.as_ref().is_none_or(|s| s.trim().is_empty())
                || goal
                    .theorem
                    .as_ref()
                    .is_none_or(|s| !crate::lean_modules::lean_name(s)))
        {
            bail!("invalid proposition obligation {}", goal.id);
        }
    }
    Ok(c)
}
pub fn load(store: &Store) -> Result<Snapshot> {
    let snapshot: Snapshot = serde_json::from_slice(
        &fs::read(store.root.join(".elicit/snapshot.json"))
            .context("no project snapshot; run cargo elicit refresh")?,
    )?;
    if snapshot.schema != 2 {
        bail!("unsupported snapshot schema; existing data was preserved");
    }
    Ok(snapshot)
}
fn save(store: &Store, s: &Snapshot) -> Result<()> {
    store::atomic_write(
        &store.root.join(".elicit/snapshot.json"),
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
        bail!("elicit.toml already exists; edit it and run refresh (no implicit migration)");
    }
    store::atomic_write(&path, toml::to_string_pretty(&c)?.as_bytes())?;
    let store = Store {
        root: root.to_path_buf(),
    };
    let _lock = store.lock("project")?;
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
    let package = analysis::package(&store.root, &selection)?;
    let version = fingerprint::tool_version(config.verification.tool)
        .unwrap_or_else(|e| format!("unavailable: {e}"));
    let before = fingerprint::inputs(&package.scope, config.verification.tool, &version);
    let mut analyzer = Analyzer::default();
    let (inventory, mut limitations) = analyzer.analyze(&package, config.verification.tool)?;
    let proof_modules = if config.verification.tool == Tool::Aeneas {
        Some(
            match crate::lean_modules::Modules::load(&crate::aeneas::proof_root(&package.scope)) {
                Ok(modules) => modules.planning_view(),
                Err(e) => serde_json::json!({"error":format!("{e:#}"),"modules":[]}),
            },
        )
    } else {
        None
    };
    // Re-resolve Cargo metadata so mutations during initial scope discovery
    // cannot attach old cfg/features/dependency resolution to a newer input.
    let resolved = analysis::package(&store.root, &selection)?;
    if resolved.scope != package.scope
        || resolved.cfg != package.cfg
        || config_identity != fingerprint::hash(fs::read(store.root.join("elicit.toml"))?)
    {
        bail!("configuration/build scope changed during refresh; retry on stable files");
    }
    let after = fingerprint::inputs(&package.scope, config.verification.tool, &version);
    let (input, context, input_files) = match (before, after) {
        (Ok(before), Ok(after)) => {
            if before != after {
                bail!("inputs changed during discovery; retry refresh");
            }
            after
        }
        (Err(error), Err(_)) => {
            limitations.push(format!("tool/input inspection unavailable: {error}"));
            let key =
                fingerprint::hash(serde_json::to_vec(&(&package.scope, &inventory, &version))?);
            (key.clone(), key, BTreeMap::new())
        }
        _ => bail!("input inspection changed during refresh; retry"),
    };
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
        let subject = target
            .as_ref()
            .and_then(|id| inventory.iter().find(|t| &t.id == id));
        let specification = fingerprint::hash(serde_json::to_vec(&(
            &target,
            subject.map(|t| &t.spec_hash),
            &goal.statement,
            &goal.theorem,
        ))?);
        let limitation=match subject {
   Some(t) if !matches!(t.kind,Kind::Function|Kind::Method)=>Some("unsupported API mapping: only function/method contracts can establish API evidence".into()),
   _ if target.is_none() && config.verification.tool!=Tool::Aeneas=>Some("proposition obligations currently require Aeneas/Lean".into()),
   _=>None,
  };
        let review_specification =
            fingerprint::hash(serde_json::to_vec(&(&specification, &goal.description))?);
        goals.push(Goal {
            id: goal.id,
            target,
            statement: goal.statement,
            theorem: goal.theorem,
            description: goal.description,
            specification,
            review_specification,
            limitation,
        });
    }
    let old = if store.root.join(".elicit/snapshot.json").exists() {
        Some(load(store)?)
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
    let snapshot = Snapshot {
        schema: 2,
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
        evidence: old.as_ref().map(|s| s.evidence.clone()).unwrap_or_default(),
        reviews: old.map(|s| s.reviews).unwrap_or_default(),
    };
    save(store, &snapshot)?;
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
fn machine_success(r: &Verification) -> bool {
    r.outcome == Outcome::Success
        && r.scope == Coverage::FullContract
        && r.reusable
        && r.assumptions.is_empty()
        && r.restrictions.is_empty()
        && !r.obligations.is_empty()
        && !r.trusted_base.is_empty()
}
pub fn evidence_status<'a>(s: &'a Snapshot, g: &Goal, kind: &str) -> Option<&'a Evidence> {
    s.evidence
        .get(&g.id)?
        .iter()
        .rev()
        .find(|e| e.kind == kind && e.input == s.input && e.specification == g.specification)
}
pub fn view(s: &Snapshot) -> serde_json::Value {
    let goals=s.goals.iter().map(|g|{
  let audit=evidence_status(s,g,"audit");let check=evidence_status(s,g,"check");
  let review=s.reviews.get(&g.id);
  let review_state=match review{None=>"unreviewed",Some(r) if r.specification!=g.review_specification=>"stale",Some(r)=>r.verdict.as_str()};
  serde_json::json!({"id":g.id,"target":g.target,"description":g.description,"limitation":g.limitation,"audit":audit.map(|e|if e.accepted{"accepted"}else{"failed"}).unwrap_or("missing_or_stale"),"check":check.map(|e|if e.accepted{"accepted"}else{"failed"}).unwrap_or("missing_or_stale"),"review":review_state,"last_evidence":s.evidence.get(&g.id).and_then(|e|e.last()),"next":if audit.is_some_and(|e|e.accepted){"inspect scope/semantic review as needed"}else{"author with native tools; check when useful; audit for final evidence"}})
 }).collect::<Vec<_>>();
    let complete = !s.goals.is_empty()
        && s.goals.iter().all(|g| {
            g.limitation.is_none() && evidence_status(s, g, "audit").is_some_and(|e| e.accepted)
        });
    let uncovered = s
        .inventory
        .iter()
        .filter(|t| {
            t.visibility == Visibility::Public
                && !s.goals.iter().any(|g| g.target.as_ref() == Some(&t.id))
        })
        .map(|t| t.id.clone())
        .collect::<Vec<_>>();
    serde_json::json!({"schema":2,"observation":"saved_snapshot","workspace_currentness":"not_observed_by_this_read; run refresh or check/audit","observed_at_unix_ms":s.observed_at,"snapshot":s.input,"scope":s.scope,"declared_obligations_audited_at_snapshot":complete,"whole_package_claim":false,"uncovered_public_inventory":uncovered,"analysis_limitations":s.limitations,"changed_inputs":s.changed_inputs,"goals":goals})
}
pub fn review(store: &Store, selectors: &[String], verdict: &str, note: &str) -> Result<Snapshot> {
    if note.trim().is_empty() {
        bail!("a semantic review note is required");
    }
    let _lock = store.lock("project")?;
    let mut s = refresh(store)?;
    let goals = select(&s, selectors)?
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    for goal in goals {
        s.reviews.insert(
            goal.id,
            Review {
                specification: goal.review_specification,
                verdict: verdict.into(),
                note: note.into(),
                recorded_at: now(),
            },
        );
    }
    save(store, &s)?;
    Ok(s)
}
pub fn check(
    store: &Store,
    selectors: &[String],
    seconds: u64,
    audit: bool,
) -> Result<(Snapshot, i32)> {
    if seconds == 0 {
        bail!("seconds must be positive");
    }
    let _lock = store.lock("project")?;
    let mut snapshot = refresh(store)?;
    let goals = select(&snapshot, selectors)?
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    if goals.is_empty() {
        bail!("no obligations; declare targets or propositions in elicit.toml");
    }
    let tool = snapshot.tool;
    let state = State {
        revision: now(),
        tool,
        scope: snapshot.scope.clone(),
        input: snapshot.input.clone(),
        context_input: snapshot.context_input.clone(),
        targets: snapshot
            .inventory
            .iter()
            .map(|t| (t.id.clone(), t.clone()))
            .collect(),
        results: vec![],
        claims: goals
            .iter()
            .filter(|g| g.target.is_none())
            .map(|g| Claim {
                id: g.id.clone(),
                statement: g.statement.clone().unwrap(),
                theorem: g.theorem.clone().unwrap(),
            })
            .collect(),
    };
    let ids = goals
        .iter()
        .filter_map(|g| g.target.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut report = verifier::verify(&state, &store.root, &ids, seconds, !audit)?;
    let id = format!("{}-{}", if audit { "audit" } else { "check" }, now());
    let receipt_dir = store.root.join(".elicit/evidence");
    fs::create_dir_all(&receipt_dir)?;
    let receipt = receipt_dir.join(format!("{id}.json"));
    let artifacts = receipt_dir.join(&id);
    fs::create_dir_all(&artifacts)?;
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
        fs::copy(entry.path(), target)?;
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

    store::atomic_write(
        &receipt,
        &serde_json::to_vec_pretty(
            &serde_json::json!({"schema":2,"kind":if audit{"audit"}else{"check"},"recorded_at":now(),"scope":snapshot.scope,"input_files":snapshot.input_files,"goals":goals,"report":report}),
        )?,
    )?;
    let mut all = true;
    for goal in goals {
        let result = goal
            .target
            .as_ref()
            .and_then(|id| report.results.iter().find(|r| &r.target == id))
            .cloned();
        let claim = report
            .backend_evidence
            .as_ref()
            .map(|e| &e["claims"][&goal.id]);
        let accepted = goal.limitation.is_none()
            && report.stable
            && if goal.target.is_some() {
                result.as_ref().is_some_and(machine_success)
            } else {
                claim.is_some_and(|c| c["checked"] == true)
            };
        all &= accepted;
        let phase = result
            .as_ref()
            .and_then(|r| r.diagnostic.as_ref())
            .map(|d| d.phase.clone())
            .unwrap_or_else(|| {
                report
                    .backend_evidence
                    .as_ref()
                    .and_then(|e| e["phase"].as_str())
                    .unwrap_or("proposition-audit")
                    .into()
            });
        let reason = if let Some(reason) = &goal.limitation {
            reason.clone()
        } else if let Some(r) = &result {
            r.summary.clone()
        } else if accepted {
            "exact proposition checked; standard axioms only".into()
        } else {
            "proposition not accepted; inspect the receipt and phase logs".into()
        };
        snapshot
            .evidence
            .entry(goal.id)
            .or_default()
            .push(Evidence {
                kind: if audit { "audit" } else { "check" }.into(),
                recorded_at: now(),
                input: report.input.clone(),
                specification: goal.specification,
                accepted,
                result,
                receipt: receipt
                    .strip_prefix(&store.root)?
                    .to_string_lossy()
                    .into_owned(),
                phase,
                reason,
            });
    }
    // Observation and evidence are distinct: a raced check never makes the old
    // input look current. Refresh is explicit on the next write/check operation.
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
    Ok((snapshot, code))
}
pub fn show(s: &Snapshot, selector: &str) -> Result<serde_json::Value> {
    let goal = select(s, &[selector.into()])?[0];
    let target = goal
        .target
        .as_ref()
        .and_then(|id| s.inventory.iter().find(|t| &t.id == id));
    Ok(
        serde_json::json!({"schema":2,"observation":"saved_snapshot","workspace_currentness":"not_observed_by_this_read; run refresh or check/audit","observed_at_unix_ms":s.observed_at,"goal":goal,"target":target,"aeneas_contract":goal.target.as_ref().map(|id|crate::aeneas::contract_name(id)),"evidence":s.evidence.get(&goal.id),"review":s.reviews.get(&goal.id),"next":"Edit native contracts/proofs freely; check for iteration; audit for final evidence. No submission or revision token is required."}),
    )
}
