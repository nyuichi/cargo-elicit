//! Bounded summaries for interactive/AI use; full saved facts remain available.
use serde_json::Value;
use std::fmt::Write;

pub fn one_line(text: &str, limit: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}
pub fn status(v: &Value) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "Snapshot {} — saved observation; status to inspect edits.",
        one_line(v["snapshot"].as_str().unwrap(), 12)
    );
    if let Some(op) = v.get("operation") {
        let _ = writeln!(
            out,
            "{}: {} passed, {} unresolved ({} requested).",
            if op["fresh"] == true {
                "verify --fresh"
            } else {
                op["kind"].as_str().unwrap()
            },
            op["passed"],
            op["unresolved"],
            op["requested"]
        );
        if op["reused_count"].as_u64().is_some_and(|n| n > 0) {
            let _ = writeln!(
                out,
                "Existing evidence reused: {} / {} requested.",
                op["reused_count"], op["requested"]
            );
        }
        if op["reused_evidence"] == true {
            out.push_str("Saved command result reused; no command executed. Use verify --fresh to run it again.\n");
        }
        let mut seen = std::collections::BTreeSet::new();
        for g in op["results"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|g| g["accepted"] != true)
            .take(5)
        {
            let d = &g["diagnostic"];
            if let Some(log) = d["log"].as_str() {
                let cache = d["cache"].as_str().unwrap_or("");
                let key = format!("{cache}/{log}");
                if seen.insert((key.clone(), g["reason"].as_str().unwrap_or("").to_owned())) {
                    if let Some(reason) = g["reason"].as_str() {
                        let _ = writeln!(out, "  {}", one_line(reason, 180));
                    }
                    let _ = writeln!(
                        out,
                        "  Failure [{}]: {}",
                        g["phase"].as_str().unwrap_or("unknown"),
                        one_line(
                            d["message"]
                                .as_str()
                                .unwrap_or("no diagnostic output; inspect log"),
                            300
                        )
                    );
                    let _ = writeln!(
                        out,
                        "    Log: {key} (shared failures shown once; per-goal outcomes in --json)"
                    );
                }
                continue;
            }
            let _ = writeln!(
                out,
                "  {}: {}",
                g["id"].as_str().unwrap(),
                one_line(g["reason"].as_str().unwrap_or("inspect details"), 140)
            );
        }
    }
    let _ = writeln!(
        out,
        "Remaining: {} / {}; complete: {} (native command results). Selected goals only.",
        v["summary"]["remaining"], v["summary"]["total"], v["summary"]["completed"]
    );
    let goals = v["goals"].as_array().unwrap();
    let limit = if v["filter"] == "all" { usize::MAX } else { 10 };
    for g in goals.iter().take(limit) {
        let _ = writeln!(
            out,
            "  {}: {}",
            g["id"].as_str().unwrap(),
            g["command_status"].as_str().unwrap()
        );
    }
    if goals.len() > limit {
        let _ = writeln!(
            out,
            "  … {} more; use --all or --verbose.",
            goals.len() - limit
        );
    }
    let candidates = v["candidate_sccs"].as_array().unwrap();
    if !candidates.is_empty() {
        out.push_str("Candidates (known call dependencies; inspect boundaries):\n");
        for c in candidates.iter().take(5) {
            let _ = writeln!(
                out,
                "  {} — {} selected roots{}",
                c["label"].as_str().unwrap(),
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
    let issues = v["analysis_limitations"].as_array().unwrap();
    if !issues.is_empty() {
        let _ = writeln!(
            out,
            "Analysis limitations: {}. {}",
            issues.len(),
            one_line(issues[0].as_str().unwrap(), 120)
        );
    }
    out.push_str("Details: status ID; status --plan --details; --verbose or --json. No need to run status again after this summary.\n");
    out
}
pub fn plan(v: &Value) -> String {
    let mut out = String::from(
        "Dependency plan — observed snapshot; --saved reads without re-observation.\nCallee-first SCC layers; structural candidates, not proof prerequisites.\n",
    );
    let rows = v["components"].as_array().unwrap();
    let open: Vec<_> = rows
        .iter()
        .filter(|r| r["all_members_have_accepted_evidence"] != true)
        .collect();
    let _ = writeln!(
        out,
        "{} roots; {} SCCs; {} with accepted evidence; {} open.",
        v["selected_roots"].as_array().unwrap().len(),
        rows.len(),
        rows.len() - open.len(),
        open.len()
    );
    for r in open.iter().take(10) {
        let n = &r["nodes"][0];
        let _ = writeln!(
            out,
            "  L{} {} [{}] — {} members; {} roots; {} open callee SCCs{}",
            r["layer"],
            r["id"].as_str().unwrap(),
            if r["candidate"] == true {
                "candidate"
            } else {
                "dependencies open"
            },
            r["members"].as_array().unwrap().len(),
            r["selected_root_count"],
            r["pending_dependency_sccs"].as_array().unwrap().len(),
            if r["recursive"] == true {
                "; recursive"
            } else {
                ""
            }
        );
        let _ = writeln!(
            out,
            "    {}: {}; external calls={}; unresolved calls={}",
            n["id"].as_str().unwrap(),
            n["evidence_state"].as_str().unwrap(),
            r["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n["external_calls"].as_array().unwrap().len())
                .sum::<usize>(),
            r["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n["unresolved_calls"].as_array().unwrap().len())
                .sum::<usize>()
        );
    }
    if open.len() > 10 {
        let _ = writeln!(out, "  … {} more open SCCs in --details.", open.len() - 10);
    }
    let _ = writeln!(
        out,
        "Standalone goals: {}; missing inventory bindings: {}; analysis limitations: {}.",
        v["standalone_obligations"].as_array().unwrap().len(),
        v["missing_inventory_obligations"].as_array().unwrap().len(),
        v["analysis_limitations"].as_array().unwrap().len()
    );
    out.push_str("Command success does not establish API coverage. Details/dependencies: status [API ...] --plan --details; --json retains full saved facts.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_failure_has_one_actionable_diagnostic_and_durable_log() {
        let failure = serde_json::json!({"id":"api", "accepted":false, "phase":"proofs",
            "diagnostic":{"cache":".elicit/evidence/check-1", "log":"logs/Codec.log",
                "message":"Codec.lean:18:2: error: recursion limit"}});
        let view = serde_json::json!({"snapshot":"abc", "operation":{"kind":"check",
            "passed":0,"unresolved":2,"requested":2,"results":[failure.clone(),failure]},
            "summary":{"remaining":2,"total":2,"completed":0},"goals":[],
            "candidate_sccs":[],"analysis_limitations":[]});
        let text = status(&view);
        assert_eq!(text.matches("Codec.lean:18:2: error:").count(), 1);
        assert!(text.contains(".elicit/evidence/check-1/logs/Codec.log"));
        assert!(text.contains("Failure [proofs]"));
        assert!(text.contains("2 unresolved"));
    }
    #[test]
    fn older_receipt_diagnostics_remain_readable() {
        let diagnostic: crate::model::Diagnostic = serde_json::from_value(
            serde_json::json!({"phase":"proofs","cache":".elicit/evidence/check-old"}),
        )
        .unwrap();
        assert!(diagnostic.message.is_none());
        assert!(diagnostic.log.is_none());
    }
    #[test]
    fn clipping_is_unicode_safe_and_removes_multiline_noise() {
        assert_eq!(one_line("日本語\n  契約", 3), "日本語…");
        assert_eq!(one_line("a \n b", 20), "a b");
    }
}
