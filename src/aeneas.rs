//! Pinned Charon → Aeneas → Lean 4 adapter. Translation is never proof evidence.
use crate::{fingerprint, model::*, process, verifier};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

pub const AENEAS_COMMIT: &str = "e62dd83d133d9043e7d3628096838b7430f36565";
pub const CHARON_COMMIT: &str = "de8736f7c995ac54dc490345cef9ea5143c06c62";
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    #[serde(default)]
    contracts: BTreeMap<String, Contract>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    #[serde(default)]
    requires: Option<String>,
    ensures: String,
    #[serde(default)]
    theorem: Option<String>,
}
fn proof_root(scope: &Scope) -> PathBuf {
    Path::new(&scope.manifest_path)
        .parent()
        .unwrap()
        .join("elicit-proofs/aeneas")
}
fn manifest(scope: &Scope) -> Result<Manifest> {
    let p = proof_root(scope).join("contracts.toml");
    if !p.exists() {
        return Ok(Manifest::default());
    }
    let m: Manifest =
        toml::from_str(&fs::read_to_string(p)?).context("invalid Aeneas contracts.toml")?;
    if m.schema_version != 1 {
        bail!("unsupported Aeneas contracts schema; expected 1");
    }
    Ok(m)
}
pub fn contract_name(id: &str) -> String {
    format!("ElicitContracts.c_{}", &fingerprint::hash(id)[..16])
}
pub fn check_submission(scope: &Scope, t: &Target) -> Result<()> {
    if t.kind.proof() {
        let m = manifest(scope)?;
        let c = m
            .contracts
            .get(&t.id)
            .context("Aeneas contract missing from elicit-proofs/aeneas/contracts.toml")?;
        if t.kind != Kind::Function {
            bail!("Aeneas method/impl contracts require unsupported scope; record an agent issue");
        }
        proposition(t, c)?;
    }
    Ok(())
}
pub fn attach_specs(scope: &Scope, targets: &mut [Target]) -> Result<()> {
    let m = manifest(scope)?;
    let models = fs::read(proof_root(scope).join("Specs.lean")).unwrap_or_default();
    for t in targets {
        let c = m.contracts.get(&t.id);
        // Proof terms and theorem names affect proof freshness, not contract staleness.
        t.spec_hash = fingerprint::hash(serde_json::to_vec(&(
            &t.spec_hash,
            c.map(|c| (&c.requires, &c.ensures)),
            &models,
        ))?);
    }
    Ok(())
}
pub fn home() -> Result<PathBuf> {
    let p = std::env::var_os("AENEAS_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            fingerprint::executable("aeneas")?
                .parent()?
                .parent()
                .map(Path::to_path_buf)
        })
        .context("set AENEAS_HOME to the pinned Aeneas checkout")?;
    if !p.join("backends/lean/lakefile.lean").is_file() {
        bail!("AENEAS_HOME lacks backends/lean/lakefile.lean");
    }
    Ok(p)
}
fn lake() -> Result<Command> {
    let backend = home()?.join("backends/lean");
    if !backend.join(".lake/build/lib/lean/Aeneas.olean").is_file() {
        bail!(
            "Aeneas Lean library is not built; run lake build in AENEAS_HOME/backends/lean first"
        );
    }
    let dependencies: Value =
        serde_json::from_slice(&fs::read(backend.join("lake-manifest.json"))?)?;
    for package in dependencies["packages"]
        .as_array()
        .context("invalid Aeneas Lake manifest")?
    {
        let name = package["name"]
            .as_str()
            .context("invalid Lake dependency name")?;
        if !backend.join(".lake/packages").join(name).is_dir() {
            bail!(
                "missing Lean dependency {name}; prepare the Aeneas backend with lake build first"
            );
        }
    }
    let mut c = Command::new("lake");
    c.current_dir(backend);
    Ok(c)
}
pub fn version() -> Result<String> {
    let a = fingerprint::version("aeneas", &["-version"])?;
    let c = fingerprint::version("charon", &["version"])?;
    let mut cmd = lake()?;
    cmd.args(["env", "lean", "--version"]);
    let l = process::run(&mut cmd, Duration::from_secs(30))?;
    if !a.contains(&AENEAS_COMMIT[..7])
        || !c.contains(&CHARON_COMMIT[..7])
        || !c.contains("0.1.293")
        || l.code != Some(0)
        || !l.stdout.contains("4.35.0")
    {
        bail!(
            "unsupported Aeneas toolchain; requires Aeneas {AENEAS_COMMIT}, Charon {CHARON_COMMIT} (0.1.293), Lean 4.35.0"
        );
    }
    Ok(format!("{a}; Charon {c}; {}", l.stdout.trim()))
}
pub fn tcb_file(path: &Path) -> bool {
    let n = path.file_name().unwrap_or_default().to_string_lossy();
    n.ends_with(".lean")
        || n.contains(".olean")
        || n.contains(".so")
        || n.ends_with(".dylib")
        || n.ends_with(".dll")
        || matches!(
            n.as_ref(),
            "lean-toolchain" | "lake-manifest.json" | "charon-pin"
        )
        || path.parent().is_some_and(|p| p.ends_with("bin"))
}
pub fn tcb_roots() -> Result<Vec<PathBuf>> {
    let Ok(h) = home() else {
        return Ok(vec![]);
    };
    let mut roots = vec![h.join("backends/lean"), h.join("charon-pin")];
    let mut c = lake()?;
    c.args(["env", "lean", "--print-prefix"]);
    let o = process::run(&mut c, Duration::from_secs(30))?;
    if o.code == Some(0) {
        let p = PathBuf::from(o.stdout.trim());
        if p.is_absolute() {
            roots.extend([p.join("bin"), p.join("lib"), p.join("src")]);
        }
    }
    if let Ok(p) = fingerprint::version("charon", &["toolchain-path"]) {
        let p = PathBuf::from(p.trim());
        if p.is_absolute() {
            roots.extend([p.join("bin"), p.join("lib")]);
        }
    }
    if let Some(p) = fingerprint::executable("charon-driver") {
        roots.push(p);
    }
    Ok(roots)
}
fn lean_type(t: &syn::Type) -> Option<String> {
    match t {
        syn::Type::Tuple(t) if t.elems.is_empty() => Some("Unit".into()),
        syn::Type::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => {
            let n = p.path.segments[0].ident.to_string();
            Some(match n.as_str() {
                "bool" => "Bool".into(),
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" => {
                    n.to_uppercase()
                }
                _ => return None,
            })
        }
        _ => None,
    }
}
fn proposition(t: &Target, c: &Contract) -> Result<String> {
    let f = verifier::original_fn(t)?;
    let types = f
        .sig
        .inputs
        .iter()
        .map(|a| {
            let syn::FnArg::Typed(a) = a else {
                bail!("methods are not supported");
            };
            lean_type(&a.ty).context("unsupported Aeneas argument type")
        })
        .collect::<Result<Vec<_>>>()?;
    let ret = match &f.sig.output {
        syn::ReturnType::Default => "Unit".into(),
        syn::ReturnType::Type(_, t) => lean_type(t).context("unsupported Aeneas return type")?,
    };
    let names = (0..types.len())
        .map(|i| format!("arg{i}"))
        .collect::<Vec<_>>();
    let binders = names
        .iter()
        .zip(&types)
        .map(|(n, t)| format!("({n} : {t})"))
        .collect::<Vec<_>>()
        .join(" ");
    let args = names.join(" ");
    let pred_ty = types.iter().map(|t| format!("{t} → ")).collect::<String>();
    let pre = c
        .requires
        .as_ref()
        .map(|p| format!("(({p}) : {pred_ty}Prop) {args}"))
        .unwrap_or_else(|| "True".into());
    let path =
        t.id.split("::")
            .skip(1)
            .take_while(|v| *v != "Function")
            .map(|n| format!("«{n}»"))
            .collect::<Vec<_>>()
            .join(".");
    if path.is_empty() || c.ensures.trim().is_empty() {
        bail!("missing Aeneas postcondition");
    }
    let forall = if binders.is_empty() {
        String::new()
    } else {
        format!("∀ {binders}, ")
    };
    Ok(format!(
        "def {} : Prop :=\n  {forall}({pre}) → ∃ (result : {ret}), ElicitGenerated.{path} {args} = .ok result ∧ ((({}) : {pred_ty}{ret} → Prop) {args} result)\n",
        contract_name(&t.id),
        c.ensures
    ))
}
fn extracted(s: &State, t: &Target, llbc: &Value) -> bool {
    let tr = &llbc["translated"];
    if llbc["charon_version"] != "0.1.293"
        || llbc["has_errors"] != false
        || tr["runtime_checks"]["overflow_checks"] != true
        || !tr["target_information"]
            .as_array()
            .is_some_and(|ts| ts.len() == 1 && ts[0]["key"] == s.scope.target)
    {
        return false;
    }
    let expected = t.id.trim_end_matches("::Function").replace('-', "_");
    let Some(fs) = tr["fun_decls"].as_array() else {
        return false;
    };
    fs.iter().any(|f| {
        let m = &f["item_meta"];
        let name = m["name"]
            .as_array()
            .and_then(|parts| {
                parts
                    .iter()
                    .map(|p| {
                        let i = p["Ident"].as_array()?;
                        if i.len() != 2 || i[1] != 0 {
                            return None;
                        }
                        i[0].as_str()
                    })
                    .collect::<Option<Vec<_>>>()
            })
            .map(|parts| parts.join("::"));
        if name.as_deref() != Some(&expected)
            || m["is_local"] != true
            || m["is_extern"] != false
            || m["has_errors"] != false
            || m["opacity"] != "Transparent"
            || !f["body"]["Structured"].is_object()
        {
            return false;
        }
        let span = &m["span"]["Untagged"]["data"];
        let Some(file) = tr["files"]
            .as_array()
            .and_then(|files| files.iter().find(|f| f["id"] == span["file_id"]))
        else {
            return false;
        };
        let Some(path) = file["name"]["Local"].as_str() else {
            return false;
        };
        let root = Path::new(&s.scope.manifest_path).parent().unwrap();
        root.join(path).canonicalize().ok() == Path::new(&t.file).canonicalize().ok()
            && file["contents"]
                .as_str()
                .is_some_and(|c| fs::read_to_string(&t.file).is_ok_and(|source| source == c))
            && span["beg"]["line"]
                .as_u64()
                .is_some_and(|line| line <= t.line as u64)
            && span["end"]["line"]
                .as_u64()
                .is_some_and(|line| line >= t.line as u64)
    })
}
fn mapped(s: &State, t: &Target, text: &str) -> bool {
    if t.kind != Kind::Function {
        return false;
    }
    let rust_name = t.id.trim_end_matches("::Function").replace('-', "_");
    let marker = format!("[{}]:", rust_name);
    let Some((_, tail)) = text.split_once(&marker) else {
        return false;
    };
    let Some((doc, body)) = tail.split_once("-/") else {
        return false;
    };
    let root = Path::new(&s.scope.manifest_path).parent().unwrap();
    let location = doc.lines().find_map(|l| l.trim().strip_prefix("Source: '"));
    let Some((file, lines)) = location.and_then(|l| l.split_once("', lines ")) else {
        return false;
    };
    let Some(start) = lines
        .split(':')
        .next()
        .and_then(|v| v.parse::<usize>().ok())
    else {
        return false;
    };
    let end = lines
        .split('-')
        .nth(1)
        .and_then(|v| v.split(':').next())
        .and_then(|v| v.parse::<usize>().ok());
    root.join(file).canonicalize().ok() == Path::new(&t.file).canonicalize().ok()
        && start <= t.line
        && end.is_some_and(|end| t.line <= end)
        && body.trim_start().starts_with(&format!("def {} ", t.name))
}
fn lean_command(out: &Path, module: &str) -> Result<Command> {
    let mut c = lake()?;
    c.args(["env", "lean"])
        .arg(format!("--root={}", out.display()))
        .arg("-o")
        .arg(out.join(format!("{module}.olean")))
        .arg(out.join(format!("{module}.lean")));
    // Modules can only be loaded from this run's fresh cache, never old proof objects.
    c.env("LEAN_PATH", out);
    Ok(c)
}
pub fn execute(
    s: &State,
    out: &Path,
    prove: bool,
    seconds: u64,
) -> Result<(process::Output, Option<Value>)> {
    let start = Instant::now();
    let run = |cmd: &mut Command| {
        process::run(
            cmd,
            Duration::from_secs(seconds.saturating_sub(start.elapsed().as_secs())),
        )
    };
    let mut cmd = Command::new("charon");
    cmd.args([
        "cargo",
        "--preset=aeneas",
        "--sysroot=default",
        "--format=json",
        "--no-dedup-serialized-ast",
        "--dest-file",
    ])
    .arg(out.join("crate.llbc"))
    .arg("--")
    .args(s.scope.cargo_args())
    .env("CARGO_TARGET_DIR", out.join("rust-build"))
    .current_dir(Path::new(&s.scope.manifest_path).parent().unwrap());
    let o = run(&mut cmd)?;
    if o.code != Some(0) {
        return Ok((o, None));
    }
    let mut cmd = Command::new("aeneas");
    cmd.args(["-backend", "lean", "-namespace", "ElicitGenerated", "-dest"])
        .arg(out)
        .arg(out.join("crate.llbc"));
    let o = run(&mut cmd)?;
    if o.code != Some(0) {
        return Ok((o, None));
    }
    let files = fs::read_dir(out)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "lean"))
        .collect::<Vec<_>>();
    if files.len() != 1 {
        bail!("expected one fresh Aeneas Lean translation");
    }
    let translated = fs::read_to_string(&files[0])?;
    let llbc: Value = serde_json::from_slice(&fs::read(out.join("crate.llbc"))?)?;
    fs::rename(&files[0], out.join("ElicitGenerated.lean"))?;
    let mut o = run(&mut lean_command(out, "ElicitGenerated")?)?;
    if o.code != Some(0) {
        return Ok((o, None));
    }
    let m = manifest(&s.scope)?;
    let root = proof_root(&s.scope);
    let mut contracts = "import ElicitGenerated\nopen Aeneas Aeneas.Std Result\n".to_string();
    if root.join("Specs.lean").exists() {
        fs::copy(root.join("Specs.lean"), out.join("Specs.lean"))?;
        o = run(&mut lean_command(out, "Specs")?)?;
        if o.code != Some(0) {
            return Ok((o, Some(json!({"phase":"contracts"}))));
        }
        contracts.insert_str(0, "import Specs\n");
    }
    let mut eligible = BTreeMap::new();
    for t in s.targets.values().filter(|t| !t.removed) {
        if let Some(c) = m.contracts.get(&t.id) {
            if mapped(s, t, &translated) && extracted(s, t, &llbc) {
                match proposition(t, c) {
                    Ok(p) => {
                        contracts.push_str(&p);
                    }
                    Err(e) if !prove => return Err(e),
                    Err(_) => continue,
                }
                let full = t.closed_primitive
                    && !t.trusted
                    && t.calls.is_empty()
                    && t.unresolved_calls.is_empty()
                    && verifier::safe_project(s)?;
                eligible.insert(t.id.clone(), (c, full));
            } else if !prove && t.kind.proof() {
                bail!(
                    "Aeneas contract has no fresh original-function mapping: {}",
                    t.id
                );
            }
        }
    }
    fs::write(out.join("ElicitContracts.lean"), contracts)?;
    o = run(&mut lean_command(out, "ElicitContracts")?)?;
    if o.code != Some(0) || !prove {
        return Ok((o, Some(json!({"phase":"contracts"}))));
    }
    if !root.join("Proofs.lean").exists() {
        return Ok((o, Some(json!({"phase":"translation-only"}))));
    }
    fs::copy(root.join("Proofs.lean"), out.join("Proofs.lean"))?;
    o = run(&mut lean_command(out, "Proofs")?)?;
    if o.code != Some(0) {
        return Ok((o, Some(json!({"phase":"proofs"}))));
    }
    let mut audits = BTreeMap::new();
    for (id, (c, full)) in eligible {
        let Some(theorem) = &c.theorem else { continue };
        if !theorem
            .split('.')
            .all(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        {
            bail!("invalid Lean theorem name");
        }
        let name = contract_name(&id);
        let source = format!(
            "import Proofs\ntheorem ElicitAudit : {name} := {theorem}\n#print axioms ElicitAudit\n"
        );
        fs::write(out.join("Audit.lean"), source)?;
        o = run(&mut lean_command(out, "Audit")?)?;
        if o.timeout || o.interrupted {
            return Ok((o, Some(json!({"phase":"audit"}))));
        }
        audits.insert(
            id,
            json!({"checked":o.code == Some(0),"full":full,"axioms":o.stdout}),
        );
    }
    // An individual rejected theorem does not discard independent successful proofs.
    o.code = Some(0);
    Ok((o, Some(json!({"phase":"audit","targets":audits}))))
}
fn standard_axioms(text: &str) -> bool {
    let lines = text
        .lines()
        .filter(|l| l.starts_with("'ElicitAudit' "))
        .collect::<Vec<_>>();
    if lines.len() != 1 {
        return false;
    }
    let l = lines[0];
    if l == "'ElicitAudit' does not depend on any axioms" {
        return true;
    }
    let Some(xs) = l
        .strip_prefix("'ElicitAudit' depends on axioms: [")
        .and_then(|v| v.strip_suffix(']'))
    else {
        return false;
    };
    xs.split(',')
        .all(|x| matches!(x.trim(), "propext" | "Classical.choice" | "Quot.sound"))
}
pub fn evidence(_s: &State, t: &Target, e: Option<&Value>, r: &mut Verification) -> Result<()> {
    r.trusted_base = vec![
        format!("Charon {CHARON_COMMIT} extraction and Aeneas {AENEAS_COMMIT} translation"),
        "Lean 4.35.0 kernel, standard axioms and pinned Aeneas Rust models".into(),
    ];
    let audit = e.map(|e| &e["targets"][&t.id]);
    if let Some(a) = audit.filter(|a| a["checked"] == true) {
        r.obligations.push(contract_name(&t.id));
        if a["axioms"].as_str().is_some_and(standard_axioms) && a["full"] != true {
            r.scope = Coverage::Limited;
            r.restrictions
                .push("generic, helper, external model or other unconfirmed Rust scope".into());
            r.summary = "kernel-checked theorem retained with unconfirmed full API coverage".into();
        } else if a["axioms"].as_str().is_some_and(standard_axioms) {
            r.scope = Coverage::FullContract;
            r.outcome = Outcome::Success;
            r.reusable = fingerprint::tcb_tracked(Tool::Aeneas);
            r.summary = "Lean kernel checked the original function contract for all primitive inputs; standard axioms only".into();
        } else {
            r.outcome = Outcome::Unresolved;
            r.assumptions
                .push("Lean proof depends on sorry, project axioms or unrecognized axioms".into());
            r.summary =
                "Lean axiom audit did not establish an assumption-free contract proof".into();
        }
    } else {
        r.outcome = Outcome::Unresolved;
        r.summary =
            "no kernel-checked theorem mapped to the complete original Rust contract".into();
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axiom_audit_excludes_admitted_and_project_proofs() {
        assert!(standard_axioms(
            "'ElicitAudit' does not depend on any axioms\n"
        ));
        assert!(standard_axioms(
            "'ElicitAudit' depends on axioms: [propext, Classical.choice, Quot.sound]"
        ));
        for s in [
            "",
            "'ElicitAudit' depends on axioms: [sorryAx]",
            "'ElicitAudit' depends on axioms: [projectAssumption]",
            "'ElicitAudit' depends on axioms: [Lean.ofReduceBool]",
        ] {
            assert!(!standard_axioms(s));
        }
    }
}
