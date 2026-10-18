//! Explicit staged semantic approval, collected from stable saved artifacts.
use crate::{
    analysis::Analyzer,
    cli, fingerprint,
    model::*,
    store::{self, Store},
    verifier,
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, time::Instant};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    schema_version: u32,
    context: String,
    targets: BTreeMap<String, String>,
    note: String,
}
fn path(store: &Store) -> std::path::PathBuf {
    store.root.join(".elicit/staged-review.json")
}
fn read(store: &Store) -> Result<Option<Review>> {
    match fs::read(path(store)) {
        Ok(bytes) => {
            let r: Review = serde_json::from_slice(&bytes).context("invalid staged review")?;
            if r.schema_version != 1 || r.note.trim().is_empty() || r.targets.is_empty() {
                bail!("invalid staged review");
            }
            Ok(Some(r))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn matches(s: &State, r: &Review) -> bool {
    r.context == s.context_input
        && r.targets.iter().all(|(id, hash)| {
            s.targets.get(id).is_some_and(|t| {
                !t.removed
                    && !t.implementation_changed
                    && &t.spec_hash == hash
                    && s.unsupported(id).is_none()
            })
        })
}
pub fn stage(store: &Store, s: &State, ids: &[String], all: bool, note: &str) -> Result<()> {
    if s.tool != Tool::Aeneas {
        bail!("staged artifact collection currently requires Aeneas");
    }
    if note.trim().is_empty() {
        bail!("a semantic review note is required");
    }
    if all == !ids.is_empty() {
        bail!("choose explicit target IDs or --all");
    }
    crate::lean_modules::Modules::load(&crate::aeneas::proof_root(&s.scope))?;
    let candidates = if all {
        crate::aeneas::declared_targets(s)?
    } else {
        ids.to_vec()
    };
    let mut targets = BTreeMap::new();
    for id in candidates {
        let t = s.targets.get(&id).context("unknown staged target")?;
        if t.removed || t.implementation_changed || s.unsupported(&id).is_some() {
            bail!("cannot stage changed, removed or unsupported target {id}");
        }
        crate::aeneas::check_submission(&s.scope, t)?;
        targets.insert(id, t.spec_hash.clone());
    }
    if targets.is_empty() {
        bail!("no declared contracts to stage");
    }
    let _review = store.lock("review")?;
    // Hold the state lock to ensure the snapshot identity is still the one
    // reconciled by this operation. Proof terms may evolve after staging;
    // the approved semantic identities and original runtime must not.
    let _state = store.state_lock()?;
    let current = store.load()?;
    if current.revision != s.revision || current.input != s.input {
        bail!("input/revision changed while staging; retry on stable sources");
    }
    let current_input = fingerprint::input(&s.scope, s.tool, &fingerprint::tool_version(s.tool)?)?;
    if current_input != s.input {
        bail!("inputs changed while staging; reconcile and retry");
    }
    let review = Review {
        schema_version: 1,
        context: s.context_input.clone(),
        targets,
        note: note.into(),
    };
    store::atomic_write(&path(store), &serde_json::to_vec_pretty(&review)?)?;
    store.event(
        "review_staged",
        serde_json::json!({"targets":review.targets,"input":s.input}),
    )?;
    Ok(())
}
pub fn status(store: &Store, s: &State) -> &'static str {
    match read(store) {
        Ok(None) => "none",
        Ok(Some(r)) if matches(s, &r) => "pending",
        Ok(Some(_)) => "stale",
        Err(_) => "invalid",
    }
}
/// The caller holds the run lock. No worker may be active. Formal checking
/// precedes approval so the same pipeline validates contracts and collects
/// proof evidence, without one syntax pipeline per submission.
pub fn collect(store: &Store, engine: &mut Analyzer, seconds: u64) -> Result<State> {
    let start = Instant::now();
    let _review = store.lock("review")?;
    let s = cli::sync(store, engine)?;
    if s.tool != Tool::Aeneas {
        bail!("artifact collection currently requires Aeneas");
    }
    let review = read(store)?;
    if review.as_ref().is_some_and(|r| !matches(&s, r)) {
        store.event(
            "review_rejected",
            serde_json::json!({"reason":"semantic or runtime input changed","input":s.input}),
        )?;
        bail!(
            "staged review no longer matches the contracts/model/runtime; preserve files and stage a new explicit review"
        );
    }
    let mut ids = review
        .as_ref()
        .map(|r| {
            r.targets
                .keys()
                .filter(|id| s.targets[*id].kind.proof())
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ids.extend(
        crate::aeneas::declared_targets(&s)?
            .into_iter()
            .filter(|id| s.targets[id].ready()),
    );
    ids.sort();
    ids.dedup();
    let needed = ids
        .iter()
        .filter(|id| {
            !s.results.iter().any(|r| {
                &r.target == *id
                    && r.input == s.input
                    && r.outcome == Outcome::Success
                    && r.reusable
                    && r.scope == Coverage::FullContract
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    let required = store.config()?.verification.required;
    let remaining = seconds.saturating_sub(start.elapsed().as_secs());
    if remaining == 0 {
        bail!("collection budget exhausted before checking");
    }
    store.event(
        "collection_started",
        serde_json::json!({"input":s.input,"targets":ids,"staged":review.is_some()}),
    )?;
    if required {
        if !needed.is_empty()
            || crate::plan::checkpoint_needed(&s)
            || review.as_ref().is_some_and(|review| {
                s.processing_input != s.input
                    || review
                        .targets
                        .keys()
                        .any(|id| s.targets[id].kind.proof() && !s.processed_contracts.contains(id))
            })
        {
            let results = verifier::verify(store, &needed, Some(s.revision), remaining)?;
            if results
                .iter()
                .any(|r| r.outcome == Outcome::EnvironmentError)
            {
                bail!(
                    "verification environment failed; saved artifacts and staged review preserved"
                );
            }
        }
    } else if review.is_some() && s.processing_input != s.input {
        let _worker = store.lock("worker")?;
        let input = verifier::syntax(store, &s, remaining)?;
        store.transaction(Some(s.revision), |state| {
            state.processing_input = input;
            Ok(())
        })?;
    }
    if crate::process::interrupted() {
        bail!("collection interrupted; no staged approvals accepted");
    }
    let after = cli::sync(store, engine)?;
    if let Some(mut review) = review {
        if !matches(&after, &review) || after.input != s.input {
            bail!("inputs changed during collection; staged approvals not accepted");
        }
        if after.processing_input == after.input {
            let _worker = store.lock("worker")?;
            let actual = fingerprint::input(
                &after.scope,
                after.tool,
                &fingerprint::tool_version(after.tool)?,
            )?;
            if actual != after.input {
                bail!("inputs changed before approval; staged review preserved");
            }
            let mut accepted = review.clone();
            accepted.targets.retain(|id, _| {
                !required
                    || !after.targets[id].kind.proof()
                    || after.processed_contracts.contains(id)
            });
            if accepted.targets.is_empty() {
                return Ok(after);
            }
            store.transaction(Some(after.revision), |state| {
                let hashes = state
                    .targets
                    .iter()
                    .filter(|(_, t)| !t.removed)
                    .map(|(id, t)| (id.clone(), t.spec_hash.clone()))
                    .collect::<BTreeMap<_, _>>();
                for (id, hash) in &accepted.targets {
                    let t = state
                        .targets
                        .get_mut(id)
                        .context("staged target disappeared")?;
                    if &t.spec_hash != hash || t.implementation_changed || t.removed {
                        bail!("staged target changed");
                    }
                    for (dependency, hash) in &mut t.dependencies {
                        *hash = hashes
                            .get(dependency)
                            .context("reviewed specification dependency was removed")?
                            .clone();
                    }
                    t.bind_models();
                    t.approved_spec_hash = Some(t.spec_hash.clone());
                    t.specified = true;
                    t.blocker = None;
                    t.stale.clear();
                    t.notes.push(review.note.clone());
                }
                Ok(())
            })?;
            store::atomic_write(
                &store.root.join(".elicit/last-review.json"),
                &serde_json::to_vec_pretty(&accepted)?,
            )?;
            review
                .targets
                .retain(|id, _| !accepted.targets.contains_key(id));
            if review.targets.is_empty() {
                fs::remove_file(path(store))?;
            } else {
                store::atomic_write(&path(store), &serde_json::to_vec_pretty(&review)?)?;
            }
            store.event("review_collected",serde_json::json!({"targets":accepted.targets,"unprocessed":review.targets,"input":after.input}))?;
        }
    }
    let after = cli::sync(store, engine)?;
    store.event("collection_finished",serde_json::json!({"elapsed_ms":start.elapsed().as_millis(),"input":after.input,"selected_goals_verified":after.selected_goals_verified()}))?;
    Ok(after)
}
