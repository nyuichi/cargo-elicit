//! Goal-directed orchestration. A plan is advice, never proof evidence.
use crate::{fingerprint, model::*, scheduler::Limits};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "apis", rename_all = "snake_case")]
pub enum VerificationGoal {
    #[default]
    WholePackage,
    SelectedApis(Vec<String>),
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Contract,
    Lemma,
    Integrate,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofStep {
    pub id: String,
    pub goal: String,
    pub kind: StepKind,
    pub target: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub needs: Vec<String>,
    pub statement: Option<String>,
    pub theorem: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LemmaEvidence {
    pub input: String,
    pub step_hash: String,
    pub outcome: Outcome,
    pub cache: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProofPlan {
    #[serde(default)]
    pub last_checkpoint: String,
    pub steps: Vec<ProofStep>,
    #[serde(default)]
    pub evidence: BTreeMap<String, LemmaEvidence>,
    #[serde(default)]
    pub inspected: BTreeMap<String, String>,
}
pub fn inspection_input(s: &State, id: &str) -> String {
    let t = &s.targets[id];
    fingerprint::hash(
        serde_json::to_vec(&(
            &t.body_hash,
            &t.source_hash,
            s.tool,
            t.model_inputs.get("@rust_model"),
            t.model_inputs.get("@tcb"),
        ))
        .unwrap(),
    )
}
pub fn goal_done(s: &State, id: &str) -> bool {
    s.targets.get(id).is_some_and(|t| {
        if t.kind.proof() {
            s.verified(id)
        } else {
            t.ready()
        }
    })
}
pub fn roots(s: &State) -> Vec<String> {
    match &s.goal {
        VerificationGoal::SelectedApis(ids) => ids.clone(),
        VerificationGoal::WholePackage => s
            .targets
            .values()
            .filter(|t| {
                !t.removed
                    && t.visibility != Visibility::Auxiliary
                    && s.unsupported(&t.id).is_none()
            })
            .map(|t| t.id.clone())
            .collect(),
    }
}
pub fn resolve(s: &State, selectors: &[String]) -> Result<Vec<String>> {
    let mut ids = BTreeSet::new();
    for selector in selectors {
        let matches: Vec<_> = s
            .targets
            .values()
            .filter(|t| {
                !t.removed
                    && t.kind.proof()
                    && (&t.id == selector || &t.name == selector || t.aliases.contains(selector))
            })
            .collect();
        if matches.len() != 1 {
            bail!(
                "API selector {selector:?} matched {} targets; use an exact target ID",
                matches.len()
            );
        }
        ids.insert(matches[0].id.clone());
    }
    Ok(ids.into_iter().collect())
}
pub fn step_hash(step: &ProofStep) -> String {
    fingerprint::hash(
        serde_json::to_vec(&(
            &step.id,
            &step.goal,
            step.kind,
            &step.target,
            &step.needs,
            &step.statement,
            &step.theorem,
        ))
        .unwrap(),
    )
}
pub fn checked(s: &State, step: &ProofStep) -> bool {
    match step.kind {
        StepKind::Lemma => s.proof_plan.evidence.get(&step.id).is_some_and(|e| {
            e.input == s.input && e.step_hash == step_hash(step) && e.outcome == Outcome::Success
        }),
        _ => step.target.as_ref().is_some_and(|id| {
            s.targets.get(id).is_some_and(|t| {
                if t.kind.proof() {
                    s.verified(id)
                } else {
                    t.ready()
                }
            })
        }),
    }
}
pub fn install(s: &mut State, steps: Vec<ProofStep>) -> Result<()> {
    if s.tool != Tool::Aeneas {
        bail!("proof plans currently require Aeneas");
    }
    let roots = roots(s);
    let mut ids = BTreeSet::new();
    for step in &steps {
        if step.id.is_empty()
            || !step
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            || !ids.insert(step.id.clone())
        {
            bail!("invalid or duplicate proof step ID");
        }
        if !roots.contains(&step.goal) || step.reason.trim().is_empty() {
            bail!("step must name a goal and explain its use at the caller");
        }
        match step.kind {
            StepKind::Lemma => {
                if step.statement.as_ref().is_none_or(|s| s.trim().is_empty())
                    || step.theorem.as_ref().is_none_or(|s| !lean_name(s))
                    || step.target.is_some()
                {
                    bail!("lemma requires a Lean proposition and theorem name, not a Rust target");
                }
            }
            _ => {
                let id = step
                    .target
                    .as_ref()
                    .context("contract/integration step requires target")?;
                let t = s.targets.get(id).context("unknown plan target")?;
                if t.removed || s.unsupported(id).is_some() {
                    bail!("removed or unsupported plan target");
                }
                if step.kind == StepKind::Integrate && id != &step.goal {
                    bail!("integration must return to its goal");
                }
                if step.statement.is_some() || step.theorem.is_some() {
                    bail!("Rust steps use the recorded API contract");
                }
            }
        }
    }
    fn visit<'a>(
        id: &'a str,
        map: &BTreeMap<&'a str, &'a ProofStep>,
        path: &mut BTreeSet<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> Result<()> {
        if done.contains(id) {
            return Ok(());
        }
        if !path.insert(id) {
            bail!("cyclic proof plan");
        }
        let step = map.get(id).context("unknown step dependency")?;
        for dep in &step.needs {
            let other = map.get(dep.as_str()).context("unknown step dependency")?;
            if other.goal != step.goal {
                bail!("step dependencies must belong to the same goal");
            }
            visit(dep, map, path, done)?;
        }
        path.remove(id);
        done.insert(id);
        Ok(())
    }
    let map = steps.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut done = BTreeSet::new();
    for step in &steps {
        visit(&step.id, &map, &mut BTreeSet::new(), &mut done)?;
    }
    // Delegating an API to another selected root is an implicit dependency.
    // Reject cycles rather than waiting forever for mutually deferred goals.
    let mut edges: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for step in &steps {
        if delegated(s, step) {
            edges
                .entry(&step.goal)
                .or_default()
                .push(step.target.as_deref().unwrap());
        }
    }
    fn root_visit<'a>(
        id: &'a str,
        edges: &BTreeMap<&'a str, Vec<&'a str>>,
        path: &mut BTreeSet<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> Result<()> {
        if done.contains(id) {
            return Ok(());
        }
        if !path.insert(id) {
            bail!("cyclic root delegation; use joint checked lemmas instead");
        }
        if let Some(needs) = edges.get(id) {
            for dep in needs {
                root_visit(dep, edges, path, done)?;
            }
        }
        path.remove(id);
        done.insert(id);
        Ok(())
    }
    let mut root_done = BTreeSet::new();
    for id in edges.keys() {
        root_visit(id, &edges, &mut BTreeSet::new(), &mut root_done)?;
    }
    // A plan cannot drop an existing goal contract or alter its required units.
    for step in &steps {
        s.proof_plan
            .inspected
            .insert(step.goal.clone(), inspection_input(s, &step.goal));
    }
    s.proof_plan.evidence.retain(|id, e| {
        steps
            .iter()
            .any(|s| &s.id == id && step_hash(s) == e.step_hash)
    });
    s.proof_plan.steps = steps;
    Ok(())
}
pub fn lean_name(s: &str) -> bool {
    !s.is_empty()
        && s.split('.').all(|p| {
            !p.is_empty()
                && p.chars().enumerate().all(|(i, c)| {
                    c.is_ascii_alphabetic() || c == '_' || i > 0 && c.is_ascii_digit()
                })
        })
}
pub fn next(s: &State, limits: &Limits) -> Option<Task> {
    // Continue a caller and its concrete prerequisites before inspecting an
    // unrelated root. Delegated selected APIs retain their own proof plans.
    for root in roots(s) {
        if let Some(task) = next_root(s, &root, limits, &mut BTreeSet::new()) {
            return Some(task);
        }
    }
    None
}
fn next_root(
    s: &State,
    root: &str,
    limits: &Limits,
    visiting: &mut BTreeSet<String>,
) -> Option<Task> {
    if goal_done(s, root) || !s.schedulable(root) || !visiting.insert(root.to_owned()) {
        return None;
    }
    if s.proof_plan.inspected.get(root) != Some(&inspection_input(s, root)) {
        // The first turn is a real proof attempt with the normal worker budget,
        // not a mandatory short planning phase for every selected API.
        let task = Task {
            kind: TaskKind::Integrate,
            targets: vec![root.to_owned()],
            work: None,
        };
        if crate::scheduler::eligible(s, &task, limits) {
            return Some(task);
        }
    }
    let mut pending = false;
    let mut waiting_on_root = false;
    for step in s.proof_plan.steps.iter().filter(|p| p.goal == root) {
        if checked(s, step) {
            continue;
        }
        pending = true;
        if !step.needs.iter().all(|id| {
            s.proof_plan
                .steps
                .iter()
                .find(|p| &p.id == id)
                .is_some_and(|p| checked(s, p))
        }) {
            continue;
        }
        if delegated(s, step) {
            waiting_on_root = true;
            if let Some(task) = next_root(s, step.target.as_deref().unwrap(), limits, visiting) {
                return Some(task);
            }
            continue;
        }
        let target = step.target.as_deref().unwrap_or(root);
        if !s.schedulable(target) {
            continue;
        }
        let task = Task {
            kind: match step.kind {
                StepKind::Lemma => TaskKind::ProveLemma,
                StepKind::Contract => TaskKind::Verify,
                StepKind::Integrate => TaskKind::Integrate,
            },
            targets: vec![target.to_owned()],
            work: Some(step.id.clone()),
        };
        if crate::scheduler::eligible(s, &task, limits) {
            return Some(task);
        }
    }
    if !pending {
        let task = Task {
            kind: TaskKind::Integrate,
            targets: vec![root.to_owned()],
            work: None,
        };
        if crate::scheduler::eligible(s, &task, limits) {
            return Some(task);
        }
    }
    visiting.remove(root);
    if !waiting_on_root {
        let task = Task {
            kind: TaskKind::Plan,
            targets: vec![root.to_owned()],
            work: Some("repair".into()),
        };
        if crate::scheduler::eligible(s, &task, limits) {
            return Some(task);
        }
    }
    None
}

pub fn checkpoint_key(s: &State) -> String {
    fingerprint::hash(
        serde_json::to_vec(&(
            &s.input,
            s.proof_plan
                .steps
                .iter()
                .filter(|p| p.kind == StepKind::Lemma)
                .map(step_hash)
                .collect::<Vec<_>>(),
        ))
        .unwrap(),
    )
}
pub fn checkpoint_needed(s: &State) -> bool {
    s.proof_plan
        .steps
        .iter()
        .any(|p| p.kind == StepKind::Lemma && !checked(s, p))
        && s.proof_plan.last_checkpoint != checkpoint_key(s)
        && std::path::Path::new(&s.scope.manifest_path)
            .parent()
            .is_some_and(|p| p.join("elicit-proofs/aeneas/Lemmas.lean").is_file())
}

pub fn task_input(s: &State, task: &Task) -> String {
    let steps: Vec<_> = s
        .proof_plan
        .steps
        .iter()
        .filter(|p| {
            task.work.as_ref().is_some_and(|w| &p.id == w)
                || task.kind == TaskKind::Plan && task.targets.contains(&p.goal)
        })
        .collect();
    let mut ids = BTreeSet::new();
    for step in &steps {
        ids.insert(step.id.as_str());
        ids.extend(step.needs.iter().map(String::as_str));
    }
    let feedback: Vec<_> = s
        .proof_plan
        .steps
        .iter()
        .filter(|p| ids.contains(p.id.as_str()))
        .map(|p| {
            (
                &p.id,
                step_hash(p),
                s.proof_plan
                    .evidence
                    .get(&p.id)
                    .map(|e| (e.outcome, e.step_hash == step_hash(p))),
                checked(s, p),
            )
        })
        .collect();
    let target_inputs: Vec<_> = task.targets.iter().map(|id| s.target_input(id)).collect();
    let proof_files: Vec<_> = if task.kind == TaskKind::Plan {
        vec![]
    } else {
        s.input_files
            .iter()
            .filter(|(path, _)| path.ends_with(".lean"))
            .collect()
    };
    fingerprint::hash(
        serde_json::to_vec(&(
            target_inputs,
            steps.iter().map(|p| step_hash(p)).collect::<Vec<_>>(),
            feedback,
            proof_files,
        ))
        .unwrap(),
    )
}

pub(crate) fn delegated(s: &State, step: &ProofStep) -> bool {
    step.kind == StepKind::Contract
        && step
            .target
            .as_ref()
            .is_some_and(|id| id != &step.goal && roots(s).contains(id))
}
