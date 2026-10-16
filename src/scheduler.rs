use crate::model::*;
use petgraph::{algo::tarjan_scc, graph::DiGraph};
use std::collections::BTreeMap;
#[derive(Debug, Clone, clap::Args)]
pub struct Limits {
    #[arg(long, default_value_t = 32)]
    pub max_tasks: u32,
    #[arg(long, default_value_t = 3600)]
    pub max_seconds: u64,
    #[arg(long, default_value_t = 600)]
    pub task_seconds: u64,
    #[arg(long, default_value_t = 600)]
    pub verify_seconds: u64,
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
            verify_seconds: 600,
            max_attempts: 3,
            max_no_progress: 2,
        }
    }
}
pub fn eligible(s: &State, t: &Task, l: &Limits) -> bool {
    s.run.attempts.get(&t.key()).is_none_or(|a| {
        a.total < l.max_attempts
            && a.no_progress < l.max_no_progress
            && !(a.failed && a.last_input == s.input)
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
    // Tarjan emits sinks first for caller -> callee edges.
    tarjan_scc(&graph)
        .into_iter()
        .map(|ns| {
            let mut ids = ns.iter().map(|n| graph[*n].clone()).collect::<Vec<_>>();
            ids.sort();
            ids
        })
        .collect()
}
pub fn next(s: &State, required: bool, l: &Limits) -> Option<Task> {
    let groups = groups(s);
    for g in &groups {
        let ids = g
            .iter()
            .filter(|id| !s.targets[*id].stale.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        if !ids.is_empty() {
            let t = Task {
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
            if g.iter().all(|id| s.targets[id].ready()) {
                let ids = g
                    .iter()
                    .filter(|id| s.targets[*id].kind.proof() && !s.verified(id))
                    .cloned()
                    .collect::<Vec<_>>();
                if !ids.is_empty() {
                    let t = Task {
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
                    !t.specified
                        && t.stale.is_empty()
                        && matches!(t.kind, Kind::Struct | Kind::Enum | Kind::Trait) == fundamental
                })
                .cloned()
                .collect::<Vec<_>>();
            if !ids.is_empty() {
                let t = Task {
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
pub fn progress(s: &State) -> String {
    crate::fingerprint::hash(
        serde_json::to_vec(
            &s.targets
                .values()
                .map(|t| {
                    (
                        &t.id,
                        t.specified,
                        &t.spec_hash,
                        &t.stale,
                        t.implementation_changed,
                        s.verified(&t.id),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
}
