//! Opt-in native tool smoke checks. No model calls or full-crate authoring run.
use serde_json::Value;
use std::{fs, process::Command};
#[test]
#[ignore = "requires installed Lean; no live model"]
fn native_lean_policy_is_owned_by_the_configured_command() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='native_fixture'\nversion='0.0.0'\nedition='2021'\n[workspace]\n",
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn identity(x:u32)->u32{x}\n").unwrap();
    // Restricted input, unrelated proposition and a custom axiom are all native
    // Lean policy. elicit neither rewrites the theorem nor rejects its axioms.
    fs::write(root.join("Proofs.lean"),"axiom user_assumption : False\ntheorem unrelated : 2 + 2 = 4 := rfl\ntheorem unfinished : False := by sorry\ntheorem from_assumption : False := user_assumption\n").unwrap();
    let call = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cargo-elicit"))
            .current_dir(root)
            .args(args)
            .output()
            .unwrap()
    };
    let init = call(&[
        "init",
        "--api",
        "identity",
        "--command",
        "lean",
        "Proofs.lean",
    ]);
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let verified = call(&["verify", "--json"]);
    assert!(
        verified.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&verified.stdout),
        String::from_utf8_lossy(&verified.stderr)
    );
    let v: Value = serde_json::from_slice(&verified.stdout).unwrap();
    assert_eq!(v["operation"]["passed"], 1);
    let receipt: Value = serde_json::from_slice(
        &fs::read(root.join(v["operation"]["results"][0]["receipt"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["report"]["results"][0]["exit_code"], 0);
    let reused: Value = serde_json::from_slice(&call(&["verify", "--json"]).stdout).unwrap();
    assert_eq!(reused["operation"]["reused_count"], 1);
    fs::write(
        root.join("Proofs.lean"),
        "theorem broken : False := by exact True.intro\n",
    )
    .unwrap();
    let failed = call(&["verify", "--json"]);
    assert_eq!(failed.status.code(), Some(1));
}
