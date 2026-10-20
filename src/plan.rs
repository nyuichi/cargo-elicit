//! Saved-data planning aids. Call edges are observations, never proof prerequisites.
use crate::project::{self, Snapshot};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type Graph = BTreeMap<String, BTreeSet<String>>;
fn reachable(graph: &Graph, root: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![root.to_owned()];
    while let Some(id) = stack.pop() {
        if seen.insert(id.clone()) {
            stack.extend(graph.get(&id).into_iter().flatten().cloned());
        }
    }
    seen
}
// Iterative Kosaraju: bounded call stack even for deep generated code.
fn components(graph: &Graph) -> Vec<Vec<String>> {
    let mut visited = BTreeSet::new();
    let mut finish = vec![];
    for root in graph.keys() {
        let mut stack = vec![(root.clone(), false)];
        while let Some((id, exiting)) = stack.pop() {
            if exiting {
                finish.push(id);
            } else if visited.insert(id.clone()) {
                stack.push((id.clone(), true));
                for next in &graph[&id] {
                    if !visited.contains(next) {
                        stack.push((next.clone(), false));
                    }
                }
            }
        }
    }
    let mut reverse: Graph = graph
        .keys()
        .map(|id| (id.clone(), BTreeSet::new()))
        .collect();
    for (from, tos) in graph {
        for to in tos {
            reverse.get_mut(to).unwrap().insert(from.clone());
        }
    }
    visited.clear();
    let mut result = vec![];
    for root in finish.into_iter().rev() {
        if visited.contains(&root) {
            continue;
        }
        let mut members = vec![];
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if visited.insert(id.clone()) {
                stack.extend(reverse[&id].iter().cloned());
                members.push(id);
            }
        }
        members.sort();
        result.push(members);
    }
    result.sort();
    result
}

pub fn build(s: &Snapshot, selectors: &[String]) -> Result<Value> {
    let inventory: BTreeMap<_, _> = s
        .inventory
        .iter()
        .filter(|t| t.kind.proof())
        .map(|t| (t.id.clone(), t))
        .collect();
    let mut roots = BTreeSet::new();
    let mut standalone = vec![];
    if selectors.is_empty() {
        for g in &s.goals {
            if let Some(id) = &g.target {
                if inventory.contains_key(id) {
                    roots.insert(id.clone());
                }
            } else {
                standalone.push(g.id.clone());
            }
        }
    } else {
        for selector in selectors {
            // Obligation IDs take precedence over inventory aliases.
            if let Some(g) = s.goals.iter().find(|g| &g.id == selector) {
                if let Some(id) = &g.target {
                    if inventory.contains_key(id) {
                        roots.insert(id.clone());
                    } else {
                        bail!(
                            "target for {selector:?} is missing from saved inventory; refresh first"
                        );
                    }
                } else {
                    standalone.push(g.id.clone());
                }
                continue;
            }
            let matches: Vec<_> = inventory
                .values()
                .filter(|t| {
                    &t.id == selector || &t.name == selector || t.aliases.contains(selector)
                })
                .collect();
            if matches.len() != 1 {
                bail!(
                    "plan selector {selector:?} is missing or ambiguous; use an exact inventory or obligation ID"
                );
            }
            roots.insert(matches[0].id.clone());
        }
    }
    let full: Graph = inventory
        .iter()
        .map(|(id, t)| {
            (
                id.clone(),
                t.calls
                    .iter()
                    .filter(|to| inventory.contains_key(*to))
                    .cloned()
                    .collect(),
            )
        })
        .collect();
    let reach: BTreeMap<_, _> = roots
        .iter()
        .map(|id| (id.clone(), reachable(&full, id)))
        .collect();
    let included: BTreeSet<_> = reach.values().flatten().cloned().collect();
    let graph: Graph = full
        .iter()
        .filter(|(id, _)| included.contains(*id))
        .map(|(id, tos)| (id.clone(), tos.clone()))
        .collect();
    let groups = components(&graph);
    // IDs use the lexicographically first member, stable across traversal orders.
    let owner: BTreeMap<_, _> = groups
        .iter()
        .flat_map(|ms| ms.iter().map(|m| (m.clone(), ms[0].clone())))
        .collect();
    let dag: Graph = groups
        .iter()
        .map(|ms| {
            let id = &ms[0];
            (
                id.clone(),
                ms.iter()
                    .flat_map(|m| graph[m].iter())
                    .map(|to| owner[to].clone())
                    .filter(|to| to != id)
                    .collect(),
            )
        })
        .collect();
    // Leaf-first layers; members of a layer have no edges to one another.
    let mut levels = BTreeMap::new();
    while levels.len() < dag.len() {
        let ready: Vec<_> = dag
            .iter()
            .filter(|(id, deps)| {
                !levels.contains_key(*id) && deps.iter().all(|d| levels.contains_key(d))
            })
            .map(|(id, deps)| {
                (
                    id.clone(),
                    deps.iter().map(|d| levels[d] + 1usize).max().unwrap_or(0),
                )
            })
            .collect();
        for (id, level) in ready {
            levels.insert(id, level);
        }
    }
    let saved = project::view(s);
    let goals = saved["goals"].as_array().unwrap();
    let mut nodes = BTreeMap::new();
    for id in &included {
        let t = inventory[id];
        let bindings: Vec<_> = goals
            .iter()
            .filter(|g| g["target"].as_str() == Some(id))
            .cloned()
            .collect();
        let checked = bindings.iter().any(|g| g["command_status"] == "succeeded");
        let callers: Vec<_> = full
            .iter()
            .filter(|(_, tos)| tos.contains(id))
            .map(|(from, _)| from.clone())
            .collect();
        let selected_roots: Vec<_> = reach
            .iter()
            .filter(|(_, rs)| rs.contains(id))
            .map(|(r, _)| r.clone())
            .collect();
        let unknown_targets: Vec<_> = t
            .calls
            .iter()
            .filter(|to| !inventory.contains_key(*to))
            .cloned()
            .collect();
        nodes.insert(id.clone(), json!({"id":id,"scc":owner[id],"location":{"file":t.file,"line":t.line,"column":t.column},
            "source_changed_since_previous_observation":s.changed_inputs.contains(&t.file),"source_facts":t.source_facts,"visibility":t.visibility,"selected_root":roots.contains(id),"selected_roots_reaching":selected_roots,
            "selected_root_count":selected_roots.len(),"calls":t.calls,"callers":callers,
            "callers_in_scope":callers.iter().filter(|c|included.contains(*c)).collect::<Vec<_>>(),
            "unresolved_calls":t.unresolved_calls,"external_calls":t.external_calls,"missing_inventory_targets":unknown_targets,
            "obligations":bindings,"has_accepted_evidence":checked,
            "evidence_state":if checked {"accepted_at_snapshot"} else if bindings.is_empty(){"not_declared"}else{"missing_failed_or_stale"},
            "specification_state":"user-declared command association; inspect native specifications and caller requirements"}));
    }
    let mut rows = vec![];
    for ms in groups {
        let id = &ms[0];
        let deps = &dag[id];
        let accepted = ms.iter().all(|m| nodes[m]["has_accepted_evidence"] == true);
        let pending: Vec<_> = deps
            .iter()
            .filter(|d| {
                nodes
                    .values()
                    .any(|n| n["scc"] == d.as_str() && n["has_accepted_evidence"] != true)
            })
            .cloned()
            .collect();
        let callers: Vec<_> = dag
            .iter()
            .filter(|(_, ds)| ds.contains(id))
            .map(|(c, _)| c.clone())
            .collect();
        let affected: BTreeSet<_> = ms
            .iter()
            .flat_map(|m| {
                reach
                    .iter()
                    .filter(move |(_, rs)| rs.contains(m))
                    .map(|(r, _)| r.clone())
            })
            .collect();
        let uncertain = ms.iter().any(|m| {
            !inventory[m].unresolved_calls.is_empty()
                || inventory[m]
                    .calls
                    .iter()
                    .any(|to| !inventory.contains_key(to))
        });
        rows.push(json!({"id":id,"members":ms,"recursive":ms.len()>1 || graph[id].contains(id),
            "layer":levels[id],"dependencies":deps,"callers":callers,"selected_roots_reaching":affected,
            "selected_root_count":affected.len(),"all_members_have_accepted_evidence":accepted,
            "pending_dependency_sccs":pending,"call_resolution_incomplete":uncertain,
            "candidate":!accepted && pending.is_empty(),
            "nodes":ms.iter().map(|m|nodes[m].clone()).collect::<Vec<_>>()}));
    }
    rows.sort_by(|a, b| {
        a["layer"]
            .as_u64()
            .cmp(&b["layer"].as_u64())
            .then_with(|| {
                b["selected_root_count"]
                    .as_u64()
                    .cmp(&a["selected_root_count"].as_u64())
            })
            .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
    });
    let standalone: Vec<_> = goals
        .iter()
        .filter(|g| standalone.iter().any(|id| g["id"] == id.as_str()))
        .cloned()
        .collect();
    let missing: Vec<_> = s
        .goals
        .iter()
        .filter(|g| {
            g.target
                .as_ref()
                .is_some_and(|id| !inventory.contains_key(id))
        })
        .map(|g| g.id.clone())
        .collect();
    Ok(
        json!({"schema":1,"observation":"saved_snapshot","snapshot":s.input,"observed_at_unix_ms":s.observed_at,
        "workspace_currentness":"not_observed_by_this_read; use status to observe inputs",
        "scope":s.scope,"proof_module_imports":s.proof_modules,"selected_roots":roots,"components":rows,"standalone_obligations":standalone,
        "missing_inventory_obligations":missing,"analysis_limitations":s.limitations,"changed_inputs":s.changed_inputs,
        "semantics":"Implementation call graph, not a proof dependency graph. Candidate means no known unaccepted callee SCC; not a prerequisite or a guarantee of contract reuse. Confirm caller requirements top-down; inspect contracts and prove bottom-up. External calls, dynamic dispatch, macros and indirect calls can leave gaps. Old snapshots may not record external calls. Saved evidence records the declared command result at this snapshot; it does not establish API coverage or theorem reuse."}),
    )
}

fn strings(v: &Value) -> String {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect::<Vec<_>>()
        .join(", ")
}
pub fn render(v: &Value) -> String {
    use std::fmt::Write;
    let mut out = format!(
        "Dependency plan — snapshot {} at {}\n{}\nSelected API roots: {}\n",
        v["snapshot"].as_str().unwrap(),
        v["observed_at_unix_ms"],
        v["workspace_currentness"]
            .as_str()
            .unwrap_or("inspect observation metadata"),
        strings(&v["selected_roots"])
    );
    let _ = writeln!(
        out,
        "Scope: package={} target={} features={}",
        v["scope"]["package"], v["scope"]["target"], v["scope"]["resolved_features"]
    );
    let candidates: Vec<_> = v["components"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["candidate"] == true)
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    let _ = writeln!(
        out,
        "Candidates with no known open callee SCC: {}",
        candidates.join(", ")
    );
    out.push_str("Layers follow known call edges, dependencies first. Within a layer, shared roots first; this is not an estimated fastest proof order.\n");
    for row in v["components"].as_array().unwrap() {
        let state = if row["all_members_have_accepted_evidence"] == true {
            "ACCEPTED"
        } else if row["candidate"] == true {
            "CANDIDATE"
        } else {
            "DEPENDENCIES OPEN"
        };
        let _ = writeln!(
            out,
            "\nLayer {} | {state} | SCC {}{} | reaches {} selected root(s)",
            row["layer"],
            row["id"].as_str().unwrap(),
            if row["recursive"] == true {
                " (recursive; consider together)"
            } else {
                ""
            },
            row["selected_root_count"]
        );
        let _ = writeln!(
            out,
            "  Callee SCCs: {}\n  Open callee SCCs: {}\n  Caller SCCs: {}\n  Selected roots using this SCC: {}",
            strings(&row["dependencies"]),
            strings(&row["pending_dependency_sccs"]),
            strings(&row["callers"]),
            strings(&row["selected_roots_reaching"])
        );
        for n in row["nodes"].as_array().unwrap() {
            let _ = writeln!(
                out,
                "  {} [{}] at {}:{}\n    Calls: {}\n    Callers (whole inventory): {}",
                n["id"].as_str().unwrap(),
                n["evidence_state"].as_str().unwrap(),
                n["location"]["file"].as_str().unwrap(),
                n["location"]["line"],
                strings(&n["calls"]),
                strings(&n["callers"])
            );
            if n["source_changed_since_previous_observation"] == true {
                out.push_str("    Source file changed since previous observation (file-level, not function-level).\n");
            }
            if !n["source_facts"].is_null() {
                let f = &n["source_facts"];
                let _ = writeln!(
                    out,
                    "    Source syntax: loops={} branches={} unsafe blocks={} macros={} indirect calls={} (unexpanded, includes nested bodies)",
                    f["loops"],
                    f["branches"],
                    f["unsafe_blocks"],
                    f["macro_invocations"],
                    f["syntactic_indirect_calls"]
                );
            } else {
                out.push_str(
                    "    Source structure unavailable in this snapshot; refresh to collect it.\n",
                );
            }
            for g in n["obligations"].as_array().unwrap() {
                let _ = writeln!(
                    out,
                    "    Goal {}: {}\n      Requirement: {}",
                    g["id"].as_str().unwrap(),
                    g["command_status"].as_str().unwrap(),
                    g["description"].as_str().unwrap()
                );
                if !g["limitation"].is_null() {
                    let _ = writeln!(out, "      Limitation: {}", g["limitation"]);
                }
                if !g["last_evidence"].is_null() {
                    let e = &g["last_evidence"];
                    let _ = writeln!(
                        out,
                        "      Last recorded result (may be stale): {}: {}; receipt {}",
                        e["phase"], e["reason"], e["receipt"]
                    );
                }
            }
            for (key, label) in [
                ("unresolved_calls", "UNRESOLVED calls"),
                ("external_calls", "External boundaries (not proved here)"),
                ("missing_inventory_targets", "MISSING inventory targets"),
            ] {
                if !n[key].as_array().unwrap().is_empty() {
                    let _ = writeln!(out, "    {label}: {}", strings(&n[key]));
                }
            }
            if n["obligations"].as_array().unwrap().is_empty() {
                out.push_str("    No declared obligation/evidence binding. This does not mean a separate proof is mandatory.\n");
            }
        }
    }
    if let Some(p) = v["proof_module_imports"].as_object() {
        out.push_str(
            "\nLean module imports (separate from Rust calls; not theorem-use dependencies):\n",
        );
        if let Some(error) = p.get("error") {
            let _ = writeln!(out, "  Inspection failed: {error}");
        }
        for m in p["modules"].as_array().unwrap() {
            let _ = writeln!(
                out,
                "  {} -> [{}]; imported by [{}]",
                m["name"].as_str().unwrap(),
                strings(&m["imports"]),
                strings(&m["imported_by"])
            );
        }
    } else {
        out.push_str("\nProof module import information unavailable for this backend/snapshot.\n");
    }
    for g in v["standalone_obligations"].as_array().unwrap() {
        let _ = writeln!(
            out,
            "\nStandalone goal {}: {} (no inferred Rust call edges)",
            g["id"], g["command_status"]
        );
    }
    for (key, label) in [
        ("analysis_limitations", "Analysis limitation"),
        ("changed_inputs", "Changed input"),
        (
            "missing_inventory_obligations",
            "Missing inventory obligation",
        ),
    ] {
        for item in v[key].as_array().unwrap() {
            let _ = writeln!(out, "{label}: {}", item.as_str().unwrap());
        }
    }
    let _ = writeln!(out, "\n{}", v["semantics"].as_str().unwrap());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scc_handles_diamond_mutual_and_self_recursion() {
        let graph: Graph = [
            ("A", vec!["B", "C"]),
            ("B", vec!["D"]),
            ("C", vec!["D"]),
            ("D", vec!["E"]),
            ("E", vec!["D"]),
            ("F", vec!["F"]),
        ]
        .into_iter()
        .map(|(k, vs)| (k.into(), vs.into_iter().map(String::from).collect()))
        .collect();
        assert_eq!(
            components(&graph),
            vec![vec!["A"], vec!["B"], vec!["C"], vec!["D", "E"], vec!["F"]]
        );
        assert_eq!(
            reachable(&graph, "A"),
            ["A", "B", "C", "D", "E"]
                .into_iter()
                .map(String::from)
                .collect()
        );
    }
}
