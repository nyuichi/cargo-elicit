//! Opt-in checks use real native verifiers; no AI client or credentials exist.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};
fn command(root: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cargo-elicit"));
    cmd.current_dir(root);
    cmd
}
fn invoke(root: &Path, args: &[&str], code: i32) -> Value {
    let started = Instant::now();
    let output = command(root).args(args).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(code),
        "{args:?}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let directory = root.join(".elicit/native-validation");
    fs::create_dir_all(&directory).unwrap();
    let number = fs::read_dir(&directory).unwrap().count();
    fs::write(directory.join(format!("{number:02}.json")),serde_json::to_vec_pretty(&serde_json::json!({"args":args,"code":code,"seconds":started.elapsed().as_secs_f64(),"stdout":String::from_utf8_lossy(&output.stdout),"stderr":String::from_utf8_lossy(&output.stderr)})).unwrap()).unwrap();
    if args.contains(&"--json") {
        serde_json::from_slice(&output.stdout).unwrap()
    } else {
        Value::Null
    }
}
fn fixture(name: &str) -> PathBuf {
    let root = tempfile::Builder::new()
        .prefix("elicit-independent-native-")
        .tempdir()
        .unwrap()
        .keep();
    let original = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    for entry in walkdir::WalkDir::new(&original)
        .into_iter()
        .filter_entry(|e| !matches!(e.file_name().to_str(), Some("target" | ".elicit")))
    {
        let entry = entry.unwrap();
        let target = root.join(entry.path().strip_prefix(&original).unwrap());
        if entry.file_type().is_dir() {
            fs::create_dir_all(target).unwrap();
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
    root
}
fn stages(root: &Path) -> Vec<Value> {
    fs::read_to_string(root.join(".elicit/events.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|e| e["event"] == "pipeline_stage_finished")
        .map(|e| e["data"].clone())
        .collect()
}
#[test]
#[ignore = "requires pinned Charon/Aeneas and Lean 4.35.0; no live model"]
fn aeneas_incremental_audit_and_independent_semantic_review() {
    let root = fixture("aeneas");
    println!("validation_root={}", root.display());
    let proof = root.join("elicit-proofs/aeneas");
    fs::create_dir(proof.join("Models")).unwrap();
    fs::rename(proof.join("Specs.lean"), proof.join("Models/Identity.lean")).unwrap();
    fs::write(proof.join("Specs.lean"), "import Models.Identity\n").unwrap();
    let file = proof.join("Proofs.lean");
    let original = fs::read_to_string(&file).unwrap();
    fs::write(
        &file,
        format!("{original}\n theorem Proofs.fact : True := True.intro\n"),
    )
    .unwrap();
    invoke(&root, &["init", "--api", "identity"], 0);
    let config = root.join("elicit.toml");
    fs::write(
        &config,
        format!(
            "{}\n[[obligations]]\nid=\"fact\"\nstatement=\"True\"\ntheorem=\"Proofs.fact\"\n",
            fs::read_to_string(&config).unwrap()
        ),
    )
    .unwrap();
    invoke(&root, &["audit", "--json"], 0);
    let status = invoke(&root, &["status", "--json"], 0);
    assert_eq!(status["declared_obligations_audited_at_snapshot"], true);
    assert!(
        status["goals"]
            .as_array()
            .unwrap()
            .iter()
            .all(|g| g["review"] == "unreviewed")
    );
    invoke(&root, &["check", "--json"], 0);
    let count = stages(&root).len();
    invoke(&root, &["check", "--json"], 0);
    assert!(
        stages(&root)[count..]
            .iter()
            .any(|stage| stage["stage"] == "Proofs.lean" && stage["cache_hit"] == true)
    );
    assert!(
        stages(&root)[count..]
            .iter()
            .any(|stage| stage["stage"] == "Audit.lean" && stage["cache_hit"] == false)
    );
    let count = stages(&root).len();
    fs::write(
        &file,
        format!(
            "{}\n-- proof-only edit\n",
            fs::read_to_string(&file).unwrap()
        ),
    )
    .unwrap();
    invoke(&root, &["check", "--json"], 0);
    assert!(
        stages(&root)[count..]
            .iter()
            .any(|stage| stage["stage"] == "Identity.lean" && stage["cache_hit"] == true)
    );
    invoke(
        &root,
        &[
            "review",
            "--note",
            "Identity model and arbitrary-input contract match the requested behavior",
            "--json",
        ],
        0,
    );
    let model = proof.join("Models/Identity.lean");
    fs::write(
        &model,
        fs::read_to_string(&model)
            .unwrap()
            .replace(":= x", ":= x + 1"),
    )
    .unwrap();
    let refreshed = invoke(&root, &["refresh", "--json"], 0);
    assert!(
        refreshed["goals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["id"] == "identity"
                && g["review"] == "stale"
                && g["audit"] == "missing_or_stale")
    );
    fs::write(
        &model,
        fs::read_to_string(&model)
            .unwrap()
            .replace(":= x + 1", ":= x"),
    )
    .unwrap();
    let proposition = "ElicitContracts.c_a944f8e645077fd5";
    fs::write(&file,format!("import ElicitContracts\naxiom projectAssumption : {proposition}\nnamespace Proofs\ntheorem identity_spec : {proposition} := projectAssumption\ntheorem fact : True := True.intro\nend Proofs\n")).unwrap();
    let rejected = invoke(&root, &["check", "--json"], 1);
    assert!(
        rejected["goals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["id"] == "identity" && g["check"] == "failed")
    );
    assert!(
        rejected["goals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g["id"] == "fact" && g["check"] == "accepted")
    );
    fs::write(
        &file,
        format!("{original}\n theorem Proofs.fact : True := True.intro\n"),
    )
    .unwrap();
    let count = stages(&root).len();
    invoke(&root, &["audit", "--json"], 0);
    assert!(
        stages(&root)[count..]
            .iter()
            .all(|e| e["cache_hit"] == false)
    );
    let snapshot: Value =
        serde_json::from_slice(&fs::read(root.join(".elicit/snapshot.json")).unwrap()).unwrap();
    let receipt = snapshot["evidence"]["identity"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()["receipt"]
        .as_str()
        .unwrap()
        .to_string();
    let evidence: Value = serde_json::from_slice(&fs::read(root.join(&receipt)).unwrap()).unwrap();
    let logs = root
        .join(evidence["report"]["cache"].as_str().unwrap())
        .join("logs/Audit.log");
    assert!(logs.is_file());
    fs::remove_dir_all(root.join(".elicit/cache")).unwrap();
    invoke(&root, &["status", "--json"], 0);
    assert!(logs.is_file());
}
#[test]
#[ignore = "requires native Kani 0.68 toolchain"]
fn kani_original_contract_audit() {
    let root = fixture("kani");
    invoke(&root, &["init", "--tool", "kani", "--api", "identity"], 0);
    invoke(&root, &["audit", "--json"], 0);
}
#[test]
#[ignore = "requires native Creusot 0.14 with Why3/solvers"]
fn creusot_original_contract_audit() {
    let root = fixture("creusot");
    invoke(
        &root,
        &["init", "--tool", "creusot", "--api", "identity"],
        0,
    );
    invoke(&root, &["audit", "--json"], 0);
}
