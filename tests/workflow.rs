use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
    thread,
    time::{Duration, Instant},
};
struct Project {
    dir: tempfile::TempDir,
}
impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir(root.join("bin")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname=\"fixture\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[workspace]\n",
        )
        .unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn identity(x:u32)->u32 { x }\n",
        )
        .unwrap();
        for (name, source) in [
            ("cargo", "cargo.py"),
            ("charon", "aeneas.py"),
            ("aeneas", "aeneas.py"),
            ("lake", "aeneas.py"),
        ] {
            let path = root.join("bin").join(name);
            fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/support")
                    .join(source),
                &path,
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let backend = root.join(".elicit/runtime/aeneas/backends/lean");
        fs::create_dir_all(backend.join(".lake/build/lib/lean")).unwrap();
        fs::write(backend.join("lakefile.lean"), "").unwrap();
        fs::write(backend.join("lake-manifest.json"), "{\"packages\":[]}").unwrap();
        fs::write(backend.join(".lake/build/lib/lean/Aeneas.olean"), "").unwrap();
        let p = Self { dir };
        p.ok(&["init", "--api", "identity"]);
        p
    }
    fn cmd(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_cargo-elicit"));
        c.current_dir(self.dir.path())
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    self.dir.path().join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("ELICIT_REAL_CARGO", env!("CARGO"))
            .env(
                "AENEAS_HOME",
                self.dir.path().join(".elicit/runtime/aeneas"),
            );
        c
    }
    fn call(&self, args: &[&str]) -> Output {
        self.cmd().args(args).output().unwrap()
    }
    fn ok(&self, args: &[&str]) -> Output {
        let o = self.call(args);
        assert!(
            o.status.success(),
            "{args:?}: {}\n{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        o
    }
    fn snapshot(&self) -> Value {
        serde_json::from_slice(&fs::read(self.dir.path().join(".elicit/snapshot.json")).unwrap())
            .unwrap()
    }
    fn proof(&self) {
        let root = self.dir.path().join("elicit-proofs/aeneas");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("contracts.toml"),"schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x out => out = x\"\ntheorem=\"Proofs.identity_spec\"\n").unwrap();
        fs::write(root.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    }
    fn stages(&self) -> Vec<Value> {
        fs::read_to_string(self.dir.path().join(".elicit/events.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .filter(|v| v["event"] == "pipeline_stage_finished")
            .map(|v| v["data"].clone())
            .collect()
    }
}
#[test]
fn observation_is_pure_even_with_missing_tools_and_changed_workspace() {
    let p = Project::new();
    let saved = fs::read(p.dir.path().join(".elicit/snapshot.json")).unwrap();
    assert!(
        !p.dir.path().join(".elicit/events.jsonl").exists(),
        "refresh must not compile verification inputs"
    );
    fs::write(
        p.dir.path().join("src/lib.rs"),
        "pub fn identity(x:u32)->u32 { x + 1 }\n",
    )
    .unwrap();
    for args in [
        vec!["status", "--json"],
        vec!["list", "--inventory", "--json"],
        vec!["show", "identity", "--json"],
        vec!["plan", "identity", "--json"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_cargo-elicit"))
            .current_dir(p.dir.path())
            .env("PATH", "/nonexistent")
            .env("AENEAS_HOME", "/nonexistent")
            .args(&args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["observation"], "saved_snapshot");
        assert!(
            v["workspace_currentness"]
                .as_str()
                .unwrap()
                .contains("not_observed")
        );
    }
    assert_eq!(
        saved,
        fs::read(p.dir.path().join(".elicit/snapshot.json")).unwrap()
    );
    let previous = p.snapshot()["input"].clone();
    p.ok(&["refresh"]);
    assert_ne!(p.snapshot()["input"], previous);
    assert!(
        p.snapshot()["changed_inputs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().ends_with("src/lib.rs"))
    );
    assert!(!p.dir.path().join(".elicit/events.jsonl").exists());
    assert_eq!(
        p.call(&["run"]).status.code(),
        Some(2),
        "AI orchestration was removed"
    );
}
#[test]
fn incremental_check_reuses_dependencies_and_audit_is_independent() {
    let p = Project::new();
    p.proof();
    let root = p.dir.path().join("elicit-proofs/aeneas");
    fs::write(root.join("Peer.lean"), "import ElicitContracts\n").unwrap();
    fs::write(root.join("Facts.lean"), "import ElicitContracts\n").unwrap();
    fs::write(root.join("Proofs.lean"), "import\tPeer\tFacts\n").unwrap();
    p.ok(&["check"]);
    let count = p.stages().len();
    p.ok(&["check"]);
    let stages = p.stages();
    let second = &stages[count..];
    assert!(
        second
            .iter()
            .any(|v| v["stage"] == "Facts.lean" && v["cache_hit"] == true)
    );
    assert!(
        second
            .iter()
            .any(|v| v["stage"] == "Audit.lean" && v["cache_hit"] == false)
    );
    let count = stages.len();
    fs::write(
        root.join("Facts.lean"),
        "import ElicitContracts\n-- changed supporting proof\n",
    )
    .unwrap();
    p.ok(&["check"]);
    let stages = p.stages();
    let changed = &stages[count..];
    assert!(
        changed
            .iter()
            .any(|v| v["stage"] == "Peer.lean" && v["cache_hit"] == true)
    );
    assert!(
        changed
            .iter()
            .any(|v| v["stage"] == "Facts.lean" && v["cache_hit"] == false)
    );
    assert!(
        changed
            .iter()
            .any(|v| v["stage"] == "Proofs.lean" && v["cache_hit"] == false)
    );
    let count = stages.len();
    p.ok(&["audit"]);
    assert!(p.stages()[count..].iter().all(|v| v["cache_hit"] == false));
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["declared_obligations_audited_at_snapshot"], true);
    assert_eq!(status["goals"][0]["review"], "unreviewed");
    // Checked objects cannot be used after corruption, even with unchanged sources.
    fs::write(
        p.dir.path().join(".elicit/cache/check/Facts.olean"),
        "corrupt",
    )
    .unwrap();
    let count = p.stages().len();
    p.ok(&["check"]);
    assert!(
        p.stages()[count..]
            .iter()
            .any(|v| v["stage"] == "Facts.lean" && v["cache_hit"] == false)
    );
    // Persisted logs and receipts survive deletion of all regenerable products.
    fs::remove_dir_all(p.dir.path().join(".elicit/cache")).unwrap();
    let receipt = p.snapshot()["evidence"]["identity"][0]["receipt"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(p.dir.path().join(&receipt).exists());
    p.ok(&["status"]);
}
#[test]
fn prose_review_and_machine_evidence_are_independent() {
    let p = Project::new();
    p.proof();
    p.ok(&["audit"]);
    p.ok(&[
        "review",
        "--note",
        "The postcondition matches the desired identity behavior",
    ]);
    let input = p.snapshot()["input"].clone();
    let file = p.dir.path().join("elicit.toml");
    fs::write(
        &file,
        fs::read_to_string(&file).unwrap().replace(
            "description = \"\"",
            "description = \"Returns its argument\"",
        ),
    )
    .unwrap();
    p.ok(&["refresh"]);
    assert_eq!(
        p.snapshot()["input"],
        input,
        "editorial intent must not invalidate machine proofs"
    );
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["goals"][0]["audit"], "accepted");
    assert_eq!(status["goals"][0]["review"], "stale");
    p.ok(&[
        "review",
        "--verdict",
        "needs-work",
        "--note",
        "Need a clearer human interpretation",
    ]);
    p.ok(&["check"]);
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["goals"][0]["review"], "needs_work");
    assert_eq!(status["goals"][0]["audit"], "accepted");
}
#[test]
fn extra_axioms_and_unsupported_imports_never_accept() {
    let p = Project::new();
    p.proof();
    let out = p
        .cmd()
        .env("MOCK_AENEAS", "sorryAx")
        .args(["audit"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["goals"][0]["audit"], "failed");
    let root = p.dir.path().join("elicit-proofs/aeneas");
    fs::write(root.join("Support.lean"), "import ElicitContracts\n").unwrap();
    fs::write(root.join("Proofs.lean"), "import Support\n").unwrap();
    p.ok(&["check"]);
    fs::remove_file(root.join("Support.lean")).unwrap();
    fs::write(root.join("Proofs.lean"), "meta import Support\n").unwrap();
    assert!(
        !p.call(&["check"]).status.success(),
        "unsupported headers cannot load stale cached dependencies"
    );
}
#[test]
fn proposition_obligations_and_semantic_changes_during_check() {
    let p = Project::new();
    let root = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    let member = p.dir.path().join("member");
    fs::create_dir(&member).unwrap();
    fs::rename(
        p.dir.path().join("elicit-proofs"),
        member.join("elicit-proofs"),
    )
    .unwrap();
    fs::rename(p.dir.path().join("src"), member.join("src")).unwrap();
    fs::rename(p.dir.path().join("Cargo.toml"), member.join("Cargo.toml")).unwrap();
    fs::write(
        member.join("Cargo.toml"),
        "[package]\nname=\"fixture\"\nversion=\"0.0.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(
        p.dir.path().join("Cargo.toml"),
        "[workspace]\nmembers=[\"member\"]\nresolver=\"2\"\n",
    )
    .unwrap();
    fs::create_dir_all(member.join(".elicit/runtime")).unwrap();
    let config = p.dir.path().join("elicit.toml");
    let original = fs::read_to_string(&config).unwrap();
    let head = original.split("[[obligations]]").next().unwrap();
    fs::write(
        &config,
        format!(
            "{head}\n[[obligations]]\nid=\"fact\"\nstatement=\"True\"\ntheorem=\"Proofs.fact\"\n"
        ),
    )
    .unwrap();
    let result = p.call(&["audit"]);
    if !result.status.success() {
        for entry in walkdir::WalkDir::new(p.dir.path().join(".elicit/evidence"))
            .into_iter()
            .flatten()
        {
            if entry.path().extension().is_some_and(|v| v == "log") {
                eprintln!(
                    "{}: {}",
                    entry.path().display(),
                    fs::read_to_string(entry.path()).unwrap_or_default()
                );
            }
        }
    }
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["declared_obligations_audited_at_snapshot"], true);
    assert_eq!(status["whole_package_claim"], false);
    let original_input = p.snapshot()["input"].clone();
    fs::write(
        &config,
        fs::read_to_string(&config).unwrap().replace(
            "id=\"fact\"",
            "id=\"fact\"\ndescription=\"Editorial change\"",
        ),
    )
    .unwrap();
    p.ok(&["refresh"]);
    assert_eq!(
        p.snapshot()["input"],
        original_input,
        "workspace-root descriptions do not invalidate machine proofs"
    );
    let mut child = p
        .cmd()
        .env("MOCK_AENEAS", "wait")
        .args(["check", "--seconds", "3"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let end = Instant::now() + Duration::from_secs(30);
    while !p
        .dir
        .path()
        .join("member/.elicit/runtime/mock-checker-started")
        .exists()
    {
        assert!(Instant::now() < end);
        thread::sleep(Duration::from_millis(20));
    }
    fs::write(
        &config,
        fs::read_to_string(&config)
            .unwrap()
            .replace("statement=\"True\"", "statement=\"False\""),
    )
    .unwrap();
    assert!(!child.wait().unwrap().success());
    let snapshot = p.snapshot();
    let last = snapshot["evidence"]["fact"]
        .as_array()
        .unwrap()
        .last()
        .unwrap();
    assert_eq!(last["accepted"], false);
    p.ok(&["refresh"]);
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["declared_obligations_audited_at_snapshot"], false);
}

#[test]
fn dependency_plan_combines_scc_sharing_evidence_and_uncertainty_without_tools() {
    let p = Project::new();
    let mut snapshot = p.snapshot();
    let base = snapshot["inventory"][0].clone();
    let mut targets = vec![];
    for (id, calls) in [
        ("A", vec!["B", "C"]),
        ("X", vec!["D"]),
        ("B", vec!["D"]),
        ("C", vec![]),
        ("D", vec!["E"]),
        ("E", vec!["D"]),
        ("Outside", vec!["D"]),
        ("Unrelated", vec![]),
    ] {
        let mut t = base.clone();
        t["id"] = id.into();
        t["name"] = id.into();
        t["aliases"] = serde_json::json!([]);
        t["calls"] = serde_json::json!(calls);
        if id == "D" {
            t["unresolved_calls"] = serde_json::json!(["src/lib.rs:3:4"]);
            t["external_calls"] = serde_json::json!(["external.rs:42"]);
        }
        targets.push(t);
    }
    snapshot["inventory"] = targets.into();
    let goal = snapshot["goals"][0].clone();
    snapshot["goals"] = ["A", "X", "C"]
        .into_iter()
        .map(|id| {
            let mut g = goal.clone();
            g["id"] = id.into();
            g["target"] = id.into();
            g
        })
        .collect::<Vec<_>>()
        .into();
    snapshot["evidence"] = serde_json::json!({"C":[{
        "kind":"check","recorded_at":1,"input":snapshot["input"],
        "specification":goal["specification"],"accepted":true,"result":null,
        "receipt":".elicit/evidence/c.json","phase":"check","reason":"accepted"
    }]});
    let path = p.dir.path().join(".elicit/snapshot.json");
    fs::write(&path, serde_json::to_vec_pretty(&snapshot).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    let out = p
        .cmd()
        .env("PATH", "/nonexistent")
        .args(["plan", "A", "X", "--json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let plan: Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = plan["components"].as_array().unwrap();
    assert_eq!(rows.len(), 5);
    let row = |id: &str| rows.iter().find(|r| r["id"] == id).unwrap();
    assert_eq!(row("D")["members"], serde_json::json!(["D", "E"]));
    assert_eq!(row("D")["recursive"], true);
    assert_eq!(row("D")["selected_root_count"], 2);
    assert_eq!(row("D")["candidate"], true);
    assert_eq!(row("D")["call_resolution_incomplete"], true);
    assert_eq!(
        row("D")["nodes"][0]["callers"],
        serde_json::json!(["B", "E", "Outside", "X"])
    );
    assert_eq!(row("C")["all_members_have_accepted_evidence"], true);
    assert_eq!(row("C")["candidate"], false);
    assert_eq!(
        row("A")["pending_dependency_sccs"],
        serde_json::json!(["B"])
    );
    assert_eq!(row("A")["layer"], 2);
    assert_eq!(row("D")["layer"], 0);
    assert_eq!(fs::read(&path).unwrap(), before);
    let text = String::from_utf8(p.ok(&["plan", "A", "X"]).stdout).unwrap();
    assert!(text.contains("UNRESOLVED calls"));
    assert!(text.contains("External boundaries"));
    assert!(text.contains("No declared obligation"));
    assert!(text.contains("not a proof dependency graph"));
    assert!(!p.call(&["plan", "missing"]).status.success());
    // A contract edit invalidates the saved evidence binding without hiding history.
    snapshot["goals"][2]["specification"] = "changed-contract".into();
    fs::write(&path, serde_json::to_vec(&snapshot).unwrap()).unwrap();
    let plan: Value = serde_json::from_slice(&p.ok(&["plan", "C", "--json"]).stdout).unwrap();
    assert_eq!(
        plan["components"][0]["all_members_have_accepted_evidence"],
        false
    );
    assert_eq!(
        plan["components"][0]["nodes"][0]["obligations"][0]["check"],
        "missing_or_stale"
    );
}

#[test]
fn plan_collects_native_contracts_source_structure_and_module_imports_on_refresh() {
    let p = Project::new();
    p.proof();
    fs::write(p.dir.path().join("src/lib.rs"),
        "pub fn identity(x:u32)->u32 { let mut y=x; while y>0 { if y==1 { y=0; } else { y-=1; } } x }\n").unwrap();
    p.ok(&["refresh"]);
    let plan: Value =
        serde_json::from_slice(&p.ok(&["plan", "identity", "--json"]).stdout).unwrap();
    let node = &plan["components"][0]["nodes"][0];
    assert_eq!(node["source_facts"]["loops"], 1);
    assert_eq!(node["source_facts"]["branches"], 1);
    assert_eq!(node["native_contract"]["ensures"], "fun x out => out = x");
    assert_eq!(node["native_contract"]["theorem"], "Proofs.identity_spec");
    assert_eq!(plan["proof_module_imports"]["modules"][0]["name"], "Proofs");
    assert_eq!(
        plan["proof_module_imports"]["modules"][0]["imports"],
        serde_json::json!(["ElicitContracts"])
    );
}

#[test]
fn status_focus_hides_only_audited_reviewed_goals_and_restores_stale_items() {
    let p = Project::new();
    let mut s = p.snapshot();
    let id = s["goals"][0]["id"].as_str().unwrap().to_owned();
    s["evidence"] = serde_json::json!({id.clone():[{
        "kind":"audit","recorded_at":1,"input":s["input"],
        "specification":s["goals"][0]["specification"],"accepted":true,
        "result":null,"receipt":".elicit/evidence/a.json","phase":"audit","reason":"accepted"
    }]});
    let path = p.dir.path().join(".elicit/snapshot.json");
    fs::write(&path, serde_json::to_vec(&s).unwrap()).unwrap();
    // A kernel result alone is visible until the specification is reviewed.
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["goals"].as_array().unwrap().len(), 1);
    s["reviews"] = serde_json::json!({id.clone():{
        "specification":s["goals"][0]["review_specification"],"verdict":"accepted","note":"Matches intended property","recorded_at":1
    }});
    fs::write(&path, serde_json::to_vec(&s).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert!(status["goals"].as_array().unwrap().is_empty());
    assert_eq!(status["summary"]["hidden_completed"], 1);
    assert_eq!(status["summary"]["remaining"], 0);
    let all: Value = serde_json::from_slice(&p.ok(&["status", "--all", "--json"]).stdout).unwrap();
    assert_eq!(all["goals"].as_array().unwrap().len(), 1);
    assert_eq!(all["summary"]["hidden_completed"], 0);
    assert_eq!(fs::read(&path).unwrap(), before);
    let plan: Value = serde_json::from_slice(&p.ok(&["plan", "--json"]).stdout).unwrap();
    assert_eq!(plan["components"].as_array().unwrap().len(), 1);
    // A refreshed observation with a changed input makes the API visible again.
    s["input"] = "changed-input".into();
    fs::write(&path, serde_json::to_vec(&s).unwrap()).unwrap();
    let status: Value = serde_json::from_slice(&p.ok(&["status", "--json"]).stdout).unwrap();
    assert_eq!(status["summary"]["remaining"], 1);
    assert_eq!(status["goals"][0]["audit"], "missing_or_stale");
}

#[test]
fn notes_preserve_audit_and_cache_while_tracked_changes_explain_staleness() {
    let p = Project::new();
    p.proof();
    p.ok(&["check"]);
    p.ok(&["audit"]);
    let binding = p.snapshot()["input"].clone();
    let receipt = p.snapshot()["evidence"]["identity"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()["receipt"]
        .clone();
    let output: Value = serde_json::from_slice(&p.ok(&["notes", "--json"]).stdout).unwrap();
    assert_eq!(output["excluded_from_verification_inputs"], true);
    let notes = p.dir.path().join(".elicit/notes");
    fs::write(notes.join("PROGRESS.md"), "Progress changed\n").unwrap();
    fs::write(notes.join("FINAL_REPORT.md"), "Final report changed\n").unwrap();
    p.ok(&["refresh"]);
    assert_eq!(p.snapshot()["input"], binding);
    let status: Value =
        serde_json::from_slice(&p.ok(&["status", "--all", "--json"]).stdout).unwrap();
    assert_eq!(
        status["goals"][0]["audit_validity"]["state"],
        "valid_at_snapshot"
    );
    assert_eq!(status["goals"][0]["audit_validity"]["receipt"], receipt);
    assert_eq!(
        status["goals"][0]["audit_validity"]["fresh_audit_needed_at_snapshot"],
        false
    );
    let count = p.stages().len();
    p.ok(&["check"]);
    assert!(p.stages()[count..].iter().any(|s| s["cache_hit"] == true));
    // Arbitrary Markdown stays tracked; no extension or benchmark-filename whitelist.
    fs::write(
        p.dir.path().join("FINAL_REPORT.md"),
        "Tracked build input\n",
    )
    .unwrap();
    p.ok(&["refresh"]);
    p.ok(&["refresh"]); // Reasons must compare with the audit, not only the last refresh.
    let status: Value =
        serde_json::from_slice(&p.ok(&["status", "--all", "--json"]).stdout).unwrap();
    let detail = &status["goals"][0]["audit_validity"];
    assert_eq!(detail["state"], "stale");
    assert!(detail["input_changes"].as_array().unwrap().iter().any(|c| {
        c["path"].as_str().unwrap().ends_with("/FINAL_REPORT.md") && c["change"] == "added"
    }));
    fs::remove_file(p.dir.path().join("FINAL_REPORT.md")).unwrap();
    let file = p.dir.path().join("elicit-proofs/aeneas/contracts.toml");
    let source = fs::read_to_string(&file)
        .unwrap()
        .replace("out = x", "out = x + 1");
    fs::write(file, source).unwrap();
    p.ok(&["refresh"]);
    let status: Value =
        serde_json::from_slice(&p.ok(&["status", "--all", "--json"]).stdout).unwrap();
    let detail = &status["goals"][0]["audit_validity"];
    assert_eq!(detail["state"], "stale");
    assert_eq!(detail["specification_changed"], true);
    assert!(
        detail["input_changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["path"].as_str().unwrap().ends_with("/contracts.toml"))
    );
    // Retain older snapshots without making status read receipt files.
    let mut saved = p.snapshot();
    for e in saved["evidence"]["identity"].as_array_mut().unwrap() {
        e.as_object_mut().unwrap().remove("input_files");
    }
    fs::write(
        p.dir.path().join(".elicit/snapshot.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    let status: Value =
        serde_json::from_slice(&p.ok(&["status", "--all", "--json"]).stdout).unwrap();
    assert_eq!(
        status["goals"][0]["audit_validity"]["change_details_available"],
        false
    );
}
