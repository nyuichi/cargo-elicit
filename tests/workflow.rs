use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

struct Project {
    dir: tempfile::TempDir,
}
impl Project {
    fn new() -> Self {
        Self::with_source("pub fn identity(x:u32)->u32{x}\npub fn other(x:u32)->u32{x}\n")
    }
    fn with_source(source: &str) -> Self {
        let p = Self {
            dir: tempfile::tempdir().unwrap(),
        };
        fs::create_dir(p.root().join("src")).unwrap();
        fs::write(
            p.root().join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.0.0'\nedition='2021'\n[workspace]\n",
        )
        .unwrap();
        fs::write(p.root().join("src/lib.rs"), source).unwrap();
        p.script("echo run >> .elicit/count\nprintf 'native success\\n'\n");
        p.ok(&[
            "init",
            "--api",
            "identity",
            "--api",
            "other",
            "--command",
            "sh",
            "verify.sh",
        ]);
        p
    }
    fn root(&self) -> &Path {
        self.dir.path()
    }
    fn script(&self, source: &str) {
        fs::write(self.root().join("verify.sh"), source).unwrap();
    }
    fn cmd(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_cargo-elicit"));
        c.current_dir(self.root());
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
    fn json(&self, args: &[&str]) -> Value {
        serde_json::from_slice(&self.ok(args).stdout).unwrap()
    }
    fn snapshot(&self) -> Value {
        serde_json::from_slice(&fs::read(self.root().join(".elicit/snapshot.json")).unwrap())
            .unwrap()
    }
    fn count(&self) -> usize {
        fs::read_to_string(self.root().join(".elicit/count"))
            .unwrap_or_default()
            .lines()
            .count()
    }
}

#[test]
fn only_three_commands_and_no_proof_scaffolds_or_approval_are_required() {
    let p = Project::new();
    let help = String::from_utf8(p.ok(&["--help"]).stdout).unwrap();
    let names: Vec<_> = help
        .lines()
        .skip_while(|l| *l != "Commands:")
        .skip(1)
        .take_while(|l| !l.trim().is_empty())
        .map(|l| l.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(names, ["init", "status", "verify"]);
    for name in [
        "review", "finish", "check", "audit", "notes", "resume", "show", "list", "plan", "refresh",
    ] {
        assert_eq!(p.call(&[name]).status.code(), Some(2));
    }
    assert!(!p.root().join("elicit-proofs").exists());
    assert!(!p.root().join(".elicit/notes").exists());
    assert!(p.snapshot().get("reviews").is_none());
}
#[test]
fn shared_native_command_runs_once_and_saved_results_are_reused() {
    let p = Project::new();
    let first = p.json(&["verify", "--json"]);
    assert_eq!(first["operation"]["passed"], 2);
    assert_eq!(
        p.count(),
        1,
        "same project command should not run once per API"
    );
    let status = p.json(&["status", "--saved", "--json"]);
    assert_eq!(status["summary"]["completed"], 2);
    assert!(status["goals"].as_array().unwrap().is_empty());
    assert_eq!(
        status["evidence_semantics"],
        "native_command_result; no theorem, axiom or Rust correspondence checks"
    );
    let second = p.json(&["verify", "--json"]);
    assert_eq!(second["operation"]["reused_count"], 2);
    assert_eq!(p.count(), 1);
    p.ok(&["verify", "--fresh"]);
    assert_eq!(p.count(), 2);
    assert_eq!(
        p.json(&["status", "--saved", "--json"])["summary"]["completed"],
        2
    );
}
#[test]
fn source_and_command_edits_stale_results_but_internal_logs_do_not() {
    let p = Project::new();
    p.ok(&["verify"]);
    let old = p.snapshot()["input"].clone();
    fs::write(p.root().join(".elicit/arbitrary.md"), "not an input").unwrap();
    p.ok(&["status"]);
    assert_eq!(p.snapshot()["input"], old);
    fs::write(
        p.root().join("src/lib.rs"),
        "pub fn identity(x:u32)->u32{x+1}\npub fn other(x:u32)->u32{x}\n",
    )
    .unwrap();
    let status = p.json(&["status", "--json"]);
    assert_eq!(status["summary"]["remaining"], 2);
    assert!(
        status["goals"][0]["changes_since_last_evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().ends_with("src/lib.rs"))
    );
    p.ok(&["verify"]);
    assert_eq!(p.count(), 2);
    p.script("echo run >> .elicit/count\n# changed native script\n");
    p.ok(&["verify"]);
    assert_eq!(p.count(), 3);
}
#[test]
fn failure_has_durable_log_and_fresh_success_replaces_it() {
    let p = Project::new();
    p.script(
        "echo run >> .elicit/count\necho 'Proof.lean:3:2: error: expected failure' >&2\nexit 7\n",
    );
    let failed = p.call(&["verify", "--json"]);
    assert_eq!(failed.status.code(), Some(1));
    let output: Value = serde_json::from_slice(&failed.stdout).unwrap();
    let receipt = output["operation"]["results"][0]["receipt"]
        .as_str()
        .unwrap();
    let saved: Value = serde_json::from_slice(&fs::read(p.root().join(receipt)).unwrap()).unwrap();
    let result = &saved["report"]["results"][0];
    assert_eq!(result["exit_code"], 7);
    let log = p
        .root()
        .join(result["diagnostic"]["cache"].as_str().unwrap())
        .join(result["diagnostic"]["log"].as_str().unwrap());
    assert!(
        fs::read_to_string(log)
            .unwrap()
            .contains("expected failure")
    );
    p.script("echo run >> .elicit/count\n");
    p.ok(&["verify", "--fresh"]);
    assert_eq!(
        p.json(&["status", "--saved", "--json"])["summary"]["remaining"],
        0
    );
}
#[test]
fn input_race_is_not_saved_as_current_success() {
    let p = Project::new();
    p.script("echo '// edit during execution' >> src/lib.rs\nexit 0\n");
    let out = p.call(&["verify", "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["operation"]["passed"], 0);
    let evidence = &p.snapshot()["evidence"]["identity"][0];
    assert_eq!(evidence["result"]["exit_code"], 0);
    assert_eq!(evidence["result"]["outcome"], "input_changed");
    assert_eq!(evidence["accepted"], false);
}
#[test]
fn missing_or_corrupt_receipt_forces_execution_and_partial_reuse_is_preserved() {
    let p = Project::new();
    p.ok(&["verify"]);
    let s = p.snapshot();
    let path = p
        .root()
        .join(s["evidence"]["identity"][0]["receipt"].as_str().unwrap());
    let mut receipt: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    receipt["report"]["results"]
        .as_array_mut()
        .unwrap()
        .retain(|r| r["target"] != "identity");
    fs::write(&path, serde_json::to_vec(&receipt).unwrap()).unwrap();
    let v = p.json(&["verify", "--json"]);
    assert_eq!(v["operation"]["reused_count"], 1);
    assert_eq!(p.count(), 2);
    let s = p.snapshot();
    let path = p.root().join(
        s["evidence"]["identity"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["receipt"]
            .as_str()
            .unwrap(),
    );
    fs::write(path, "broken json").unwrap();
    let v = p.json(&["verify", "identity", "--json"]);
    assert_eq!(v["operation"]["reused_count"], 0);
    assert_eq!(p.count(), 3);
}
#[test]
fn per_goal_native_commands_do_not_require_contracts_or_theorem_mapping() {
    let p = Project::new();
    let path = p.root().join("elicit.toml");
    let mut c: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    c["obligations"][1]["command"] =
        toml::Value::try_from(vec!["sh", "-c", "echo separate >> .elicit/count"]).unwrap();
    // Arbitrary user-declared statement/theorem strings are metadata, not admission criteria.
    c["obligations"][1].as_table_mut().unwrap().insert(
        "statement".into(),
        toml::Value::String("some arbitrary relational property".into()),
    );
    c["obligations"][1].as_table_mut().unwrap().insert(
        "theorem".into(),
        toml::Value::String("not inspected by elicit".into()),
    );
    fs::write(path, toml::to_string(&c).unwrap()).unwrap();
    p.ok(&["verify", "identity"]);
    assert_eq!(p.count(), 1);
    let v = p.json(&["verify", "--json"]);
    assert_eq!(v["operation"]["reused_count"], 1);
    assert_eq!(p.count(), 2);
}
#[test]
fn saved_status_is_pure_and_dependency_planning_remains_available() {
    let p = Project::new();
    fs::write(p.root().join("src/lib.rs"),"fn leaf(x:u32)->u32{x}\npub fn identity(x:u32)->u32{leaf(x)}\npub fn other(x:u32)->u32{leaf(x)}\n").unwrap();
    p.ok(&["status"]);
    let snapshot = fs::read(p.root().join(".elicit/snapshot.json")).unwrap();
    let history = fs::read(p.root().join(".elicit/history.jsonl")).unwrap();
    for args in [
        vec!["status", "--saved", "--json"],
        vec!["status", "identity", "--saved", "--json"],
        vec!["status", "--plan", "--saved", "--json"],
        vec!["status", "--inventory", "--saved", "--json"],
    ] {
        let o = p
            .cmd()
            .env("PATH", "/no-tools")
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    }
    assert_eq!(
        snapshot,
        fs::read(p.root().join(".elicit/snapshot.json")).unwrap()
    );
    assert_eq!(
        history,
        fs::read(p.root().join(".elicit/history.jsonl")).unwrap()
    );
    let plan = p.json(&["status", "--plan", "--saved", "--json"]);
    assert!(plan["components"].as_array().unwrap().iter().any(|c| {
        c["members"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("leaf"))
    }));
}
#[test]
fn timeout_and_missing_executable_remain_failures() {
    let p = Project::new();
    p.script("sleep 10\n");
    assert_eq!(p.call(&["verify", "--seconds", "1"]).status.code(), Some(1));
    let path = p.root().join("elicit.toml");
    let mut c: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    c["verification"]["command"] =
        toml::Value::try_from(vec!["/no/such/elicit-test-command"]).unwrap();
    fs::write(path, toml::to_string(&c).unwrap()).unwrap();
    assert_eq!(p.call(&["verify"]).status.code(), Some(2));
}
#[test]
fn declared_external_inputs_invalidate_saved_results() {
    let p = Project::new();
    let external = tempfile::tempdir().unwrap();
    let f = external.path().join("Library.lean");
    fs::write(&f, "-- original library\n").unwrap();
    let path = p.root().join("elicit.toml");
    let mut c: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    c["verification"]["inputs"] =
        toml::Value::try_from(vec![f.to_string_lossy().into_owned()]).unwrap();
    fs::write(path, toml::to_string(&c).unwrap()).unwrap();
    p.ok(&["verify"]);
    fs::write(f, "-- changed library\n").unwrap();
    let v = p.json(&["verify", "--json"]);
    assert_eq!(v["operation"]["reused_count"], 0);
    assert_eq!(p.count(), 2);
}
#[test]
fn no_command_does_not_turn_init_or_verify_into_successful_verification() {
    let p = Project::new();
    let path = p.root().join("elicit.toml");
    let mut c: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    c["verification"]["command"] = toml::Value::Array(vec![]);
    fs::write(path, toml::to_string(&c).unwrap()).unwrap();
    let v = p.json(&["status", "--json"]);
    assert!(
        v["goals"][0]["limitation"]
            .as_str()
            .unwrap()
            .contains("no verification command")
    );
    assert_eq!(p.call(&["verify"]).status.code(), Some(2));
}

#[test]
fn working_directory_and_literal_argv_are_preserved() {
    let p = Project::new();
    let directory = p.root().join("verification");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("verify.sh"),"test -f proof-input || exit 9\ntest \"$1\" = '$(touch SHOULD_NOT_EXIST)' || exit 8\necho run >> ../.elicit/count\n").unwrap();
    fs::write(directory.join("proof-input"), "native proof input").unwrap();
    let path = p.root().join("elicit.toml");
    let mut c: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    c["verification"].as_table_mut().unwrap().insert(
        "directory".into(),
        toml::Value::String("verification".into()),
    );
    c["verification"]["command"] =
        toml::Value::try_from(vec!["sh", "verify.sh", "$(touch SHOULD_NOT_EXIST)"]).unwrap();
    fs::write(path, toml::to_string(&c).unwrap()).unwrap();
    p.ok(&["verify"]);
    assert_eq!(p.count(), 1);
    assert!(!directory.join("SHOULD_NOT_EXIST").exists());
    fs::create_dir_all(directory.join(".lake/build")).unwrap();
    fs::write(
        directory.join(".lake/build/generated.olean"),
        "native output",
    )
    .unwrap();
    let reused = p.json(&["verify", "--json"]);
    assert_eq!(reused["operation"]["reused_count"], 2);
    assert_eq!(p.count(), 1);
}
#[test]
fn inherited_native_environment_changes_invalidate_saved_results() {
    let p = Project::new();
    let first = p
        .cmd()
        .env("CUSTOM_PROOF_SETTING", "one")
        .args(["verify", "--json"])
        .output()
        .unwrap();
    assert!(first.status.success());
    assert_eq!(p.count(), 1);
    let second = p
        .cmd()
        .env("CUSTOM_PROOF_SETTING", "two")
        .args(["verify", "--json"])
        .output()
        .unwrap();
    assert!(second.status.success());
    let second: Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(second["operation"]["reused_count"], 0);
    assert_eq!(p.count(), 2);
    // Only hashes, not the environment value, are stored in the input map.
    assert_ne!(
        p.snapshot()["input_files"]["@env:CUSTOM_PROOF_SETTING"],
        "two"
    );
}

#[test]
fn lean_edits_reuse_rust_analysis_but_stale_native_results() {
    let p = Project::new();
    fs::write(
        p.root().join("Proof.lean"),
        "theorem fact : True := True.intro\n",
    )
    .unwrap();
    p.ok(&["verify"]);
    assert!(p.snapshot()["analysis_reused"].as_bool().unwrap());
    fs::write(
        p.root().join("Proof.lean"),
        "-- new proof input\ntheorem fact : True := True.intro\n",
    )
    .unwrap();
    let status = p.json(&["status", "--json"]);
    assert_eq!(status["analysis_reused"], true);
    assert_eq!(status["summary"]["remaining"], 2);
    p.ok(&["verify"]);
    assert_eq!(
        p.count(),
        2,
        "proof edits must still rerun native validation"
    );
    fs::remove_file(p.root().join("Proof.lean")).unwrap();
    let status = p.json(&["status", "--json"]);
    assert_eq!(status["analysis_reused"], true);
    assert_eq!(status["summary"]["remaining"], 2);
}
#[test]
fn rust_edits_rebuild_call_graph_and_then_reuse_the_new_observation() {
    let p = Project::with_source(
        "fn leaf(x:u32)->u32{x}\npub fn identity(x:u32)->u32{leaf(x)}\npub fn other(x:u32)->u32{x}\n",
    );
    assert_eq!(p.snapshot()["analysis_reused"], false);
    let warm = p.json(&["status", "--json"]);
    assert_eq!(warm["analysis_reused"], true);
    fs::write(p.root().join("src/lib.rs"),"fn replacement(x:u32)->u32{x}\npub fn identity(x:u32)->u32{replacement(x)}\npub fn other(x:u32)->u32{x}\n").unwrap();
    p.ok(&["status"]);
    let snapshot = p.snapshot();
    assert_eq!(snapshot["analysis_reused"], false);
    let inventory = snapshot["inventory"].as_array().unwrap();
    let caller = inventory.iter().find(|t| t["name"] == "identity").unwrap();
    assert!(
        caller["calls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str().unwrap().contains("replacement"))
    );
    assert!(!inventory.iter().any(|t| t["name"] == "leaf"));
    assert_eq!(p.json(&["status", "--json"])["analysis_reused"], true);
}
#[test]
fn dependency_and_feature_changes_refresh_cached_cargo_scope() {
    let p = Project::new();
    let config_file = p.root().join("elicit.toml");
    let mut selected: toml::Value =
        toml::from_str(&fs::read_to_string(&config_file).unwrap()).unwrap();
    selected["package"]
        .as_table_mut()
        .unwrap()
        .insert("name".into(), toml::Value::String("fixture".into()));
    fs::write(config_file, toml::to_string(&selected).unwrap()).unwrap();
    let dependency = p.root().join("addon");
    fs::create_dir_all(dependency.join("src")).unwrap();
    fs::write(
        dependency.join("Cargo.toml"),
        "[package]\nname='addon'\nversion='0.0.0'\nedition='2021'\n",
    )
    .unwrap();
    fs::write(dependency.join("src/lib.rs"), "pub fn helper(){}\n").unwrap();
    let manifest = p.root().join("Cargo.toml");
    let old = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        format!("{old}\n[dependencies]\naddon={{path='addon'}}\n[features]\nextra=[]\n"),
    )
    .unwrap();
    p.ok(&["status"]);
    let snapshot = p.snapshot();
    assert_eq!(snapshot["analysis_reused"], false);
    assert!(
        snapshot["scope"]["local_roots"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().ends_with("/addon"))
    );
    p.ok(&["status"]);
    assert_eq!(p.snapshot()["analysis_reused"], true);
    fs::write(dependency.join("src/lib.rs"), "pub fn changed_helper(){}\n").unwrap();
    p.ok(&["status"]);
    assert_eq!(p.snapshot()["analysis_reused"], false);
    fs::write(p.root().join("src/lib.rs"),"pub fn identity(x:u32)->u32{x}\npub fn other(x:u32)->u32{x}\n#[cfg(feature=\"extra\")] pub fn gated(){}\n").unwrap();
    p.ok(&["status"]);
    let file = p.root().join("elicit.toml");
    let mut config: toml::Value = toml::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
    config["package"]["features"] = toml::Value::try_from(vec!["extra"]).unwrap();
    fs::write(file, toml::to_string(&config).unwrap()).unwrap();
    p.ok(&["status"]);
    let snapshot = p.snapshot();
    assert_eq!(snapshot["analysis_reused"], false);
    assert!(
        snapshot["inventory"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "gated")
    );
}
#[test]
fn build_scripts_and_file_reading_macros_keep_conservative_analysis_binding() {
    let p = Project::new();
    fs::write(p.root().join("build.rs"), "fn main(){}\n").unwrap();
    fs::write(p.root().join("Proof.lean"), "-- original\n").unwrap();
    p.ok(&["status"]);
    p.ok(&["status"]);
    assert_eq!(p.snapshot()["analysis_reused"], true);
    fs::write(p.root().join("Proof.lean"), "-- changed\n").unwrap();
    p.ok(&["status"]);
    assert_eq!(p.snapshot()["analysis_reused"], false);
    fs::remove_file(p.root().join("build.rs")).unwrap();
    fs::write(p.root().join("src/lib.rs"),"const DATA: &str = include_str!(\"../Proof.lean\");\npub fn identity(x:u32)->u32{x}\npub fn other(x:u32)->u32{x}\n").unwrap();
    p.ok(&["status"]);
    fs::write(p.root().join("Proof.lean"), "-- changed again\n").unwrap();
    p.ok(&["status"]);
    assert_eq!(p.snapshot()["analysis_reused"], false);
}
#[test]
fn analysis_cache_corruption_or_deletion_falls_back_without_losing_receipts() {
    let p = Project::new();
    p.ok(&["verify"]);
    let path = p.root().join(".elicit/cache/analysis.json");
    let mut cache: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    cache["payload"]["inventory"] = serde_json::json!([]);
    fs::write(&path, serde_json::to_vec(&cache).unwrap()).unwrap();
    p.ok(&["status"]);
    assert_eq!(p.snapshot()["analysis_reused"], false);
    assert!(!p.snapshot()["inventory"].as_array().unwrap().is_empty());
    fs::remove_file(&path).unwrap();
    let verified = p.json(&["verify", "--json"]);
    assert_eq!(verified["operation"]["reused_count"], 2);
    assert_eq!(p.count(), 1);
}
#[test]
fn workspace_glob_additions_invalidate_cached_metadata() {
    let p = Project::new();
    let file = p.root().join("elicit.toml");
    let mut config: toml::Value = toml::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
    config["package"]
        .as_table_mut()
        .unwrap()
        .insert("name".into(), toml::Value::String("fixture".into()));
    fs::write(file, toml::to_string(&config).unwrap()).unwrap();
    let member = p.root().join("members/one");
    fs::create_dir_all(member.join("src")).unwrap();
    fs::write(
        member.join("Cargo.toml"),
        "[package]\nname='one'\nversion='0.0.0'\nedition='2021'\n",
    )
    .unwrap();
    fs::write(member.join("src/lib.rs"), "").unwrap();
    let manifest = p.root().join("Cargo.toml");
    let source = fs::read_to_string(&manifest)
        .unwrap()
        .replace("[workspace]", "[workspace]\nmembers=['members/*']");
    fs::write(&manifest, source).unwrap();
    p.ok(&["status"]);
    p.ok(&["status"]);
    assert_eq!(p.snapshot()["analysis_reused"], true);
    let member = p.root().join("members/two");
    fs::create_dir_all(member.join("src")).unwrap();
    fs::write(
        member.join("Cargo.toml"),
        "[package]\nname='two'\nversion='0.0.0'\nedition='2021'\n",
    )
    .unwrap();
    fs::write(member.join("src/lib.rs"), "").unwrap();
    p.ok(&["status"]);
    let snapshot = p.snapshot();
    assert_eq!(snapshot["analysis_reused"], false);
    assert!(
        snapshot["scope"]["local_roots"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().ends_with("members/two"))
    );
}

#[cfg(unix)]
#[test]
fn sources_without_definition_queries_do_not_start_rust_analyzer() {
    use std::os::unix::fs::PermissionsExt;
    let p = Project::new();
    let bin = p.root().join("bin");
    fs::create_dir(&bin).unwrap();
    let executable = bin.join("rust-analyzer");
    fs::write(
        &executable,
        "#!/bin/sh\necho called >> .elicit/analyzer-called\nexit 73\n",
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    let result = p
        .cmd()
        .env(
            "PATH",
            format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
        )
        .args(["status", "--json"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!p.root().join(".elicit/analyzer-called").exists());
    assert_eq!(
        p.snapshot()["analysis_reused"],
        false,
        "tool identity changes must invalidate the cache even when no language server is needed"
    );
}
#[test]
fn nonstandard_rust_module_paths_are_not_mistaken_for_independent_lean_files() {
    let p = Project::new();
    fs::write(p.root().join("src/Support.lean"), "pub fn before(){}\n").unwrap();
    fs::write(p.root().join("src/lib.rs"),"#[path=\"Support.lean\"] mod support;\npub fn identity(x:u32)->u32{x}\npub fn other(x:u32)->u32{x}\n").unwrap();
    p.ok(&["status"]);
    fs::write(p.root().join("src/Support.lean"), "pub fn after(){}\n").unwrap();
    p.ok(&["status"]);
    let snapshot = p.snapshot();
    assert_eq!(snapshot["analysis_reused"], false);
    assert!(
        snapshot["inventory"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "after")
    );
}

#[test]
fn synthetic_environment_keys_are_never_treated_as_source_paths() {
    let p = Project::new();
    let result = p
        .cmd()
        .env("CUSTOM_INPUT.rs", "value")
        .args(["status", "--json"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(p.snapshot()["analysis_reused"], false);
    assert!(p.snapshot()["input_files"]["@env:CUSTOM_INPUT.rs"].is_string());
}
