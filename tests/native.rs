//! Opt-in tests use actual Kani/Creusot and a deterministic Codex protocol peer.
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
fn exercise(tool: &str) {
    let root = tempfile::Builder::new()
        .prefix(&format!("elicit-native-{tool}-"))
        .tempdir()
        .unwrap()
        .keep();
    let from = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(tool);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    for name in ["Cargo.toml", "Cargo.lock", "why3find.json", "src/lib.rs"] {
        if from.join(name).is_file() {
            fs::copy(from.join(name), root.join(name)).unwrap();
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
    ok(&root, &["run", "--retry"]);
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
    exercise("kani");
}
#[test]
#[ignore = "requires Creusot 0.14.x with Why3find/Why3/SMT setup and rust-analyzer"]
fn creusot_both_modes_interruption_resume() {
    exercise("creusot");
}
