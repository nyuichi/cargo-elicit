//! Opt-in tests use actual Kani/Creusot/Aeneas and a deterministic Codex protocol peer.
//! They do not call a live AI model or consume API credits.
use cargo_elicit::model::State;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn cmd(root: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_cargo-elicit"));
    c.current_dir(root)
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("ELICIT_BIN", env!("CARGO_BIN_EXE_cargo-elicit"));
    c
}
fn ok(root: &Path, args: &[&str]) {
    let o = cmd(root).args(args).output().unwrap();
    assert!(
        o.status.success(),
        "{} {:?}: {}\n{}",
        root.display(),
        args,
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
}
fn state(root: &Path) -> State {
    toml::from_str(&fs::read_to_string(root.join(".elicit/state.toml")).unwrap()).unwrap()
}
fn prepare_fixture(tool: &str, fixture: &str) -> PathBuf {
    let root = tempfile::Builder::new()
        .prefix(&format!("elicit-native-{tool}-"))
        .tempdir()
        .unwrap()
        .keep();
    let from = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(fixture);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    for name in ["Cargo.toml", "Cargo.lock", "why3find.json", "src/lib.rs"] {
        if from.join(name).is_file() {
            fs::copy(from.join(name), root.join(name)).unwrap();
        }
    }
    if from.join("elicit-proofs").is_dir() {
        for e in walkdir::WalkDir::new(from.join("elicit-proofs")) {
            let e = e.unwrap();
            let dest = root.join(e.path().strip_prefix(&from).unwrap());
            if e.file_type().is_dir() {
                fs::create_dir_all(dest).unwrap();
            } else {
                fs::copy(e.path(), dest).unwrap();
            }
        }
    }
    let p = root.join("bin/codex");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/codex.py"),
        &p,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    }
    root
}
fn exercise_fixture(tool: &str, fixture: &str) {
    let root = prepare_fixture(tool, fixture);
    ok(&root, &["init", "--tool", tool]);
    #[cfg(unix)]
    {
        let mut worker = cmd(&root)
            .env("MOCK_CASE", "interrupt")
            .arg("run")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        while !root.join(".elicit/runtime/mock-started").exists() {
            assert!(
                Instant::now() < deadline,
                "{}: no worker started",
                root.display()
            );
            thread::sleep(Duration::from_millis(100));
        }
        unsafe {
            libc::kill(worker.id() as i32, libc::SIGINT);
        }
        assert_eq!(worker.wait().unwrap().code(), Some(130));
        assert!(state(&root).run.active.is_some());
    }
    ok(&root, &["run", "--retry"]);
    let s = state(&root);
    assert!(
        s.complete(false),
        "{}: specification mode incomplete",
        root.display()
    );
    assert!(s.results.is_empty());
    let path = root.join("elicit.toml");
    fs::write(
        &path,
        fs::read_to_string(&path)
            .unwrap()
            .replace("required = false", "required = true"),
    )
    .unwrap();
    if fixture == "aeneas-rich" {
        let mut running = cmd(&root)
            .args(["run", "--retry"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut observed = false;
        while running.try_wait().unwrap().is_none() {
            if !observed
                && state(&root)
                    .results
                    .iter()
                    .any(|r| r.outcome == cargo_elicit::model::Outcome::Success)
            {
                let status = cmd(&root).arg("status").output().unwrap();
                if !status.status.success() {
                    unsafe {
                        libc::kill(running.id() as i32, libc::SIGINT);
                    }
                    let _ = running.wait();
                    panic!(
                        "{}: status failed during verification: {}",
                        root.display(),
                        String::from_utf8_lossy(&status.stderr)
                    );
                }
                let text = String::from_utf8(status.stdout).unwrap();
                assert!(text.contains("fully verified:"));
                fs::write(root.join(".elicit/runtime/progress-status.txt"), text).unwrap();
                observed = true;
            }
            thread::sleep(Duration::from_millis(100));
        }
        let output = running.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(observed, "no mid-verification status observation was made");
    } else {
        ok(&root, &["run", "--retry"]);
    }
    let s = state(&root);
    assert!(
        s.complete(true),
        "{}: required mode incomplete: {:?}",
        root.display(),
        s.results
    );
    assert!(
        s.targets
            .values()
            .filter(
                |t| t.visibility != cargo_elicit::model::Visibility::Auxiliary && t.kind.proof()
            )
            .all(|t| s.verified(&t.id))
    );
    if fixture == "aeneas" {
        let specs = root.join("elicit-proofs/aeneas/Specs.lean");
        let original = fs::read_to_string(&specs).unwrap();
        let before = state(&root);
        let worker_count = || {
            fs::read_to_string(root.join(".elicit/events.jsonl"))
                .unwrap()
                .lines()
                .filter(|line| line.contains("\"worker_started\""))
                .count()
        };
        let workers = worker_count();
        fs::write(
            &specs,
            format!(
                "{original}\nnamespace FixtureModel\ndef unrelated : Nat := 7\nend FixtureModel\n"
            ),
        )
        .unwrap();
        ok(&root, &["status"]);
        let extended = state(&root);
        let id = "elicit-aeneas-fixture::identity::Function";
        assert!(
            extended.targets[id].ready(),
            "unrelated model extension reopened accepted specification"
        );
        assert_eq!(before.targets[id].spec_hash, extended.targets[id].spec_hash);
        let marker = "elicit-aeneas-fixture::Marker::Struct";
        assert!(
            extended.targets[marker].ready(),
            "unrelated extension reopened type invariant review"
        );
        assert_eq!(
            before.targets[marker].spec_hash,
            extended.targets[marker].spec_hash
        );
        assert!(
            !extended.verified(id),
            "proof input edits still require a fresh kernel check"
        );
        ok(&root, &["run"]);
        assert!(state(&root).complete(true));
        assert_eq!(
            workers,
            worker_count(),
            "unrelated model extension invoked an AI worker"
        );
        let extended_source = fs::read_to_string(&specs).unwrap();
        fs::write(
            &specs,
            extended_source.replace("(x : U32) : U32 := x", "(x : U32) : U32 := 0#u32"),
        )
        .unwrap();
        ok(&root, &["status"]);
        assert!(
            !state(&root).targets[id].ready(),
            "changed reachable model meaning was not marked stale"
        );
        assert!(!state(&root).verified(id));
        let revision = state(&root).revision.to_string();
        assert_eq!(
            cmd(&root)
                .args(["agent", "verify", id, "--revision", &revision])
                .output()
                .unwrap()
                .status
                .code(),
            Some(1),
            "old identity proof accepted for constant-zero model"
        );
        fs::write(&specs, &extended_source).unwrap();
        ok(&root, &["run", "--retry"]);
        assert!(state(&root).complete(true));
        fs::write(&specs, format!("{extended_source}\nnamespace FixtureModel\ninductive Extra where | first | second\nend FixtureModel\n")).unwrap();
        ok(&root, &["status"]);
        assert!(
            !state(&root).targets[id].ready(),
            "unsupported inductive metadata must fall back to whole-model review"
        );
        let cache: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(".elicit/cache/model-inputs.json")).unwrap(),
        )
        .unwrap();
        assert!(
            cache["inputs"].is_null(),
            "inductive declaration metadata was approximated"
        );
        fs::write(&specs, extended_source).unwrap();
        ok(&root, &["run", "--retry"]);
        assert!(state(&root).complete(true));
    }
    if tool == "aeneas" {
        let events = || {
            fs::read_to_string(root.join(".elicit/events.jsonl"))
                .unwrap()
                .lines()
                .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
                .collect::<Vec<_>>()
        };
        let original_input = state(&root).input;
        fs::write(
            root.join("PROGRESS.md"),
            "Report-only update after successful proofs.\n",
        )
        .unwrap();
        ok(&root, &["status"]);
        assert_eq!(original_input, state(&root).input);
        assert!(state(&root).complete(true));
        let workers_before = events()
            .iter()
            .filter(|e| e["event"] == "worker_started")
            .count();
        let proof = root.join("elicit-proofs/aeneas/Proofs.lean");
        let mut source = fs::read_to_string(&proof).unwrap();
        source.push_str("\n-- Proof artifact update: recheck without an AI turn.\n");
        fs::write(&proof, source).unwrap();
        if fixture == "aeneas-rich" {
            ok(&root, &["status"]);
            let current = state(&root);
            let id = current
                .targets
                .values()
                .find(|t| t.kind.proof())
                .unwrap()
                .id
                .clone();
            ok(
                &root,
                &[
                    "agent",
                    "verify",
                    &id,
                    "--revision",
                    &current.revision.to_string(),
                ],
            );
        }
        ok(&root, &["run"]);
        assert!(state(&root).complete(true));
        assert_eq!(
            workers_before,
            events()
                .iter()
                .filter(|e| e["event"] == "worker_started")
                .count()
        );
        if fixture == "aeneas-rich" {
            let last = events()
                .into_iter()
                .rev()
                .find(|e| e["event"] == "verification_finished")
                .unwrap();
            assert_eq!(last["data"]["requested"].as_array().unwrap().len(), 1);
            assert_eq!(
                last["data"]["results"].as_array().unwrap().len(),
                5,
                "all checked ready contracts must be recorded"
            );
        }
    }
    if fixture == "aeneas" {
        let proof = root.join("elicit-proofs/aeneas/Proofs.lean");
        let original = fs::read_to_string(&proof).unwrap();
        let id = "elicit-aeneas-fixture::identity::Function";
        let proposition = cargo_elicit::aeneas::contract_name(id);
        for source in [
            format!(
                "import ElicitContracts\n theorem Proofs.identity_spec : {proposition} := by sorry\n"
            ),
            format!(
                "import ElicitContracts\n axiom projectAssumption : {proposition}\n theorem Proofs.identity_spec : {proposition} := projectAssumption\n"
            ),
            "import ElicitContracts\n theorem Proofs.identity_spec : True := by trivial\n".into(),
        ] {
            fs::write(&proof, source).unwrap();
            ok(&root, &["status"]);
            let revision = state(&root).revision.to_string();
            let o = cmd(&root)
                .args(["agent", "verify", id, "--revision", &revision])
                .output()
                .unwrap();
            assert_eq!(
                o.status.code(),
                Some(1),
                "{}: {}",
                root.display(),
                String::from_utf8_lossy(&o.stderr)
            );
            assert!(
                !state(&root).verified(id),
                "admitted, assumed or mismatched theorem was accepted"
            );
        }
        fs::write(&proof, original).unwrap();
        ok(&root, &["run", "--retry"]);
        assert!(state(&root).complete(true));
    }
    fs::remove_dir_all(root.join(".elicit/cache")).unwrap();
    ok(&root, &["init"]);
    assert!(
        state(&root).complete(true),
        "{}: lost persisted proof",
        root.display()
    );
    fs::remove_dir_all(&root).unwrap();
}
#[test]
#[ignore = "requires Kani 0.68.x, matching nightly and rust-analyzer"]
fn kani_both_modes_interruption_resume() {
    exercise_fixture("kani", "kani");
}
#[test]
#[ignore = "requires Creusot 0.14.x with Why3find/Why3/SMT setup and rust-analyzer"]
fn creusot_both_modes_interruption_resume() {
    exercise_fixture("creusot", "creusot");
}

#[test]
#[ignore = "requires pinned Charon/Aeneas, Lean 4.35.0 library and rust-analyzer"]
fn aeneas_both_modes_interruption_resume() {
    exercise_fixture("aeneas", "aeneas");
}

#[test]
#[ignore = "requires pinned Charon/Aeneas, Lean 4.35.0 library and rust-analyzer"]
fn aeneas_methods_structs_slices_and_helpers() {
    exercise_fixture("aeneas", "aeneas-rich");
}

#[test]
#[ignore = "requires pinned Charon/Aeneas, Lean 4.35.0 library and rust-analyzer"]
fn aeneas_model_dependencies_do_not_turn_documentation_into_specification_changes() {
    let root = prepare_fixture("aeneas", "aeneas");
    ok(&root, &["init", "--tool", "aeneas"]);
    ok(&root, &["run"]);
    let before = state(&root);
    assert!(before.complete(false));
    let id = "elicit-aeneas-fixture::identity::Function";
    assert!(
        before.targets[id].model_inputs.contains_key("@rust_model"),
        "native model dependency collection was not exercised"
    );
    fs::write(
        root.join("README.md"),
        "Documentation-only change; the Rust and logical model are unchanged.\n",
    )
    .unwrap();
    ok(&root, &["status"]);
    let after = state(&root);
    assert_ne!(
        before.input, after.input,
        "this test must exercise conservative whole-proof invalidation"
    );
    for (id, target) in &before.targets {
        assert_eq!(
            target.spec_hash, after.targets[id].spec_hash,
            "documentation reopened {id}"
        );
        assert!(after.targets[id].ready(), "documentation made {id} stale");
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires pinned Charon/Aeneas and Lean 4.35.0"]
fn aeneas_planned_lemma_checkpoints_and_caller_registration() {
    use cargo_elicit::{plan::*, store::Store};
    let root = prepare_fixture("aeneas", "aeneas");
    ok(&root, &["init", "--tool", "aeneas", "--api", "identity"]);
    let store = Store { root: root.clone() };
    let checkpoint = || {
        let revision = state(&root).revision.to_string();
        let out = cmd(&root)
            .args(["agent", "verify", "--revision", &revision])
            .output()
            .unwrap();
        assert!(
            matches!(out.status.code(), Some(0 | 1)),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    let id = "elicit-aeneas-fixture::identity::Function";
    let proof_root = root.join("elicit-proofs/aeneas");
    let source = r#"import ElicitContracts
open Aeneas Aeneas.Std Result
namespace Checkpoint

def helper (x : U32) : Result U32 := .ok x

theorem helper_contract (x : U32) : ∃ y, helper x = ok y ∧ y.val = x.val := by
  exact ⟨x, rfl, rfl⟩
elicit_spec helper_rule from helper_contract
attribute [irreducible] helper

def caller (x : U32) : Result U32 := do
  let y ← helper x
  pure y

theorem caller_rule (x : U32) : caller x ⦃ y => y.val = x.val ⦄ := by
  unfold caller
  step*
end Checkpoint
"#;
    fs::write(proof_root.join("Lemmas.lean"), source).unwrap();
    fs::remove_file(proof_root.join("Proofs.lean")).unwrap();
    ok(&root, &["status"]);
    let step = ProofStep {
        id: "caller_rule".into(),
        goal: id.into(),
        kind: StepKind::Lemma,
        target: None,
        reason: "Exercise composition of a checked helper through the Aeneas step database".into(),
        needs: vec![],
        statement: Some("∀ x : Aeneas.Std.U32, Checkpoint.caller x ⦃ y => y.val = x.val ⦄".into()),
        theorem: Some("Checkpoint.caller_rule".into()),
    };
    store
        .transaction(None, |s| install(s, vec![step.clone()]))
        .unwrap();
    checkpoint();
    let s = store.load().unwrap();
    assert!(checked(&s, &step));
    assert!(!s.verified(id));
    // A changed proposition cannot inherit the old checkpoint even when the
    // theorem name and original Rust stay unchanged.
    let mut wrong = step.clone();
    wrong.statement = Some("False".into());
    store
        .transaction(None, |s| install(s, vec![wrong.clone()]))
        .unwrap();
    checkpoint();
    assert!(!checked(&store.load().unwrap(), &wrong));
    // An admitted theorem is never a checked checkpoint.
    fs::write(
        proof_root.join("Lemmas.lean"),
        format!("{source}\ntheorem bad : False := by sorry\n"),
    )
    .unwrap();
    ok(&root, &["status"]);
    wrong.theorem = Some("bad".into());
    store
        .transaction(None, |s| install(s, vec![wrong.clone()]))
        .unwrap();
    checkpoint();
    assert!(!checked(&store.load().unwrap(), &wrong));
    // Selected completion does not require authoring the unrelated Marker type.
    fs::write(proof_root.join("Lemmas.lean"), source).unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/aeneas/elicit-proofs/aeneas/Proofs.lean");
    fs::copy(fixture, proof_root.join("Proofs.lean")).unwrap();
    ok(&root, &["status"]);
    let revision = state(&root).revision.to_string();
    ok(
        &root,
        &[
            "agent",
            "submit",
            id,
            "--revision",
            &revision,
            "--note",
            "Reviewed selected original API",
        ],
    );
    checkpoint();
    let s = store.load().unwrap();
    assert!(s.complete(true));
    assert!(!s.targets["elicit-aeneas-fixture::Marker::Struct"].ready());
    // A transient failure in collecting semantic metadata must not permanently
    // lose an unchanged approval. Recovery still requires fresh proof evidence
    // when the global proof input changed.
    let specs = proof_root.join("Specs.lean");
    let original_specs = fs::read_to_string(&specs).unwrap();
    fs::write(&specs, "import ElicitGenerated\ndef broken := (\n").unwrap();
    ok(&root, &["status"]);
    assert!(!state(&root).targets[id].ready());
    fs::write(&specs, &original_specs).unwrap();
    let proofs = proof_root.join("Proofs.lean");
    fs::write(
        &proofs,
        format!(
            "{}\n-- Require a fresh whole-input kernel check.\n",
            fs::read_to_string(&proofs).unwrap()
        ),
    )
    .unwrap();
    ok(&root, &["status"]);
    assert!(state(&root).targets[id].ready());
    assert!(!state(&root).verified(id));
    checkpoint();
    assert!(state(&root).complete(true));
    // An actually changed referenced model remains unapproved.
    fs::write(
        &specs,
        original_specs.replace(": U32 := x", ": U32 := 0#u32"),
    )
    .unwrap();
    ok(&root, &["status"]);
    assert!(!state(&root).targets[id].ready());
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires the pinned real Lean/Aeneas backend"]
fn aeneas_batch_audits_isolate_invalid_roots_and_diagnostics() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("Batch.lean");
    fs::write(
        &file,
        format!(
            "import Aeneas\n{}\n{}",
            include_str!("../src/lean/ElicitAudit.lean"),
            r#"
theorem good : True := True.intro
theorem bad : False := by sorry
axiom project : False
theorem ElicitBatchAudit.root_5 : False := by sorry
elicit_audit 0 "True" "good"
elicit_audit 1 "True" "missing"
elicit_audit 2 "False" "good"
elicit_audit 3 "False" "bad"
elicit_audit 4 "False" "project"
elicit_audit 5 "True" "good"
elicit_audit 6 "True" "good"
elicit_audit 7 "malformed (" "good"
theorem poly {α : Type u} (x : α) : x = x := rfl
elicit_audit 8 "∀ {α : Type} (x : α), x = x" "poly"
unsafe def forged : False := unsafeCast True.intro
elicit_audit 9 "False" "forged"
theorem implicitRoot {x : Nat} : x = x := rfl
elicit_audit 10 "0 = 0" "implicitRoot"
"#
        ),
    )
    .unwrap();
    let out = Command::new("lake")
        .args(["env", "lean"])
        .arg(&file)
        .current_dir(Path::new(&std::env::var("AENEAS_HOME").unwrap()).join("backends/lean"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let rows: Vec<serde_json::Value> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix("ELICIT_AUDIT="))
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len(), 11);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row["index"], index);
        assert_eq!(row["checked"], matches!(index, 0 | 6 | 8 | 10));
    }
}
