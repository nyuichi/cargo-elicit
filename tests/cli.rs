use cargo_elicit::model::{State, Visibility};
use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
struct Project {
    dir: tempfile::TempDir,
}
impl Project {
    fn new(tool: &str, source: &str) -> Self {
        let d = tempfile::tempdir().unwrap();
        let root = d.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("Cargo.toml"),"[package]\nname=\"fixture\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[workspace]\n[features]\ndefault=[\"enabled\"]\nenabled=[]\n").unwrap();
        fs::write(root.join("src/lib.rs"), source).unwrap();
        for name in ["cargo", "codex"] {
            let path = root.join("bin").join(name);
            fs::write(
                &path,
                fs::read(
                    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/support/{name}.py")),
                )
                .unwrap(),
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        if tool == "aeneas" {
            for name in ["charon", "aeneas", "lake"] {
                let path = root.join("bin").join(name);
                fs::copy(
                    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/aeneas.py"),
                    &path,
                )
                .unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
                }
            }
            fs::create_dir_all(root.join(".elicit/runtime/aeneas/backends/lean")).unwrap();
            fs::write(
                root.join(".elicit/runtime/aeneas/backends/lean/lakefile.lean"),
                "",
            )
            .unwrap();
            let backend = root.join(".elicit/runtime/aeneas/backends/lean");
            fs::create_dir_all(backend.join(".lake/build/lib/lean")).unwrap();
            fs::write(backend.join(".lake/build/lib/lean/Aeneas.olean"), "").unwrap();
            fs::write(backend.join("lake-manifest.json"), r#"{"packages":[]}"#).unwrap();
        }
        let p = Self { dir: d };
        p.ok(&["init", "--tool", tool]);
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
            .env("ELICIT_BIN", env!("CARGO_BIN_EXE_cargo-elicit"));
        if self.dir.path().join("bin/aeneas").exists() {
            c.env(
                "AENEAS_HOME",
                self.dir.path().join(".elicit/runtime/aeneas"),
            );
        }
        c
    }
    fn call(&self, a: &[&str]) -> Output {
        self.cmd().args(a).output().unwrap()
    }
    fn ok(&self, a: &[&str]) -> Output {
        let o = self.call(a);
        assert!(
            o.status.success(),
            "{:?}: {}\n{}",
            a,
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        o
    }
    fn state(&self) -> State {
        toml::from_str(&fs::read_to_string(self.dir.path().join(".elicit/state.toml")).unwrap())
            .unwrap()
    }
    fn required(&self) {
        let p = self.dir.path().join("elicit.toml");
        fs::write(
            &p,
            fs::read_to_string(&p)
                .unwrap()
                .replace("required = false", "required = true"),
        )
        .unwrap();
    }
}
const SOURCE: &str =
    "#[cfg_attr(kani, kani::ensures(|result| *result == x))]\npub fn identity(x:u8)->u8 { x }\n";
#[test]
fn both_modes_and_partial_evidence() {
    for tool in ["kani", "creusot"] {
        let p = Project::new(tool, SOURCE);
        p.ok(&["run"]);
        assert!(p.state().targets.values().all(|t| t.specified));
        p.required();
        let out = p.call(&["run"]);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!p.state().results.is_empty());
        assert!(p.state().targets.keys().all(|id| !p.state().verified(id)));
        p.ok(&["status", "--json"]);
        p.ok(&["list", "--json"]);
    }
}
#[test]
fn completed_notification_without_work_is_not_success() {
    let p = Project::new("kani", SOURCE);
    let o = p
        .cmd()
        .env("MOCK_CASE", "noop")
        .args(["run"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    let total = p
        .state()
        .run
        .attempts
        .values()
        .map(|a| a.total)
        .sum::<u32>();
    assert_eq!(total, 1);
    let o = p
        .cmd()
        .env("MOCK_CASE", "noop")
        .args(["run"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(
        p.state()
            .run
            .attempts
            .values()
            .map(|a| a.total)
            .sum::<u32>(),
        total
    );
    p.ok(&["run", "--retry"]);
}
#[test]
fn worker_failure_and_unattended_approval_stop() {
    for case in ["failed", "approval"] {
        let p = Project::new("kani", SOURCE);
        let o = p.cmd().env("MOCK_CASE", case).arg("run").output().unwrap();
        assert_eq!(o.status.code(), Some(2));
        assert!(!p.state().complete(false));
        assert!(p.state().run.active.is_none());
    }
}
#[test]
fn revision_conflict_preserves_state() {
    let p = Project::new("kani", SOURCE);
    let s = p.state();
    let id = s.targets.keys().next().unwrap();
    let o = p.call(&[
        "agent",
        "issue",
        id,
        "--revision",
        "9999",
        "--reason",
        "bad",
    ]);
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(p.state().revision, s.revision);
}
#[test]
fn verifier_switch_rejected_without_erasing_ledger() {
    let p = Project::new("kani", SOURCE);
    let state = fs::read(p.dir.path().join(".elicit/state.toml")).unwrap();
    let c = p.dir.path().join("elicit.toml");
    fs::write(
        &c,
        fs::read_to_string(&c).unwrap().replace("kani", "creusot"),
    )
    .unwrap();
    assert_eq!(p.call(&["init"]).status.code(), Some(2));
    assert_eq!(
        state,
        fs::read(p.dir.path().join(".elicit/state.toml")).unwrap()
    );
}
#[test]
fn aliases_cfg_traits_and_impl_identities() {
    let p = Project::new(
        "kani",
        "mod hidden { pub fn f(){} }\npub use hidden::f as exposed;\n#[cfg(feature=\"enabled\")] pub fn enabled(){}\n#[cfg(not(feature=\"enabled\"))] pub fn disabled(){}\npub struct A; pub struct B; pub trait T { fn f(&self); } impl A { pub fn f(&self){} } impl B { pub fn f(&self){} }\n",
    );
    let s = p.state();
    let f = s
        .targets
        .values()
        .find(|t| t.name == "f" && t.kind == cargo_elicit::model::Kind::Function)
        .unwrap();
    assert_eq!(f.visibility, Visibility::Public);
    assert!(f.aliases.contains(&"exposed".into()));
    assert!(s.targets.values().any(|t| t.name == "enabled"));
    assert!(!s.targets.values().any(|t| t.name == "disabled"));
    assert_eq!(
        s.targets
            .values()
            .filter(|t| t.name == "f" && t.kind == cargo_elicit::model::Kind::Method)
            .count(),
        2
    );
    assert!(s.scope.resolved_features.contains(&"enabled".into()));
}
#[test]
fn docs_do_not_turn_public_api_into_proof_helper() {
    let p = Project::new(
        "kani",
        "/// kani::proof logic predicate\npub fn actual() {}\nmacro_rules! generated { () => {} }\ngenerated!();\n",
    );
    assert_eq!(
        p.state().targets.values().next().unwrap().visibility,
        Visibility::Public
    );
    assert!(!p.state().analysis_incomplete.is_empty());
}
#[test]
fn syntax_failure_does_not_mark_specified() {
    let p = Project::new("kani", SOURCE);
    let s = p.state();
    let id = s.targets.keys().next().unwrap();
    let o = p
        .cmd()
        .env("MOCK_CHECK_FAIL", "1")
        .args([
            "agent",
            "submit",
            id,
            "--revision",
            &s.revision.to_string(),
            "--note",
            "reviewed",
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(!p.state().targets[id].specified);
}
#[test]
fn runtime_edits_remain_blocked_and_sources_preserved() {
    let p = Project::new("kani", SOURCE);
    let o = p
        .cmd()
        .env("MOCK_CASE", "body")
        .arg("run")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    let s = p.state();
    assert!(s.targets.values().any(|t| t.implementation_changed));
    let source = fs::read_to_string(p.dir.path().join("src/lib.rs")).unwrap();
    assert!(source.contains("wrapping_add"));
    p.ok(&["init"]);
    assert_eq!(
        fs::read_to_string(p.dir.path().join("src/lib.rs")).unwrap(),
        source
    );
    assert!(p.state().targets.values().any(|t| t.implementation_changed));
}
#[cfg(unix)]
#[test]
fn interruption_resume_and_run_lock_both_verifiers() {
    for tool in ["kani", "creusot"] {
        let p = Project::new(tool, SOURCE);
        let mut child = p
            .cmd()
            .env("MOCK_CASE", "interrupt")
            .arg("run")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let end = Instant::now() + Duration::from_secs(45);
        while !p.dir.path().join(".elicit/runtime/mock-started").exists() {
            assert!(Instant::now() < end, "worker never started");
            thread::sleep(Duration::from_millis(100));
        }
        assert_eq!(p.call(&["run"]).status.code(), Some(2));
        unsafe {
            libc::kill(child.id() as i32, libc::SIGINT);
        }
        assert_eq!(child.wait().unwrap().code(), Some(130));
        assert!(p.state().run.active.is_some());
        p.ok(&["run", "--retry"]);
        assert!(p.state().complete(false));
        fs::remove_dir_all(p.dir.path().join(".elicit/cache")).unwrap();
        p.ok(&["init"]);
        assert!(p.state().complete(false));
    }
}

#[test]
fn workspace_requires_explicit_single_package() {
    let d = tempfile::tempdir().unwrap();
    fs::write(
        d.path().join("Cargo.toml"),
        "[workspace]\nmembers=[\"a\",\"b\"]\nresolver=\"2\"\n",
    )
    .unwrap();
    for name in ["a", "b"] {
        fs::create_dir_all(d.path().join(name).join("src")).unwrap();
        fs::write(
            d.path().join(name).join("Cargo.toml"),
            format!("[package]\nname=\"{name}\"\nversion=\"0.0.0\"\nedition=\"2021\"\n"),
        )
        .unwrap();
        fs::write(d.path().join(name).join("src/lib.rs"), "pub fn f() {}\n").unwrap();
    }
    let mut c = Command::new(env!("CARGO_BIN_EXE_cargo-elicit"));
    let o = c
        .current_dir(d.path())
        .args(["init", "--tool", "kani"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(!d.path().join(".elicit/state.toml").exists());
    let o = Command::new(env!("CARGO_BIN_EXE_cargo-elicit"))
        .current_dir(d.path())
        .args(["init", "--package", "a"])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let s: State =
        toml::from_str(&fs::read_to_string(d.path().join(".elicit/state.toml")).unwrap()).unwrap();
    assert_eq!(s.scope.package, "a");
    assert!(s.targets.keys().all(|id| id.starts_with("a::")));
}

#[test]
fn formal_verifier_environment_failure_exits_two() {
    let p = Project::new("kani", SOURCE);
    p.ok(&["run"]);
    p.required();
    let o = p
        .cmd()
        .env("MOCK_PROOF_FAIL", "1")
        .arg("run")
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        p.state()
            .results
            .iter()
            .any(|r| r.outcome == cargo_elicit::model::Outcome::EnvironmentError)
    );
}

#[cfg(unix)]
#[test]
fn unchanged_status_reuses_persisted_semantic_analysis() {
    let p = Project::new("kani", SOURCE);
    use std::os::unix::fs::PermissionsExt;
    let external = tempfile::tempdir().unwrap();
    let executable = external.path().join("rust-analyzer");
    fs::write(&executable, "#!/bin/sh\nexit 99\n").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let original = p
        .cmd()
        .get_envs()
        .find(|(k, _)| *k == "PATH")
        .unwrap()
        .1
        .unwrap()
        .to_owned();
    let out = p
        .cmd()
        .env(
            "PATH",
            format!(
                "{}:{}",
                external.path().display(),
                original.to_string_lossy()
            ),
        )
        .args(["status", "--json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn aeneas_requires_matching_kernel_proof_and_standard_axioms() {
    let p = Project::new("aeneas", "pub fn identity(x:u32)->u32 { x }\n");
    let proof = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&proof).unwrap();
    fs::write(proof.join("contracts.toml"), "schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x result => result = x\"\ntheorem=\"Proofs.identity_spec\"\n").unwrap();
    p.ok(&["run"]);
    assert!(p.state().complete(false));
    p.required();
    assert_eq!(p.call(&["run"]).status.code(), Some(1));
    assert!(!p.state().complete(true), "translation is not a proof");
    fs::write(proof.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    for rejection in [
        "sorryAx",
        "projectAssumption",
        "mismatch",
        "opaque",
        "missing",
        "wrongtarget",
        "errors",
        "specialized",
        "specialized_mut",
        "disabled_checks",
    ] {
        let out = p
            .cmd()
            .env("MOCK_AENEAS", rejection)
            .args(["run", "--retry"])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!p.state().complete(true));
    }
    p.ok(&["run", "--retry"]);
    assert!(p.state().complete(true));
    let old = p.state().targets["fixture::identity::Function"]
        .spec_hash
        .clone();
    fs::write(
        proof.join("Proofs.lean"),
        "import ElicitContracts\n-- revised proof\n",
    )
    .unwrap();
    p.ok(&["status"]);
    assert_eq!(
        old,
        p.state().targets["fixture::identity::Function"].spec_hash
    );
    assert!(
        !p.state().complete(true),
        "proof edits invalidate proof freshness"
    );
    fs::write(proof.join("Specs.lean"), "-- revised model\n").unwrap();
    p.ok(&["status"]);
    let s = p.state();
    assert_ne!(old, s.targets["fixture::identity::Function"].spec_hash);
    assert!(!s.targets["fixture::identity::Function"].stale.is_empty());
}
