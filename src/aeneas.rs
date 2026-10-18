//! Pinned Charon → Aeneas → Lean 4 adapter. Translation is never proof evidence.
use crate::{fingerprint, model::*, process};
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
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchAudit {
    index: usize,
    checked: bool,
    axioms: Option<Vec<String>>,
}
impl BatchAudit {
    fn accepted(&self) -> bool {
        self.checked
            && self.axioms.as_ref().is_some_and(|xs| {
                xs.iter()
                    .all(|x| matches!(x.as_str(), "propext" | "Classical.choice" | "Quot.sound"))
            })
    }
    fn axiom_report(&self) -> String {
        match &self.axioms {
            Some(xs) if xs.is_empty() => "'ElicitAudit' does not depend on any axioms".into(),
            Some(xs) => format!("'ElicitAudit' depends on axioms: [{}]", xs.join(", ")),
            None => String::new(),
        }
    }
}
fn parse_batch_audits(text: &str, count: usize) -> Result<Vec<BatchAudit>> {
    let mut rows = BTreeMap::new();
    for line in text.lines().filter_map(|l| l.strip_prefix("ELICIT_AUDIT=")) {
        let row: BatchAudit = serde_json::from_str(line).context("malformed batch audit row")?;
        if row.index >= count || rows.insert(row.index, row).is_some() {
            bail!("duplicate or unexpected batch audit index");
        }
    }
    if rows.len() != count {
        bail!("incomplete batch audit output");
    }
    Ok(rows.into_values().collect())
}
fn batch_audit_source(module: &str, roots: &[(&str, &str)]) -> Result<String> {
    let mut source = format!(
        "import {module}\n{}\nopen Aeneas Aeneas.Std Result ElicitGenerated\n",
        include_str!("lean/ElicitAudit.lean")
    );
    for (index, (statement, theorem)) in roots.iter().enumerate() {
        // Parse propositions inside the trusted command: malformed syntax must
        // fail one root, never inject another Lean command into the batch.
        source.push_str(&format!(
            "elicit_audit {index} {} {}\n",
            serde_json::to_string(statement)?,
            serde_json::to_string(theorem)?
        ));
    }
    Ok(source)
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
// Collect existing declared proofs before opening unrelated specification work.
// Names are only candidates: fresh extraction, kernel checks and axiom audits
// remain mandatory, including for previously successful roots after proof edits.
pub fn prepared_targets(s: &State) -> Result<Vec<String>> {
    if !proof_root(&s.scope).join("Proofs.lean").is_file() {
        return Ok(vec![]);
    }
    let m = manifest(&s.scope)?;
    Ok(m.contracts
        .iter()
        .filter(|(id, c)| {
            c.theorem
                .as_ref()
                .is_some_and(|name| !name.trim().is_empty())
                && s.targets.contains_key(*id)
                && s.schedulable(id)
                && s.dependencies_ready(id)
                && s.targets[*id].kind.proof()
                && !s.verified(id)
                && !s
                    .results
                    .iter()
                    .any(|r| &r.target == *id && r.input == s.input)
        })
        .map(|(id, _)| id.clone())
        .collect())
}
pub fn check_submission(scope: &Scope, t: &Target) -> Result<()> {
    if t.kind.proof() {
        let m = manifest(scope)?;
        let c = m
            .contracts
            .get(&t.id)
            .context("Aeneas contract missing from elicit-proofs/aeneas/contracts.toml")?;
        if !matches!(t.kind, Kind::Function | Kind::Method) {
            bail!(
                "unsupported Aeneas trait implementation mapping; this target remains blocked even when explicit method obligations are proved"
            );
        }
        if c.ensures.trim().is_empty() {
            bail!("missing Aeneas postcondition");
        }
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
struct ModelInputs {
    models: BTreeMap<String, String>,
    contracts: BTreeMap<String, BTreeMap<String, String>>,
}
#[derive(Serialize, Deserialize)]
struct ModelCache {
    key: String,
    inputs: Option<ModelInputs>,
}
fn parse_model_inputs(text: &str) -> Option<ModelInputs> {
    let mut lines = text
        .lines()
        .filter_map(|line| line.strip_prefix("ELICIT_MODEL_INPUTS="));
    let data: Value = serde_json::from_str(lines.next()?).ok()?;
    if lines.next().is_some() || data["supported"] != true {
        return None;
    }
    fn rows(data: &Value) -> Option<BTreeMap<String, String>> {
        let mut result = BTreeMap::new();
        for row in data.as_array()? {
            let values = row.as_array()?;
            if values.len() != 6
                || values[4] != false
                || values[1].as_str().is_none()
                || values[3].as_str().is_none()
                || !(values[2].is_null() || values[2].as_str().is_some())
                || !matches!(
                    values[5].as_str(),
                    Some("definition" | "theorem" | "opaque" | "axiom")
                )
            {
                return None;
            }
            let name = format!("decl:{}", values[0].as_str()?);
            if result
                .insert(name, fingerprint::hash(serde_json::to_vec(row).ok()?))
                .is_some()
            {
                return None;
            }
        }
        Some(result)
    }
    let models = rows(&data["models"])?;
    let mut contracts = BTreeMap::new();
    for contract in data["contracts"].as_array()? {
        let pair = contract.as_array()?;
        if pair.len() != 2 {
            return None;
        }
        let name = pair[0].as_str()?.to_owned();
        let dependencies = rows(&pair[1])?;
        let root = format!("decl:{name}");
        if !name.starts_with("ElicitContracts.c_")
            || !dependencies.contains_key(&root)
            || dependencies
                .iter()
                .any(|(key, value)| key != &root && models.get(key) != Some(value))
            || contracts.insert(name, dependencies).is_some()
        {
            return None;
        }
    }
    Some(ModelInputs { models, contracts })
}
fn model_inputs(scope: &Scope, targets: &[Target]) -> Result<Option<ModelInputs>> {
    let source = proof_root(scope).join("Specs.lean");
    if !source.is_file() {
        return Ok(None);
    }
    // Static source/model inputs only. File-reading, custom compiler hooks,
    // custom imports and metaprograms retain whole-file invalidation.
    version()?;
    let (before, context, files) = fingerprint::inputs(scope, Tool::Aeneas, "model-inputs-v1")?;
    let paths = files
        .keys()
        .filter(|path| !path.starts_with('@'))
        .map(PathBuf::from)
        .collect();
    if !fingerprint::reports_are_independent(scope, &paths)? {
        return Ok(None);
    }
    let m = manifest(scope)?;
    let contracts: Vec<_> = m
        .contracts
        .iter()
        .map(|(id, c)| (id, &c.requires, &c.ensures))
        .collect();
    let key = fingerprint::hash(serde_json::to_vec(&(
        &context,
        fs::read(&source)?,
        contracts,
    ))?);
    let cache = Path::new(&scope.manifest_path)
        .parent()
        .unwrap()
        .join(".elicit/cache");
    fs::create_dir_all(&cache)?;
    let cache_file = cache.join("model-inputs.json");
    if let Ok(bytes) = fs::read(&cache_file) {
        if let Ok(cached) = serde_json::from_slice::<ModelCache>(&bytes) {
            if cached.key == key {
                return Ok(cached.inputs);
            }
        }
    }
    let dir = tempfile::Builder::new()
        .prefix("model-inputs-")
        .tempdir_in(&cache)?;
    let mut snapshot = State::new(Tool::Aeneas, scope.clone());
    snapshot.targets = targets.iter().cloned().map(|t| (t.id.clone(), t)).collect();
    let mut inputs = match execute(&snapshot, dir.path(), false, 600) {
        Ok((output, _)) if output.code == Some(0) && !output.timeout && !output.interrupted => {
            fs::write(
                dir.path().join("ElicitModelInputs.lean"),
                include_str!("lean/ElicitModelInputs.lean"),
            )?;
            let output = process::run(
                &mut lean_command(dir.path(), "ElicitModelInputs")?,
                Duration::from_secs(60),
            )?;
            if output.code == Some(0) && !output.timeout && !output.interrupted {
                parse_model_inputs(&output.stdout)
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(inputs) = &mut inputs {
        // Source/tool inputs select the cache, but unrelated documentation is
        // not part of a model's meaning. Retain the actual freshly translated
        // Rust model and imported toolchain as conservative semantic inputs.
        let rust_model = fingerprint::hash(fs::read(dir.path().join("ElicitGenerated.lean"))?);
        let tcb = fingerprint::hash(serde_json::to_vec(&(
            files.get("@installed_tcb"),
            files.get("@rustc_version"),
        ))?);
        inputs
            .models
            .insert("@rust_model".into(), rust_model.clone());
        inputs.models.insert("@tcb".into(), tcb.clone());
        for contract in inputs.contracts.values_mut() {
            contract.insert("@rust_model".into(), rust_model.clone());
            contract.insert("@tcb".into(), tcb.clone());
        }
    }
    if process::interrupted()
        || fingerprint::input(scope, Tool::Aeneas, "model-inputs-v1")? != before
    {
        return Ok(None);
    }
    crate::store::atomic_write(
        &cache_file,
        &serde_json::to_vec(&ModelCache {
            key,
            inputs: inputs.clone(),
        })?,
    )?;
    Ok(inputs)
}

pub fn attach_specs(scope: &Scope, targets: &mut [Target]) -> Result<()> {
    let m = manifest(scope)?;
    let models = fs::read(proof_root(scope).join("Specs.lean")).unwrap_or_default();
    let semantic = model_inputs(scope, targets).unwrap_or(None);
    for t in targets {
        let c = m.contracts.get(&t.id);
        t.model_base_hash = fingerprint::hash(serde_json::to_vec(&(
            &t.spec_hash,
            c.map(|c| (&c.requires, &c.ensures)),
        ))?);
        t.model_inputs = match &semantic {
            Some(inputs) if !t.kind.proof() => inputs.models.clone(),
            Some(inputs) => inputs
                .contracts
                .get(&contract_name(&t.id))
                .cloned()
                .unwrap_or_else(|| {
                    BTreeMap::from([("@source".into(), fingerprint::hash(&models))])
                }),
            None => BTreeMap::from([("@source".into(), fingerprint::hash(&models))]),
        };
        t.model_bindings = if t.kind.proof() {
            None
        } else {
            Some(t.model_inputs.keys().cloned().collect())
        };
        t.spec_hash = t.model_hash();
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
fn proposition(t: &Target, c: &Contract, function: &str) -> Result<String> {
    if c.ensures.trim().is_empty() {
        bail!("missing Aeneas postcondition");
    }
    let pre = c.requires.as_deref().unwrap_or("by intros; exact True");
    Ok(format!(
        "elicit_contract {} for {function} requires ({pre}) ensures ({})\n",
        contract_name(&t.id),
        c.ensures
    ))
}
fn extracted(s: &State, t: &Target, llbc: &Value) -> Option<(usize, usize, usize, usize)> {
    let tr = &llbc["translated"];
    if !matches!(t.kind, Kind::Function | Kind::Method)
        || llbc["charon_version"] != "0.1.293"
        || llbc["has_errors"] != false
        || tr["runtime_checks"]["overflow_checks"] != true
        || !tr["target_information"]
            .as_array()
            .is_some_and(|ts| ts.len() == 1 && ts[0]["key"] == s.scope.target)
        || [
            "monomorphize",
            "monomorphize_mut",
            "skip_borrowck",
            "no_typecheck",
        ]
        .iter()
        .any(|k| tr["options"][k] == true)
    {
        return None;
    }
    let mut matches = vec![];
    for f in tr["fun_decls"].as_array()? {
        let m = &f["item_meta"];
        let Some(parts) = m["name"].as_array() else {
            continue;
        };
        if parts.last()?["Ident"][0].as_str() != Some(&t.name)
            || m["is_local"] != true
            || m["is_extern"] != false
            || m["has_errors"] != false
            || m["opacity"] != "Transparent"
            || !f["body"]["Structured"].is_object()
        {
            continue;
        }
        let span = &m["span"]["Untagged"]["data"];
        let Some(file) = tr["files"]
            .as_array()?
            .iter()
            .find(|f| f["id"] == span["file_id"])
        else {
            continue;
        };
        let Some(path) = file["name"]["Local"].as_str() else {
            continue;
        };
        let root = Path::new(&s.scope.manifest_path).parent().unwrap();
        if root.join(path).canonicalize().ok() != Path::new(&t.file).canonicalize().ok()
            || !file["contents"]
                .as_str()
                .is_some_and(|c| fs::read_to_string(&t.file).is_ok_and(|source| source == c))
        {
            continue;
        }
        let begin = (
            span["beg"]["line"].as_u64()? as usize,
            span["beg"]["col"].as_u64()? as usize,
        );
        let end = (
            span["end"]["line"].as_u64()? as usize,
            span["end"]["col"].as_u64()? as usize,
        );
        if begin <= (t.line, t.column) && (t.line, t.column) <= end {
            matches.push((begin.0, begin.1, end.0, end.1));
        }
    }
    if matches.len() == 1 {
        matches.pop()
    } else {
        None
    }
}
fn mapped(s: &State, t: &Target, text: &str, span: (usize, usize, usize, usize)) -> Option<String> {
    let mut matches = vec![];
    for doc in text.split("/-- ").skip(1) {
        let Some((header, body)) = doc.split_once("-/") else {
            continue;
        };
        if !header
            .lines()
            .next()
            .is_some_and(|l| l.ends_with(&format!("::{}]:", t.name)))
        {
            continue;
        }
        let Some(location) = header
            .lines()
            .find_map(|l| l.trim().strip_prefix("Source: '"))
        else {
            continue;
        };
        let Some((file, positions)) = location.split_once("', lines ") else {
            continue;
        };
        let root = Path::new(&s.scope.manifest_path).parent().unwrap();
        if root.join(file).canonicalize().ok() != Path::new(&t.file).canonicalize().ok() {
            continue;
        }
        if positions.split_whitespace().next()
            != Some(format!("{}:{}-{}:{}", span.0, span.1, span.2, span.3).as_str())
        {
            continue;
        }
        let Some(def) = body.trim_start().strip_prefix("def ") else {
            continue;
        };
        let Some(path) = def.split_whitespace().next() else {
            continue;
        };
        if path.rsplit('.').next().map(|v| v.trim_matches(['«', '»'])) != Some(&t.name) {
            continue;
        }
        if !path.chars().all(|c| {
            c.is_alphanumeric() || (matches!(c, '_' | '.' | '«' | '»') || c == char::from(39))
        }) {
            continue;
        }
        matches.push(format!("ElicitGenerated.{path}"));
    }
    if matches.len() == 1 {
        matches.pop()
    } else {
        None
    }
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
        let tick = Instant::now();
        let stage = cmd
            .get_args()
            .last()
            .and_then(|s| Path::new(s).file_name())
            .filter(|s| s.to_string_lossy().ends_with(".lean"))
            .or_else(|| Path::new(cmd.get_program()).file_name())
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let result = process::run(
            cmd,
            Duration::from_secs(seconds.saturating_sub(start.elapsed().as_secs())),
        );
        crate::store::Store { root: Path::new(&s.scope.manifest_path).parent().unwrap().to_path_buf() }
            .event("pipeline_stage_finished", json!({"stage":stage,"prove":prove,"elapsed_ms":tick.elapsed().as_millis(),"success":result.as_ref().is_ok_and(|o| o.code==Some(0) && !o.timeout && !o.interrupted)}))?;
        result
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
    fs::write(
        out.join("ElicitAdapter.lean"),
        include_str!("lean/ElicitAdapter.lean"),
    )?;
    o = run(&mut lean_command(out, "ElicitAdapter")?)?;
    if o.code != Some(0) {
        return Ok((o, None));
    }
    let m = manifest(&s.scope)?;
    let root = proof_root(&s.scope);
    let mut contracts = "import ElicitAdapter\nimport ElicitGenerated\nopen Aeneas Aeneas.Std Result ElicitGenerated\n".to_string();
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
            if let Some(function) =
                extracted(s, t, &llbc).and_then(|span| mapped(s, t, &translated, span))
            {
                match proposition(t, c, &function) {
                    Ok(p) => {
                        contracts.push_str(&p);
                    }
                    Err(e) if !prove => return Err(e),
                    Err(_) => continue,
                }
                // Lean unfolds the actual translated body and its helpers. The
                // transitive axiom audit also covers their assumptions; an opaque
                // helper cannot be promoted merely because its caller translated.
                let full = !t.trusted;
                eligible.insert(t.id.clone(), (c, full));
            } else if !prove && t.kind.proof() && s.unsupported(&t.id).is_none() {
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
    let mut lemmas = BTreeMap::new();
    if root.join("Lemmas.lean").exists() {
        fs::copy(root.join("Lemmas.lean"), out.join("Lemmas.lean"))?;
        o = run(&mut lean_command(out, "Lemmas")?)?;
        if o.code != Some(0) {
            return Ok((o, Some(json!({"phase":"lemmas"}))));
        }
        let steps: Vec<_> = s
            .proof_plan
            .steps
            .iter()
            .filter(|p| p.kind == crate::plan::StepKind::Lemma)
            .collect();
        let roots: Vec<_> = steps
            .iter()
            .map(|step| {
                let statement = step
                    .statement
                    .as_deref()
                    .context("missing lemma proposition")?;
                let theorem = step.theorem.as_deref().context("missing lemma theorem")?;
                if !crate::plan::lean_name(theorem) {
                    bail!("invalid lemma theorem name");
                }
                Ok((statement, theorem))
            })
            .collect::<Result<_>>()?;
        if !roots.is_empty() {
            fs::write(
                out.join("Audit.lean"),
                batch_audit_source("Lemmas", &roots)?,
            )?;
            o = run(&mut lean_command(out, "Audit")?)?;
            if o.code != Some(0) || o.timeout || o.interrupted {
                return Ok((o, Some(json!({"phase":"lemma-audit"}))));
            }
            for (step, row) in steps
                .iter()
                .zip(parse_batch_audits(&o.stdout, roots.len())?)
            {
                lemmas.insert(
                    step.id.clone(),
                    json!({"checked":row.accepted(),"step_hash":crate::plan::step_hash(step)}),
                );
            }
        }
    }
    if !root.join("Proofs.lean").exists() {
        o.code = Some(0);
        return Ok((o, Some(json!({"phase":"lemma-audit","lemmas":lemmas}))));
    }
    fs::copy(root.join("Proofs.lean"), out.join("Proofs.lean"))?;
    o = run(&mut lean_command(out, "Proofs")?)?;
    if o.code != Some(0) {
        return Ok((o, Some(json!({"phase":"proofs","lemmas":lemmas}))));
    }
    let mut audits = BTreeMap::new();
    let mut records = vec![];
    for (id, (c, full)) in eligible {
        let Some(theorem) = &c.theorem else { continue };
        if !crate::plan::lean_name(theorem) {
            bail!("invalid Lean theorem name");
        }
        records.push((id.clone(), contract_name(&id), theorem, full));
    }
    if !records.is_empty() {
        let roots: Vec<_> = records
            .iter()
            .map(|(_, statement, theorem, _)| (statement.as_str(), theorem.as_str()))
            .collect();
        fs::write(
            out.join("Audit.lean"),
            batch_audit_source("Proofs", &roots)?,
        )?;
        o = run(&mut lean_command(out, "Audit")?)?;
        if o.code != Some(0) || o.timeout || o.interrupted {
            return Ok((o, Some(json!({"phase":"audit","lemmas":lemmas}))));
        }
        let rows = parse_batch_audits(&o.stdout, roots.len())?;
        for ((id, _, _, full), row) in records.into_iter().zip(rows) {
            audits.insert(
                id,
                json!({"checked":row.accepted(),"full":full,"axioms":row.axiom_report()}),
            );
        }
    }
    // An individual rejected theorem does not discard independent successful proofs.
    o.code = Some(0);
    Ok((
        o,
        Some(json!({"phase":"audit","targets":audits,"lemmas":lemmas})),
    ))
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
            r.summary = "Lean kernel checked the original function contract for all translated inputs; standard axioms only".into();
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
    #[test]
    fn batch_audits_require_complete_unique_strict_rows_and_allowed_axioms() {
        use super::parse_batch_audits;
        let good = r#"ELICIT_AUDIT={"index":0,"checked":true,"axioms":[]}"#;
        assert!(parse_batch_audits(good, 1).unwrap()[0].accepted());
        for malformed in [
            String::new(),
            format!("{good}\n{good}"),
            good.replace("\"index\":0", "\"index\":1"),
            good.replace("\"axioms\":[]", "\"axioms\":[],\"unexpected\":true"),
        ] {
            assert!(parse_batch_audits(&malformed, 1).is_err());
        }
        for axiom in ["sorryAx", "projectAssumption", "Lean.ofReduceBool"] {
            let row = good.replace("\"axioms\":[]", &format!("\"axioms\":[\"{axiom}\"]"));
            assert!(!parse_batch_audits(&row, 1).unwrap()[0].accepted());
        }
        let unknown = good.replace("\"axioms\":[]", "\"axioms\":null");
        assert!(!parse_batch_audits(&unknown, 1).unwrap()[0].accepted());
    }
    #[test]
    fn malformed_or_unsupported_model_metadata_falls_back_conservatively() {
        use super::parse_model_inputs;
        for text in [
            "",
            "ELICIT_MODEL_INPUTS={}",
            "ELICIT_MODEL_INPUTS={\"supported\":false,\"models\":[],\"contracts\":[]}",
            "ELICIT_MODEL_INPUTS={\"supported\":true,\"models\":[[\"unsafe\",\"type\",null,\"[]\",true,\"definition\"]],\"contracts\":[]}",
        ] {
            assert!(parse_model_inputs(text).is_none());
        }
        let valid = "ELICIT_MODEL_INPUTS={\"supported\":true,\"models\":[],\"contracts\":[]}";
        assert!(parse_model_inputs(valid).is_some());
        assert!(parse_model_inputs(&format!("{valid}\n{valid}")).is_none());
    }

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
