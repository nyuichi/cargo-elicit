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
        "Snapshot {} — saved observation; refresh to inspect edits.",
        one_line(v["snapshot"].as_str().unwrap(), 12)
    );
    if let Some(op) = v.get("operation") {
        let _ = writeln!(
            out,
            "{}: {} passed, {} unresolved ({} requested).",
            op["kind"].as_str().unwrap(),
            op["passed"],
            op["unresolved"],
            op["requested"]
        );
        for g in op["results"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|g| g["accepted"] != true)
            .take(5)
        {
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
        "Remaining: {} / {}; complete: {} (fresh audit + semantic review). Selected obligations only.",
        v["summary"]["remaining"], v["summary"]["total"], v["summary"]["completed"]
    );
    let goals = v["goals"].as_array().unwrap();
    let limit = if v["filter"] == "all" { usize::MAX } else { 10 };
    for g in goals.iter().take(limit) {
        let _ = writeln!(
            out,
            "  {}: check={} audit={} review={}",
            g["id"].as_str().unwrap(),
            g["check"].as_str().unwrap(),
            g["audit"].as_str().unwrap(),
            g["review"].as_str().unwrap()
        );
        if g["audit_validity"]["state"] == "stale" {
            let changes = g["audit_validity"]["input_changes"].as_array().unwrap();
            if let Some(c) = changes.first() {
                let _ = writeln!(
                    out,
                    "    Audit stale: {} {} ({} changed inputs)",
                    c["change"].as_str().unwrap(),
                    one_line(c["path"].as_str().unwrap(), 100),
                    changes.len()
                );
            } else {
                out.push_str("    Audit stale; inspect show ID / --verbose for binding details.\n");
            }
        }
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
            let _ = writeln!(out, "  … {} more in plan.", candidates.len() - 5);
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
    out.push_str("Details: show ID; plan --details; --verbose or --json. No need to run status again after this summary.\n");
    out
}
pub fn plan(v: &Value) -> String {
    let mut out = String::from(
        "Dependency plan — saved observation; refresh to inspect edits.\nCallee-first SCC layers; structural candidates, not proof prerequisites.\n",
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
    out.push_str("Accepted contracts may be too weak for callers. Details/accepted dependencies/contracts: plan [API ...] --details; --json retains full saved facts.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipping_is_unicode_safe_and_removes_multiline_noise() {
        assert_eq!(one_line("日本語\n  契約", 3), "日本語…");
        assert_eq!(one_line("a \n b", 20), "a b");
    }
}
