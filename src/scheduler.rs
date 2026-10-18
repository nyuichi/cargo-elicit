use crate::model::*;
use petgraph::{algo::tarjan_scc, graph::DiGraph};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, clap::Args)]
pub struct Limits {
    #[arg(long, default_value_t = 32)]
    pub max_tasks: u32,
    #[arg(long, default_value_t = 3600)]
    pub max_seconds: u64,
    #[arg(long, default_value_t = 600)]
    pub task_seconds: u64,
    #[arg(long, default_value_t = 180)]
    pub plan_seconds: u64,
    #[arg(long, default_value_t = 600)]
    pub verify_seconds: u64,
    /// Reserve time inside the overall budget for final reconciliation and checks.
    #[arg(long, default_value_t = 60)]
    pub finalize_seconds: u64,
    #[arg(long, default_value_t = 3)]
    pub max_attempts: u32,
    #[arg(long, default_value_t = 2)]
    pub max_no_progress: u32,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_tasks: 32,
            max_seconds: 3600,
            task_seconds: 600,
            plan_seconds: 180,
            verify_seconds: 600,
            finalize_seconds: 60,
            max_attempts: 3,
            max_no_progress: 2,
        }
    }
}
pub fn authoring_seconds(limits: &Limits, required: bool) -> u64 {
    let reserved = if required {
        limits.finalize_seconds.min(limits.max_seconds / 4)
    } else {
        0
    };
    limits.max_seconds - reserved
}
pub fn task_input(s: &State, t: &Task) -> String {
    if t.kind == TaskKind::Author {
        return s.input.clone();
    }
    if t.work.is_some()
        || matches!(
            t.kind,
            TaskKind::Plan | TaskKind::ProveLemma | TaskKind::Integrate
        )
    {
        return crate::plan::task_input(s, t);
    }
    if t.kind == TaskKind::Verify {
        return s.input.clone();
    }
    crate::fingerprint::hash(
        serde_json::to_vec(
            &t.targets
                .iter()
                .map(|id| s.target_input(id))
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
}
// Scheduling credit is bookkeeping, never proof acceptance. Use proposition
// text rather than plan IDs/theorem selectors so aliases and changed needs do
// not buy another attempt. Omitting model changes is intentionally conservative.
fn checked_lemmas(s: &State, goals: Option<&BTreeSet<String>>) -> BTreeSet<String> {
    s.proof_plan
        .steps
        .iter()
        .filter(|step| {
            goals.is_none_or(|goals| goals.contains(&step.goal))
                && step.kind == crate::plan::StepKind::Lemma
                && crate::plan::checked(s, step)
        })
        .filter_map(|step| step.statement.as_ref())
        .map(|statement| {
            format!(
                "lemma:{}",
                statement.split_whitespace().collect::<Vec<_>>().join(" ")
            )
        })
        .collect()
}
fn insert_checked_apis(s: &State, ids: &[String], facts: &mut BTreeSet<String>) {
    for id in ids {
        if s.verified(id) {
            facts.insert(format!("api:{id}:{}", s.targets[id].spec_hash));
        }
    }
}
/// Snapshot globally known facts before a turn, so moving existing work into
/// its scope cannot earn credit. Existing per-attempt history is retained too.
pub fn known_checked_work(s: &State) -> BTreeSet<String> {
    let mut facts = checked_lemmas(s, None);
    insert_checked_apis(
        s,
        &s.targets.keys().cloned().collect::<Vec<_>>(),
        &mut facts,
    );
    facts
}
pub fn checked_work(s: &State, task: &Task) -> BTreeSet<String> {
    let mut goals = task
        .work
        .as_ref()
        .and_then(|id| s.proof_plan.steps.iter().find(|p| &p.id == id))
        .map_or_else(
            || task.targets.iter().cloned().collect(),
            |p| BTreeSet::from([p.goal.clone()]),
        );
    // A selected child can produce lemmas owned by its callers. Follow only
    // reverse delegation edges: siblings and unrelated roots are not callers.
    loop {
        let previous = goals.len();
        for step in &s.proof_plan.steps {
            if crate::plan::delegated(s, step)
                && step.target.as_ref().is_some_and(|id| goals.contains(id))
            {
                goals.insert(step.goal.clone());
            }
        }
        if goals.len() == previous {
            break;
        }
    }
    let mut facts = checked_lemmas(s, Some(&goals));
    // API credit remains scoped to the actual task targets.
    insert_checked_apis(s, &task.targets, &mut facts);
    facts
}
pub fn record_checked_progress(attempt: &mut Attempt, facts: BTreeSet<String>) {
    let novel = facts
        .iter()
        .any(|fact| !attempt.checked_work.contains(fact));
    attempt.checked_work.extend(facts);
    if novel {
        attempt.progress_credits = (attempt.progress_credits + 1).min(attempt.total);
    }
}
pub fn eligible(s: &State, t: &Task, l: &Limits) -> bool {
    s.run.attempts.get(&t.key()).is_none_or(|a| {
        a.total.saturating_sub(a.progress_credits) < l.max_attempts
            && a.no_progress < l.max_no_progress
            && !(a.failed && a.last_input == task_input(s, t))
    })
}
pub fn groups(s: &State) -> Vec<Vec<String>> {
    let mut graph = DiGraph::<String, ()>::new();
    let mut nodes = BTreeMap::new();
    for t in s
        .targets
        .values()
        .filter(|t| !t.removed && t.visibility != Visibility::Auxiliary)
    {
        nodes.insert(t.id.clone(), graph.add_node(t.id.clone()));
    }
    for (id, n) in &nodes {
        for call in &s.targets[id].calls {
            if let Some(c) = nodes.get(call) {
                graph.add_edge(*n, *c, ());
            }
        }
    }
    let components: Vec<Vec<String>> = tarjan_scc(&graph)
        .into_iter()
        .map(|ns| {
            let mut ids = ns.iter().map(|n| graph[*n].clone()).collect::<Vec<_>>();
            ids.sort();
            ids
        })
        .collect();
    let mut remaining: BTreeSet<usize> = (0..components.len()).collect();
    let owners = components
        .iter()
        .enumerate()
        .flat_map(|(i, ids)| ids.iter().map(move |id| (id.clone(), i)))
        .collect::<BTreeMap<_, _>>();
    let mut ordered = vec![];
    while !remaining.is_empty() {
        let mut sinks = remaining
            .iter()
            .copied()
            .filter(|i| {
                components[*i].iter().all(|id| {
                    s.targets[id].calls.iter().all(|call| {
                        owners
                            .get(call)
                            .is_none_or(|j| j == i || !remaining.contains(j))
                    })
                })
            })
            .collect::<Vec<_>>();
        sinks.sort_by(|a, b| components[*a].cmp(&components[*b]));
        for i in sinks {
            remaining.remove(&i);
            ordered.push(components[i].clone());
        }
    }
    ordered
}

pub fn next(s: &State, required: bool, l: &Limits) -> Option<Task> {
    if s.tool == Tool::Aeneas && required {
        return crate::plan::next(s, l);
    }
    let groups = groups(s);
    for g in &groups {
        let ids = g
            .iter()
            .filter(|id| s.schedulable(id) && !s.targets[*id].stale.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        if !ids.is_empty() {
            let t = Task {
                work: None,
                kind: TaskKind::Recheck,
                targets: ids,
            };
            if eligible(s, &t, l) {
                return Some(t);
            }
        }
    }
    // Finish the current group before opening another group. Failed proof tasks
    // are suppressed, so a hard proof never blocks all remaining specification work.
    if required {
        for g in &groups {
            if g.iter()
                .all(|id| s.schedulable(id) && s.targets[id].ready())
            {
                let ids = g
                    .iter()
                    .filter(|id| s.targets[*id].kind.proof() && !s.verified(id))
                    .cloned()
                    .collect::<Vec<_>>();
                if !ids.is_empty() {
                    let t = Task {
                        work: None,
                        kind: TaskKind::Verify,
                        targets: ids,
                    };
                    if eligible(s, &t, l) {
                        return Some(t);
                    }
                }
            }
        }
    }
    for fundamental in [true, false] {
        for g in &groups {
            let ids = g
                .iter()
                .filter(|id| {
                    let t = &s.targets[*id];
                    s.schedulable(id)
                        && !t.specified
                        && t.stale.is_empty()
                        && matches!(t.kind, Kind::Struct | Kind::Enum | Kind::Trait) == fundamental
                })
                .cloned()
                .collect::<Vec<_>>();
            if !ids.is_empty() {
                let t = Task {
                    work: None,
                    kind: TaskKind::Specify,
                    targets: ids,
                };
                if eligible(s, &t, l) {
                    return Some(t);
                }
            }
        }
    }
    None
}
// Diagnostic text changes and regressions are not forward progress.
pub fn advanced(before: &State, after: &State) -> bool {
    after.proof_plan.steps.iter().any(|step| {
        crate::plan::checked(after, step)
            && !before
                .proof_plan
                .steps
                .iter()
                .find(|s| s.id == step.id)
                .is_some_and(|s| crate::plan::checked(before, s))
    }) || after.targets.iter().any(|(id, t)| {
        let was_ready = before.targets.get(id).is_some_and(Target::ready);
        (!was_ready && t.ready()) || (!before.verified(id) && after.verified(id))
    })
}
