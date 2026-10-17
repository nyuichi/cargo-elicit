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
fn exercise_fixture(tool: &str, fixture: &str) {
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
