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
        // Existing scheduler tests exercise the explicitly supported planned workflow.
        let mut cmd = self.cmd();
        cmd.args(a);
        if a.first() == Some(&"run") {
            cmd.args(["--workflow", "planned"]);
        }
        cmd.output().unwrap()
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
        .args(["run", "--workflow", "planned"])
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
        .args(["run", "--workflow", "planned"])
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
            .args(["run", "--workflow", "planned", "--retry"])
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

#[test]
fn issue_does_not_create_a_recheck_loop() {
    let p = Project::new("kani", SOURCE);
    let s = p.state();
    let id = s.targets.keys().next().unwrap();
    p.ok(&[
        "agent",
        "issue",
        id,
        "--revision",
        &s.revision.to_string(),
        "--reason",
        "unsupported external model",
    ]);
    assert!(p.state().targets[id].blocker.is_some());
    assert_eq!(p.call(&["run"]).status.code(), Some(1));
    assert!(p.state().run.attempts.is_empty());
    fs::write(p.dir.path().join("PROGRESS.md"), "No new evidence.\n").unwrap();
    assert_eq!(p.call(&["run"]).status.code(), Some(1));
    assert!(p.state().run.attempts.is_empty());
    p.ok(&["run", "--retry"]);
    assert!(p.state().targets[id].blocker.is_none());
}

#[test]
fn operational_reports_do_not_change_simple_crate_inputs() {
    let p = Project::new("kani", SOURCE);
    fs::create_dir(p.dir.path().join("benches")).unwrap();
    fs::write(
        p.dir.path().join("benches/unused.rs"),
        "unknown_benchmark_macro!();\n",
    )
    .unwrap();
    p.ok(&["status"]);
    let original = p.state().input;
    fs::write(p.dir.path().join("PROGRESS.md"), "Progress only.\n").unwrap();
    fs::write(p.dir.path().join("verification-inventory.json"), "{}\n").unwrap();
    p.ok(&["status"]);
    assert_eq!(original, p.state().input);
    fs::write(
        p.dir.path().join("other-input.txt"),
        "Unknown compiler input.\n",
    )
    .unwrap();
    p.ok(&["status"]);
    assert_ne!(original, p.state().input);
}

#[test]
fn local_lemma_import_tracks_content_and_dynamic_inputs_conservatively() {
    for lemma in [
        "import ElicitContracts\ntheorem local_fact : True := trivial\n",
        "import Lean\n#eval IO.FS.readFile \"PROGRESS.md\"\n",
        "import Custom\n",
        "import Lemmas.Untracked\n",
    ] {
        let p = Project::new("aeneas", SOURCE);
        let root = p.dir.path().join("elicit-proofs/aeneas");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Proofs.lean"), "import Lemmas\n").unwrap();
        fs::write(root.join("Lemmas.lean"), lemma).unwrap();
        fs::write(p.dir.path().join("PROGRESS.md"), "one\n").unwrap();
        p.ok(&["status"]);
        let original = p.state().input;
        fs::write(p.dir.path().join("PROGRESS.md"), "two\n").unwrap();
        p.ok(&["status"]);
        assert_eq!(original == p.state().input, lemma.contains("local_fact"));
        let before_lemma = p.state().input;
        fs::write(
            root.join("Lemmas.lean"),
            format!("{lemma}\n-- Changed lemma input.\n"),
        )
        .unwrap();
        p.ok(&["status"]);
        assert_ne!(before_lemma, p.state().input);
    }
}

#[test]
fn build_scripts_and_file_input_macros_keep_report_invalidation() {
    for extra in [
        "pub const REPORT: &str = include_str!(\"../PROGRESS.md\");",
        "",
    ] {
        let p = Project::new("kani", &format!("{SOURCE}\n{extra}\n"));
        if extra.is_empty() {
            fs::write(p.dir.path().join("build.rs"), "fn main() {}\n").unwrap();
        }
        fs::write(p.dir.path().join("PROGRESS.md"), "one\n").unwrap();
        p.ok(&["status"]);
        let original = p.state().input;
        fs::write(p.dir.path().join("PROGRESS.md"), "two\n").unwrap();
        p.ok(&["status"]);
        assert_ne!(original, p.state().input);
    }
}

#[test]
fn unsupported_aeneas_trait_contract_does_not_poison_method_submission() {
    let p = Project::new(
        "aeneas",
        "pub fn identity(x:u32)->u32 { x }\npub struct S;\nimpl core::ops::AddAssign<u32> for S { fn add_assign(&mut self, _:u32) {} }\n",
    );
    let state = p.state();
    let trait_id = state
        .targets
        .values()
        .find(|t| t.kind == cargo_elicit::model::Kind::TraitImpl)
        .unwrap()
        .id
        .clone();
    let proof = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&proof).unwrap();
    fs::write(proof.join("contracts.toml"), format!("schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x r => r = x\"\ntheorem=\"Proofs.identity_spec\"\n[contracts.\"{trait_id}\"]\nensures=\"fun _ => True\"\n")).unwrap();
    assert_eq!(p.call(&["run"]).status.code(), Some(1));
    let state = p.state();
    assert!(state.targets["fixture::identity::Function"].specified);
    assert!(state.unsupported(&trait_id).is_some());
    assert!(state.run.attempts.keys().all(|k| !k.contains(&trait_id)));
    assert!(!state.complete(false));
}

#[test]
fn lean_file_reading_and_rust_path_attributes_keep_report_invalidation() {
    for lean in [true, false] {
        let p = Project::new("kani", SOURCE);
        fs::write(p.dir.path().join("PROGRESS.md"), "pub fn report() {}\n").unwrap();
        if lean {
            let root = p.dir.path().join("elicit-proofs/aeneas");
            fs::create_dir_all(&root).unwrap();
            fs::write(
                root.join("Proofs.lean"),
                "import Lean\n#eval IO.FS.readFile \"PROGRESS.md\"\n",
            )
            .unwrap();
        } else {
            fs::write(
                p.dir.path().join("src/lib.rs"),
                format!("{SOURCE}\n#[path = \"../PROGRESS.md\"] pub mod report;\n"),
            )
            .unwrap();
        }
        p.ok(&["status"]);
        let old = p.state().input;
        fs::write(
            p.dir.path().join("PROGRESS.md"),
            "pub fn changed_report() {}\n",
        )
        .unwrap();
        p.ok(&["status"]);
        assert_ne!(old, p.state().input);
    }
}

#[test]
fn simple_aeneas_cargo_selector_and_overridden_lean_path_are_not_source_changes() {
    let p = Project::new("aeneas", "pub fn identity(x:u32)->u32 { x }\n");
    let without = p
        .cmd()
        .env_remove("RUSTUP_TOOLCHAIN")
        .args(["status", "--json"])
        .output()
        .unwrap();
    assert!(without.status.success());
    let original = p.state().input;
    let with = p
        .cmd()
        .env("RUSTUP_TOOLCHAIN", "stable")
        .env("LEAN_PATH", "/tmp/unused-parent-search-path")
        .args(["status", "--json"])
        .output()
        .unwrap();
    assert!(with.status.success());
    assert_eq!(original, p.state().input);
}

#[test]
fn prepared_proofs_are_checked_before_unrelated_worker_tasks() {
    let p = Project::new(
        "aeneas",
        "pub fn identity(x:u32)->u32 { x }\npub fn unfinished(x:u32)->u32 { x }\n",
    );
    let proof = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&proof).unwrap();
    fs::write(proof.join("contracts.toml"), "schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x result => result = x\"\ntheorem=\"Proofs.identity_spec\"\n").unwrap();
    fs::write(proof.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    // A retained contract for a removed target must not panic during collection.
    use std::io::Write;
    writeln!(fs::OpenOptions::new().append(true).open(proof.join("contracts.toml")).unwrap(), "[contracts.\"fixture::removed::Function\"]\nensures=\"fun x r => r = x\"\ntheorem=\"Proofs.old\"").unwrap();
    let show = p.ok(&["agent", "show", "fixture::identity::Function"]);
    let data: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    p.ok(&[
        "agent",
        "submit",
        "fixture::identity::Function",
        "--revision",
        &data["revision"].to_string(),
        "--note",
        "Reviewed identity while unrelated work remains",
    ]);
    p.required();
    let out = p
        .cmd()
        .env("MOCK_CASE", "noop")
        .args(["run", "--workflow", "planned", "--max-tasks", "1"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(p.state().verified("fixture::identity::Function"));
    assert!(!p.state().targets["fixture::unfinished::Function"].specified);
    let events: Vec<serde_json::Value> =
        fs::read_to_string(p.dir.path().join(".elicit/events.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
    let checked = events
        .iter()
        .position(|e| e["event"] == "verification_finished")
        .unwrap();
    let worker = events
        .iter()
        .position(|e| e["event"] == "worker_started")
        .unwrap();
    assert!(
        checked < worker,
        "saved proof must be recorded before opening unrelated AI work"
    );
}

#[test]
#[cfg(unix)]
fn interrupted_checker_is_retained_as_interrupted_without_fingerprint_errors() {
    let p = Project::new("aeneas", "pub fn identity(x:u32)->u32 { x }\n");
    let proof = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&proof).unwrap();
    fs::write(proof.join("contracts.toml"), "schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x result => result = x\"\ntheorem=\"Proofs.identity_spec\"\n").unwrap();
    fs::write(proof.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    p.ok(&["run"]);
    let before = p.state();
    let mut process = p
        .cmd()
        .env("MOCK_AENEAS", "wait")
        .args([
            "agent",
            "verify",
            "fixture::identity::Function",
            "--revision",
            &before.revision.to_string(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    while !p
        .dir
        .path()
        .join(".elicit/runtime/mock-checker-started")
        .exists()
    {
        if Instant::now() >= deadline {
            let _ = process.kill();
            panic!("checker did not start");
        }
        thread::sleep(Duration::from_millis(50));
    }
    unsafe {
        libc::kill(process.id() as i32, libc::SIGINT);
    }
    let out = process.wait_with_output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(130),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = p.state();
    assert_eq!(before.input, after.input);
    assert_eq!(
        after.results.last().unwrap().outcome,
        cargo_elicit::model::Outcome::Interrupted
    );
    assert!(!after.verified("fixture::identity::Function"));
}

#[test]
fn selected_api_goal_and_revision_checked_plan_are_persisted_without_forging_proof() {
    let f = Project::new("aeneas", "pub fn identity(x:u32)->u32 { x }\n");
    f.ok(&["init", "--tool", "aeneas", "--api", "identity"]);
    let s = f.state();
    assert!(matches!(
        s.goal,
        cargo_elicit::plan::VerificationGoal::SelectedApis(_)
    ));
    let id = s
        .targets
        .values()
        .find(|t| t.name == "identity")
        .unwrap()
        .id
        .clone();
    let file = f.dir.path().join(".elicit/cache/plan.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file,serde_json::to_string(&serde_json::json!({"steps":[{"id":"main","goal":id,"kind":"integrate","target":id,"reason":"Prove the original goal","needs":[]}]})).unwrap()).unwrap();
    let rev = s.revision.to_string();
    f.ok(&[
        "agent",
        "plan",
        "--file",
        file.to_str().unwrap(),
        "--revision",
        &rev,
    ]);
    let s = f.state();
    assert!(!s.verified(&id));
    assert_eq!(s.proof_plan.steps.len(), 1);
    assert!(
        !f.call(&[
            "agent",
            "plan",
            "--file",
            file.to_str().unwrap(),
            "--revision",
            &rev
        ])
        .status
        .success()
    );
    f.ok(&["init", "--whole-package"]);
    assert!(matches!(
        f.state().goal,
        cargo_elicit::plan::VerificationGoal::WholePackage
    ));
}

#[test]
fn successful_kernel_refresh_does_not_skip_required_specification_review() {
    use cargo_elicit::{plan::*, store::Store};
    let p = Project::new("aeneas", "pub fn identity(x:u32)->u32 { x }\n");
    let proof = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&proof).unwrap();
    fs::write(proof.join("contracts.toml"), "schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x result => result = x\"\ntheorem=\"Proofs.identity_spec\"\n").unwrap();
    fs::write(proof.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    p.required();
    let data: serde_json::Value = serde_json::from_slice(&p.ok(&["agent", "show"]).stdout).unwrap();
    p.ok(&[
        "agent",
        "submit",
        "fixture::identity::Function",
        "--revision",
        &data["revision"].to_string(),
        "--note",
        "Original accepted specification",
    ]);
    let id = "fixture::identity::Function";
    let store = Store {
        root: p.dir.path().to_path_buf(),
    };
    store
        .transaction(None, |s| {
            let t = s.targets.get_mut(id).unwrap();
            // A stale legacy state has no known approved semantic identity. A
            // successful proof must not replace the required specification review.
            t.approved_spec_hash = None;
            t.stale = vec!["specification changed".into()];
            s.goal = VerificationGoal::SelectedApis(vec![id.into()]);
            install(
                s,
                vec![ProofStep {
                    id: "review_then_prove".into(),
                    goal: id.into(),
                    kind: StepKind::Contract,
                    target: Some(id.into()),
                    reason: "Review the unresolved selected specification".into(),
                    needs: vec![],
                    statement: None,
                    theorem: None,
                }],
            )
        })
        .unwrap();
    let out = p
        .cmd()
        .env("MOCK_CASE", "resubmit")
        .args(["run", "--workflow", "planned", "--max-tasks", "1"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(p.state().verified(id));
    let events = fs::read_to_string(p.dir.path().join(".elicit/events.jsonl")).unwrap();
    assert!(
        events
            .lines()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
            .any(|e| e["event"] == "worker_started")
    );
}

#[test]
fn autonomous_goal_set_stages_once_and_collects_without_individual_submissions() {
    let p = Project::new(
        "aeneas",
        "pub fn identity(x:u32)->u32 { x }\npub fn other(x:u32)->u32 { x }\n",
    );
    let root = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("contracts.toml"), "schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x result => result = x\"\ntheorem=\"Proofs.identity_spec\"\n[contracts.\"fixture::other::Function\"]\nensures=\"fun x result => result = x\"\ntheorem=\"Proofs.other_spec\"\n").unwrap();
    fs::write(root.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    p.required();
    let out = p
        .cmd()
        .env("MOCK_CASE", "stage")
        .args(["run", "--task-seconds", "1", "--max-seconds", "90"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let s = p.state();
    assert!(s.complete(true));
    assert!(s.proof_plan.steps.is_empty());
    assert_eq!(s.run.attempts.len(), 1);
    let task: serde_json::Value = serde_json::from_slice(
        &fs::read(p.dir.path().join(".elicit/runtime/last-task.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(task["targets"].as_array().unwrap().len(), 2);
    assert!(task["targets"].as_array().unwrap().iter().all(|t| {
        t["aeneas_contract"]
            .as_str()
            .unwrap()
            .starts_with("ElicitContracts.c_")
            && t["specification_ready"] == false
    }));
    let before = s.run.output_tokens;
    p.ok(&["collect"]);
    assert_eq!(p.state().run.output_tokens, before);
    assert!(p.dir.path().join(".elicit/last-review.json").exists());
    // Joint review must rebind previously declared specification dependencies.
    let revision = p.state().revision.to_string();
    p.ok(&[
        "agent",
        "depend",
        "fixture::other::Function",
        "fixture::identity::Function",
        "--revision",
        &revision,
    ]);
    let manifest = root.join("contracts.toml");
    fs::write(
        &manifest,
        fs::read_to_string(&manifest)
            .unwrap()
            .replacen("result = x", "result = x ∧ True", 1),
    )
    .unwrap();
    p.ok(&[
        "agent",
        "stage",
        "--all",
        "--note",
        "Reviewed caller and changed helper together",
    ]);
    p.ok(&["collect"]);
    let s = p.state();
    assert_eq!(
        s.targets["fixture::other::Function"].dependencies["fixture::identity::Function"],
        s.targets["fixture::identity::Function"].spec_hash
    );
    assert!(s.complete(true));
    // An unchanged timed-out input must be eligible for a real recheck.
    fs::write(
        root.join("Proofs.lean"),
        "import ElicitContracts\n-- refresh\n",
    )
    .unwrap();
    let timeout = p
        .cmd()
        .env("MOCK_AENEAS", "wait")
        .args(["collect", "--seconds", "10"])
        .output()
        .unwrap();
    assert_eq!(timeout.status.code(), Some(1));
    assert!(
        p.state()
            .results
            .iter()
            .any(|r| r.outcome == cargo_elicit::model::Outcome::Timeout)
    );
    p.ok(&["collect"]);
    assert!(p.state().complete(true));
}

#[test]
fn saved_sources_require_explicit_current_semantic_review() {
    let p = Project::new("aeneas", "pub fn identity(x:u32)->u32 { x }\n");
    let root = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&root).unwrap();
    let contracts = "schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x result => result = x\"\ntheorem=\"Proofs.identity_spec\"\n";
    fs::write(root.join("contracts.toml"), contracts).unwrap();
    fs::write(root.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    p.required();
    assert_eq!(p.call(&["collect"]).status.code(), Some(1));
    assert!(!p.state().targets["fixture::identity::Function"].specified);
    p.ok(&[
        "agent",
        "stage",
        "--all",
        "--note",
        "Reviewed the identity postcondition",
    ]);
    fs::write(
        root.join("contracts.toml"),
        contracts.replace("result = x", "result = result"),
    )
    .unwrap();
    assert_eq!(p.call(&["collect"]).status.code(), Some(2));
    assert!(!p.state().targets["fixture::identity::Function"].specified);
    p.ok(&[
        "agent",
        "stage",
        "--all",
        "--note",
        "Explicitly reviewed revised postcondition",
    ]);
    fs::write(
        root.join("Proofs.lean"),
        "import ElicitContracts\n-- proof term edits are allowed after review\n",
    )
    .unwrap();
    p.ok(&["collect"]);
    assert!(p.state().complete(true));
}

#[test]
#[cfg(unix)]
fn autonomous_interruption_before_edits_resumes_without_retry() {
    let p = Project::new("aeneas", "pub fn identity(x:u32)->u32 { x }\n");
    let root = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("contracts.toml"), "schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x result => result = x\"\ntheorem=\"Proofs.identity_spec\"\n").unwrap();
    fs::write(root.join("Proofs.lean"), "import ElicitContracts\n").unwrap();
    p.required();
    let mut child = p
        .cmd()
        .env("MOCK_CASE", "interrupt")
        .args(["run"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let end = Instant::now() + Duration::from_secs(30);
    while !p.dir.path().join(".elicit/runtime/mock-started").exists() {
        assert!(Instant::now() < end);
        thread::sleep(Duration::from_millis(50));
    }
    unsafe {
        libc::kill(child.id() as i32, libc::SIGINT);
    }
    assert_eq!(child.wait().unwrap().code(), Some(130));
    let output = p
        .cmd()
        .env("MOCK_CASE", "stage")
        .args(["run"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(p.state().complete(true));
}

#[test]
fn staged_unmapped_contract_without_proofs_is_never_approved() {
    let p = Project::new("aeneas", "pub fn identity(x:u32)->u32 { x }\n");
    let root = p.dir.path().join("elicit-proofs/aeneas");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("contracts.toml"), "schema_version=1\n[contracts.\"fixture::identity::Function\"]\nensures=\"fun x result => result = x\"\n").unwrap();
    p.required();
    p.ok(&[
        "agent",
        "stage",
        "--all",
        "--note",
        "Reviewed postcondition, not yet mapped",
    ]);
    let out = p
        .cmd()
        .env("MOCK_AENEAS", "missing")
        .args(["collect"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = p.state();
    assert!(!s.targets["fixture::identity::Function"].specified);
    assert!(
        !s.processed_contracts
            .contains("fixture::identity::Function")
    );
    assert!(p.dir.path().join(".elicit/staged-review.json").exists());
    // Tool setup/build failure must keep its environment-error classification.
    let out = p
        .cmd()
        .env("MOCK_CHECK_FAIL", "1")
        .args(["collect"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(!p.state().targets["fixture::identity::Function"].specified);
    // A fresh valid mapping can process the same saved reviewed contract.
    assert_eq!(p.call(&["collect"]).status.code(), Some(1));
    assert!(p.state().targets["fixture::identity::Function"].ready());
    assert!(!p.state().verified("fixture::identity::Function"));
}
