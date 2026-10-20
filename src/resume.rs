//! Recovery facts from bound evidence.
use crate::{project, store::Store, ui};
use anyhow::Result;
use serde_json::{Value, json};
use std::fmt::Write;

pub fn complete(g: &Value) -> bool {
    g["command_status"] == "succeeded"
}
pub fn view(store: &Store, s: &project::Snapshot, refreshed: bool) -> Result<Value> {
    let mut v = project::view(s);
    v["observation"] = json!(if refreshed {
        "refreshed_inputs_without_proof_execution"
    } else {
        "saved_snapshot"
    });
    v["workspace_currentness"] = json!(if refreshed {
        "observed_at_snapshot; not a new proof or a guarantee against later edits"
    } else {
        "unknown; --saved did not inspect verification inputs"
    });
    for g in v["goals"].as_array_mut().unwrap() {
        if !refreshed {
            g["source_status"] = json!("unknown");
            g["environment_status"] = json!("unknown");
        }
        let goal = s.goals.iter().find(|x| x.id == g["id"]).unwrap();
        let t = goal
            .target
            .as_ref()
            .and_then(|id| s.inventory.iter().find(|t| &t.id == id));
        g["source"] = json!(t.map(|t| json!({"file":t.file,"line":t.line})));
        g["statement"] = json!(goal.statement);
        g["theorem"] = json!(goal.theorem);
        g["last_evidence_current"] = json!(
            s.evidence
                .get(&goal.id)
                .and_then(|es| es.last())
                .is_some_and(|e| e.input == s.input && e.specification == goal.specification)
        );
        let last = s.evidence.get(&goal.id).and_then(|es| es.last());
        g["log_available"] = json!(
            last.and_then(|e| e.result.as_ref())
                .and_then(|r| r.diagnostic.as_ref())
                .and_then(|d| d.log.as_ref().map(|log| store
                    .root
                    .join(&d.cache)
                    .join(log)
                    .is_file()))
        );
        let changes: Vec<_> = last
            .filter(|e| !e.input_files.is_empty())
            .into_iter()
            .flat_map(|e| {
                e.input_files
                    .keys()
                    .chain(s.input_files.keys())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .filter(|k| e.input_files.get(*k) != s.input_files.get(*k))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();
        g["changes_since_last_evidence"] = json!(changes);
        g["change_details_available"] = json!(last.is_some_and(|e| !e.input_files.is_empty()));
        g["next"] = json!(if goal.limitation.is_some() {
            "inspect the declared limitation"
        } else if g["source_status"] == "unchanged" && g["environment_status"] == "different" {
            "past result preserved; verify in the current environment"
        } else if !g["last_evidence_current"].as_bool().unwrap() && last.is_some() {
            "inspect changed inputs/specification; recheck at a useful milestone"
        } else if last.is_some_and(|e| !e.accepted) {
            "inspect last failure and linked log; iterate locally"
        } else if g["command_status"] != "succeeded" {
            "work with native tools; verify at useful milestones"
        } else {
            "complete at observed snapshot"
        });
    }
    let goals = v["goals"].as_array().unwrap();
    let done = goals.iter().filter(|g| complete(g)).count();
    v["summary"] = json!({"total":goals.len(),"completed":done,"remaining":goals.len()-done,
        "hidden_completed":0});
    let plan = crate::plan::build(s, &[])?;
    v["candidate_sccs"] = json!(plan["components"].as_array().unwrap().iter().filter(|c|c["candidate"]==true)
        .map(|c|json!({"id":c["id"],"selected_root_count":c["selected_root_count"],"call_resolution_incomplete":c["call_resolution_incomplete"]})).collect::<Vec<_>>());
    v["recent_history"] = json!(store.recent_history()?);
    v["proof_modules"] = json!(s.proof_modules);
    v["dependency_plan_hint"] = json!(
        "status [API ...] --plan inspects structural SCC candidates, not logical proof prerequisites"
    );
    Ok(v)
}
pub fn render(v: &Value) -> String {
    let mut out = format!(
        "Status — {}\n{}\nObserved at: {} ms; snapshot {}\nSelected goals only; no proof executed.\n",
        v["observation"].as_str().unwrap(),
        v["workspace_currentness"].as_str().unwrap(),
        v["observed_at_unix_ms"],
        ui::one_line(v["snapshot"].as_str().unwrap(), 12)
    );
    let goals = v["goals"].as_array().unwrap();
    let summary = &v["summary"];
    let _ = writeln!(
        out,
        "Complete: {}/{}; remaining: {} (native command results at snapshot).",
        summary["completed"], summary["total"], summary["remaining"]
    );
    let issues = v["analysis_limitations"].as_array().unwrap();
    if let Some(issue) = issues.first() {
        let _ = writeln!(
            out,
            "Analysis limitations: {}. {}",
            issues.len(),
            ui::one_line(issue.as_str().unwrap_or("inspect JSON"), 160)
        );
    }
    let pending: Vec<_> = goals
        .iter()
        .filter(|g| v["filter"] == "all" || !complete(g))
        .collect();
    let limit = if v["filter"] == "all" { usize::MAX } else { 10 };
    for g in pending.iter().take(limit) {
        let _ = writeln!(
            out,
            "{}: {}",
            g["id"].as_str().unwrap(),
            g["command_status"].as_str().unwrap()
        );
        let _ = writeln!(
            out,
            "  History: {}; inputs: {}; environment: {}",
            g["historical_result"].as_str().unwrap_or("none"),
            g["source_status"].as_str().unwrap_or("unknown"),
            g["environment_status"].as_str().unwrap_or("unknown")
        );
        if let Some(file) = g["source"]["file"].as_str() {
            let _ = writeln!(out, "  Rust: {file}:{}", g["source"]["line"]);
        }
        for (label, value) in [
            ("Theorem (user-declared)", &g["theorem"]),
            ("Statement", &g["statement"]),
            ("Limitation", &g["limitation"]),
        ] {
            if let Some(s) = value.as_str() {
                let _ = writeln!(out, "  {label}: {}", ui::one_line(s, 240));
            }
        }
        if let Some(command) = g["command"].as_array() {
            let _ = writeln!(
                out,
                "  Command: {} (directory: {})",
                serde_json::to_string(command).unwrap(),
                g["directory"].as_str().unwrap_or(".")
            );
        }
        if let Some(e) = g["last_evidence"].as_object() {
            let _ = writeln!(
                out,
                "  Last evidence (current={}): {}",
                g["last_evidence_current"],
                ui::one_line(e["reason"].as_str().unwrap_or("inspect receipt"), 240)
            );
            let _ = writeln!(out, "  Receipt: {}", e["receipt"].as_str().unwrap_or(""));
            let d = &e["result"]["diagnostic"];
            if let Some(message) = d["message"].as_str() {
                let _ = writeln!(out, "  Diagnostic: {}", ui::one_line(message, 300));
            }
            if let Some(log) = d["log"].as_str() {
                let _ = writeln!(
                    out,
                    "  Log{}: {}/{log}",
                    if g["log_available"] == false {
                        " (not available in this workspace)"
                    } else {
                        ""
                    },
                    d["cache"].as_str().unwrap_or("")
                );
            }
        }
        let changes = g["changes_since_last_evidence"].as_array().unwrap();
        for path in changes.iter().take(5) {
            let _ = writeln!(out, "  Changed since evidence: {}", path.as_str().unwrap());
        }
        if changes.len() > 5 {
            let _ = writeln!(
                out,
                "  … {} more changed inputs in --json.",
                changes.len() - 5
            );
        }
        let _ = writeln!(out, "  Next: {}", g["next"].as_str().unwrap());
    }
    if pending.len() > limit {
        let _ = writeln!(
            out,
            "… {} more unfinished goals in --json.",
            pending.len() - limit
        );
    }
    let candidates = v["candidate_sccs"].as_array().unwrap();
    if !candidates.is_empty() {
        out.push_str("Candidates (known call dependencies, not proof prerequisites):\n");
        for c in candidates.iter().take(5) {
            let _ = writeln!(
                out,
                "  {} — {} selected roots{}",
                c["id"].as_str().unwrap(),
                c["selected_root_count"],
                if c["call_resolution_incomplete"] == true {
                    "; unresolved calls"
                } else {
                    ""
                }
            );
        }
        if candidates.len() > 5 {
            let _ = writeln!(out, "  … {} more in status --plan.", candidates.len() - 5);
        }
    }
    if let Some(history) = v["recent_history"].as_array().filter(|h| !h.is_empty()) {
        out.push_str("Recent recorded milestones (not additional proof evidence):\n");
        for record in history.iter().rev().take(3) {
            let _ = writeln!(
                out,
                "  {} {}",
                record["recorded_at"],
                record["event"].as_str().unwrap_or("unknown")
            );
            if let Some(reason) = record["data"]["reason"].as_str() {
                let _ = writeln!(out, "    {}", ui::one_line(reason, 240));
            }
        }
        out.push_str("  Details: status --saved --json; durable log: .elicit/history.jsonl\n");
    }
    out.push_str("Details: status --saved --json; status ID; status [API ...] --plan. Call edges do not establish proofs.\n");
    out
}
