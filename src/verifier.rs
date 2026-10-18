use crate::{fingerprint, model::*, process};
use anyhow::{Context, Result, bail};
use quote::ToTokens;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use syn::visit::Visit;

fn compile_command(s: &State, out: &Path, prove: bool, version: &str) -> Result<Command> {
    let mut c = Command::new("cargo");
    match s.tool {
        Tool::Aeneas => bail!("Aeneas uses the Charon/Lean pipeline"),
        Tool::Kani => {
            c.args(["kani", "-Z", "unstable-options", "-Z", "function-contracts"]);
            if prove {
                c.arg("--export-json").arg(out.join("kani.json"));
            } else {
                c.arg("--only-codegen");
            }
            let host = fingerprint::version("rustc", &["-vV"])?;
            if !host
                .lines()
                .any(|l| l == format!("host: {}", s.scope.target))
            {
                bail!(
                    "Kani 0.68 only supports the host target; selected target is {}",
                    s.scope.target
                );
            }
            let args = s.scope.cargo_args();
            let mut i = 0;
            while i < args.len() {
                if args[i] == "--target" {
                    i += 2;
                } else {
                    c.arg(&args[i]);
                    i += 1;
                }
            }
        }
        Tool::Creusot => {
            c.args([
                "creusot",
                "--package",
                &s.scope.package,
                "--only",
                "coma",
                "--output-dir",
            ])
            .arg(out)
            .arg("--erasure-check-dir")
            .arg(out.join("erasure"))
            .arg("--")
            .args(s.scope.cargo_args());
            c.env(
                "CARGO_TARGET_DIR",
                Path::new(&s.scope.manifest_path)
                    .parent()
                    .unwrap()
                    .join(".elicit/cache/creusot-build"),
            );
            let _ = version;
        }
    }
    Ok(c)
}
fn base(s: &State, id: &str, version: &str) -> Verification {
    Verification {
        id: format!("{}-{}-{}", s.revision, id, s.results.len()),
        target: id.into(),
        units: vec!["contract".into()],
        obligations: vec![],
        scope: Coverage::Unknown,
        assumptions: vec![],
        restrictions: vec![],
        input: s.input.clone(),
        tool: s.tool,
        tool_version: version.into(),
        trusted_base: vec![],
        reusable: false,
        outcome: Outcome::Interrupted,
        summary: "verification started, not finalized".into(),
        diagnostic: None,
        elapsed_ms: 0,
    }
}
fn final_input_outcome(after: &Result<String>, before: &str, interrupted: bool) -> Option<Outcome> {
    if interrupted {
        Some(Outcome::Interrupted)
    } else {
        match after {
            Err(_) => Some(Outcome::EnvironmentError),
            Ok(input) if input != before => Some(Outcome::InputChanged),
            Ok(_) => None,
        }
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub results: Vec<Verification>,
    pub input: String,
    pub stable: bool,
    pub cache: String,
    pub elapsed_ms: u64,
    pub environment_error: bool,
    pub backend_evidence: Option<Value>,
}
pub fn verify(
    s: &State,
    root: &Path,
    ids: &[String],
    seconds: u64,
    incremental: bool,
) -> Result<Report> {
    let started = Instant::now();
    let version = fingerprint::tool_version(s.tool)?;
    let before = fingerprint::input(&s.scope, s.tool, &version)?;
    if before != s.input {
        bail!("inputs changed before checking; rerun the command");
    }
    let cache = root.join(".elicit/cache");
    fs::create_dir_all(&cache)?;
    let dir = if incremental && s.tool == Tool::Aeneas {
        let dir = cache.join("check");
        fs::create_dir_all(&dir)?;
        dir
    } else {
        tempfile::Builder::new()
            .prefix("audit-")
            .tempdir_in(&cache)?
            .keep()
    };
    let execute = (|| -> Result<(process::Output, Option<Value>)> {
        if s.tool == Tool::Aeneas {
            return crate::aeneas::execute(s, &dir, true, seconds, incremental);
        }
        if s.tool == Tool::Creusot {
            let mut clean = Command::new("cargo");
            if let Some(tc) = version
                .split_whitespace()
                .find(|v| v.starts_with("nightly-"))
            {
                clean.arg(format!("+{}", tc.trim_matches(['(', ')', ','])));
            }
            clean
                .args([
                    "clean",
                    "--package",
                    &s.scope.package,
                    "--manifest-path",
                    &s.scope.manifest_path,
                    "--target",
                    &s.scope.target,
                ])
                .env("CARGO_TARGET_DIR", root.join(".elicit/cache/creusot-build"))
                .current_dir(root);
            let output = process::run(&mut clean, Duration::from_secs(seconds))?;
            if output.code != Some(0) {
                return Ok((output, None));
            }
        }
        let mut command = compile_command(s, &dir, true, &version)?;
        command.current_dir(root);
        let mut output = process::run(
            &mut command,
            Duration::from_secs(seconds.saturating_sub(started.elapsed().as_secs())),
        )?;
        let evidence = match s.tool {
            Tool::Aeneas => unreachable!(),
            Tool::Kani => fs::read(dir.join("kani.json"))
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok()),
            Tool::Creusot => {
                if output.code == Some(0)
                    && walkdir::WalkDir::new(&dir)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .any(|e| e.path().extension().is_some_and(|v| v == "coma"))
                {
                    let mut command = why3_command(root, &dir)?;
                    output = process::run(
                        &mut command,
                        Duration::from_secs(seconds.saturating_sub(started.elapsed().as_secs())),
                    )?;
                    Some(serde_json::json!({"phase":"why3find"}))
                } else {
                    None
                }
            }
        };
        fs::write(
            dir.join("verifier.log"),
            format!("{}\n{}", output.stdout, output.stderr),
        )?;
        Ok((output, evidence))
    })();
    let after = if process::interrupted() {
        Ok(before.clone())
    } else {
        fingerprint::input(&s.scope, s.tool, &version)
    };
    let stable = after.as_ref().ok() == Some(&before) && !process::interrupted();
    let mut results = vec![];
    for id in ids {
        let mut result = base(s, id, &version);
        result.elapsed_ms = started.elapsed().as_millis() as u64;
        match &execute {
            Err(error) => {
                result.outcome = Outcome::EnvironmentError;
                result.summary = format!("verifier setup/execution failed: {error}");
            }
            Ok((output, evidence)) => {
                if output.interrupted {
                    result.outcome = Outcome::Interrupted;
                    result.summary = "check interrupted; no evidence accepted".into();
                } else if output.timeout {
                    result.outcome = Outcome::Timeout;
                    result.summary = "check budget exhausted; retry with more time".into();
                } else if output.code != Some(0) {
                    result.outcome = if evidence.is_some() {
                        Outcome::Unresolved
                    } else {
                        Outcome::EnvironmentError
                    };
                    result.summary =
                        format!("verifier exited {:?}; inspect phase logs", output.code);
                } else {
                    result.outcome = Outcome::UnknownScope;
                    let parsed = match s.tool {
                        Tool::Aeneas => crate::aeneas::evidence(
                            s,
                            &s.targets[id],
                            evidence.as_ref(),
                            &mut result,
                        ),
                        Tool::Kani => {
                            if let Some(e) = evidence {
                                kani_evidence(s, &s.targets[id], e, &mut result)
                            } else {
                                Ok(())
                            }
                        }
                        Tool::Creusot => {
                            creusot_evidence(s, &s.targets[id], root, &dir, &mut result)
                        }
                    };
                    if parsed.is_err() {
                        result.outcome = Outcome::UnknownScope;
                        result.reusable = false;
                        result.summary = "native output could not be mapped to the contract".into();
                    }
                }
            }
        }
        if let Some(outcome) = final_input_outcome(&after, &before, process::interrupted()) {
            result.outcome = outcome;
            result.reusable = false;
            result.summary =
                "inputs changed or checking interrupted; no current evidence accepted".into();
        }
        let phase = execute
            .as_ref()
            .ok()
            .and_then(|(_, e)| e.as_ref())
            .and_then(|e| e["phase"].as_str())
            .unwrap_or("extraction-or-environment");
        result.diagnostic = Some(Diagnostic {
            phase: phase.into(),
            cache: dir.to_string_lossy().into_owned(),
        });
        results.push(result);
    }
    let environment_error = execute.is_err()
        || execute
            .as_ref()
            .is_ok_and(|(o, e)| o.code != Some(0) && e.is_none() && !o.timeout && !o.interrupted)
        || results
            .iter()
            .any(|r| r.outcome == Outcome::EnvironmentError);
    let backend_evidence = execute.ok().and_then(|(_, e)| e);
    Ok(Report {
        results,
        input: before,
        stable,
        cache: dir.to_string_lossy().into_owned(),
        elapsed_ms: started.elapsed().as_millis() as u64,
        environment_error,
        backend_evidence,
    })
}
pub(crate) fn safe_project(s: &State) -> Result<bool> {
    let manifest: toml::Value = toml::from_str(&fs::read_to_string(&s.scope.manifest_path)?)?;
    let root = Path::new(&s.scope.manifest_path).parent().unwrap();
    if root.join("build.rs").exists()
        || manifest["package"].get("build").is_some()
        || manifest
            .get("lib")
            .and_then(|v| v.get("proc-macro"))
            .and_then(|v| v.as_bool())
            == Some(true)
    {
        return Ok(false);
    }
    let allowed = match s.tool {
        Tool::Kani | Tool::Aeneas => vec![],
        Tool::Creusot => vec!["creusot-std"],
    };
    for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if manifest
            .get(table)
            .and_then(|v| v.as_table())
            .is_some_and(|t| t.keys().any(|k| !allowed.contains(&k.as_str())))
        {
            return Ok(false);
        }
    }
    if manifest.get("target").is_some()
        || manifest.get("patch").is_some()
        || manifest.get("replace").is_some()
    {
        return Ok(false);
    }
    if manifest
        .get("package")
        .and_then(|v| v.get("metadata"))
        .is_some()
    {
        return Ok(false);
    }
    for parent in Path::new(&s.scope.manifest_path).ancestors().skip(2) {
        let path = parent.join("Cargo.toml");
        if path.is_file() {
            let m: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
            if m.get("patch").is_some() || m.get("replace").is_some() {
                return Ok(false);
            }
        }
    }
    if let Some(dep) = manifest
        .get("dependencies")
        .and_then(|v| v.get("creusot-std"))
    {
        if let Some(table) = dep.as_table() {
            if table.keys().any(|k| {
                !matches!(
                    k.as_str(),
                    "version" | "features" | "default-features" | "optional"
                )
            }) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
pub(crate) fn original_fn(t: &Target) -> Result<syn::ItemFn> {
    // Find by the original definition location, not a duplicate function name.
    let f = syn::parse_file(&fs::read_to_string(&t.file)?)?;
    fn find(items: &[syn::Item], t: &Target) -> Option<syn::ItemFn> {
        for i in items {
            match i {
                syn::Item::Fn(f)
                    if f.sig.ident == t.name && f.sig.ident.span().start().line == t.line =>
                {
                    return Some(f.clone());
                }
                syn::Item::Mod(m) => {
                    if let Some((_, is)) = &m.content {
                        if let Some(f) = find(is, t) {
                            return Some(f);
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }
    find(&f.items, t).context("original free function not found")
}
fn attr_text(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .map(|a| a.to_token_stream().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
fn contract_closed(f: &syn::ItemFn, tool: Tool) -> bool {
    let mut names = BTreeSet::new();
    for i in &f.sig.inputs {
        if let syn::FnArg::Typed(p) = i {
            if let syn::Pat::Ident(i) = p.pat.as_ref() {
                names.insert(i.ident.to_string());
            }
        }
    }
    names.insert("result".into());
    // Parse each contract's expression. Paths must be bound primitives; custom
    // models, macros, trait calls and implicit external input remain unknown.
    fn expression(e: &syn::Expr, names: &BTreeSet<String>) -> bool {
        struct Check<'a> {
            names: &'a BTreeSet<String>,
            bad: bool,
        }
        impl<'ast> Visit<'ast> for Check<'_> {
            fn visit_expr(&mut self, e: &'ast syn::Expr) {
                match e {
                    syn::Expr::Path(p)
                        if p.path.segments.len() == 1
                            && self.names.contains(&p.path.segments[0].ident.to_string()) => {}
                    syn::Expr::Lit(_)
                    | syn::Expr::Unary(_)
                    | syn::Expr::Binary(_)
                    | syn::Expr::Paren(_)
                    | syn::Expr::Group(_) => {}
                    _ => self.bad = true,
                }
                syn::visit::visit_expr(self, e);
            }
        }
        let mut c = Check { names, bad: false };
        c.visit_expr(e);
        !c.bad
    }
    let mut count = 0;
    for a in &f.attrs {
        let m = if a.path().is_ident("cfg_attr") {
            use syn::parse::Parser;
            let syn::Meta::List(l) = &a.meta else {
                return false;
            };
            let Ok(ms) = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
                .parse2(l.tokens.clone())
            else {
                return false;
            };
            if ms.len() != 2
                || !matches!(&ms[0],syn::Meta::Path(p) if p.is_ident(&tool.to_string()))
            {
                return false;
            }
            ms[1].clone()
        } else {
            a.meta.clone()
        };
        let name = m.path().segments.last().unwrap().ident.to_string();
        if name == "doc" {
            continue;
        }
        if !matches!(name.as_str(), "requires" | "ensures") {
            return false;
        }
        let syn::Meta::List(l) = m else { return false };
        let Ok(mut e) = syn::parse2::<syn::Expr>(l.tokens) else {
            return false;
        };
        let mut bound = names.clone();
        if tool == Tool::Kani && name == "ensures" {
            if let syn::Expr::Closure(c) = e {
                if c.inputs.len() != 1 {
                    return false;
                }
                if let syn::Pat::Ident(i) = &c.inputs[0] {
                    bound.insert(i.ident.to_string());
                } else {
                    return false;
                }
                e = *c.body;
            } else {
                return false;
            }
        }
        if !expression(&e, &bound) {
            return false;
        }
        if name == "ensures" {
            count += 1;
        }
    }
    count > 0
}
fn kani_evidence(s: &State, t: &Target, e: &Value, r: &mut Verification) -> Result<()> {
    r.trusted_base = vec![
        "Kani 0.68 compiler and contract transformation".into(),
        "CBMC bit-vector logic and standard Rust/Kani models".into(),
    ];
    r.assumptions.extend(
        t.calls
            .iter()
            .map(|id| format!("callee proof coverage not certified by this adapter: {id}")),
    );
    if e["metadata"]["version"] != "1.0"
        || !e["metadata"]["kani_version"]
            .as_str()
            .is_some_and(|v| v.starts_with("0.68."))
        || e["metadata"]["target"] != s.scope.target
    {
        return Ok(());
    }
    let Some(harnesses) = e["harness_metadata"].as_array() else {
        return Ok(());
    };
    let Some(results) = e["verification_results"]["results"].as_array() else {
        return Ok(());
    };
    let summary = &e["verification_results"]["summary"];
    if summary["status"] != "completed"
        || summary["executed"].as_u64() != Some(results.len() as u64)
    {
        r.outcome = Outcome::Skipped;
        r.summary = "not all selected harnesses executed".into();
        return Ok(());
    }
    let source = syn::parse_file(&fs::read_to_string(&t.file)?)?;
    // Imports or module shadowing can replace the apparent ::kani::any path.
    let simple_file = source.items.iter().all(|i| matches!(i, syn::Item::Fn(_)));
    let original = original_fn(t).ok();
    let eligible = t.kind == Kind::Function
        && t.closed_primitive
        && t.calls.is_empty()
        && t.unresolved_calls.is_empty()
        && !t.trusted
        && safe_project(s)?
        && simple_file
        && original
            .as_ref()
            .is_some_and(|f| contract_closed(f, Tool::Kani));
    let package_root = Path::new(&s.scope.manifest_path).parent().unwrap();
    for h in harnesses {
        let kind = format!("ProofForContract {{ target_fn: \"{}\" }}", t.name);
        if h["attributes"]["kind"] != kind {
            continue;
        }
        let Some(name) = h["pretty_name"].as_str() else {
            continue;
        };
        let Some(result) = results.iter().find(|v| v["harness_id"] == name) else {
            continue;
        };
        let Some(checks) = result["checks"].as_array() else {
            continue;
        };
        r.units.push(format!("harness:{name}"));
        r.obligations.extend(checks.iter().map(|c| {
            format!(
                "{}:{}:{}",
                name,
                c["id"],
                c["category"].as_str().unwrap_or("unknown")
            )
        }));
        if result["status"] != "Success"
            || checks.is_empty()
            || checks.iter().any(|c| c["status"] != "Success")
        {
            r.outcome = Outcome::Unresolved;
            r.summary = "harness has failed, missing or undetermined obligations".into();
            continue;
        }
        let file = h["source"]["file"].as_str().unwrap_or("");
        let file = if Path::new(file).is_absolute() {
            PathBuf::from(file)
        } else {
            package_root.join(file)
        };
        let same = file.canonicalize().ok() == Path::new(&t.file).canonicalize().ok();
        let harness = source.items.iter().find_map(|i| {
            if let syn::Item::Fn(f) = i {
                if h["source"]["start_line"].as_u64()
                    == Some(f.sig.ident.span().start().line as u64)
                {
                    Some(f)
                } else {
                    None
                }
            } else {
                None
            }
        });
        let unrestricted = same
            && h["attributes"]["should_panic"] == false
            && h["has_loop_contracts"] == false
            && h["is_bounded"] == false
            && h["is_ctor_based"] == false
            && h["is_automatically_generated"] == false
            && h["contract"]["recursion_tracker"].is_null()
            && h["contract"]["contracted_function_name"].is_string()
            && eligible
            && harness
                .zip(original.as_ref())
                .is_some_and(|(h, f)| kani_unrestricted_harness(h, f));
        let ensures = original
            .as_ref()
            .map(|f| {
                f.attrs
                    .iter()
                    .filter(|a| attr_text(std::slice::from_ref(a)).contains("ensures"))
                    .count()
            })
            .unwrap_or(0);
        let postconditions = checks
            .iter()
            .filter(|c| {
                c["category"] == "assertion"
                    && c["function"]
                        .as_str()
                        .is_some_and(|f| f.starts_with(&format!("{}::", t.name)))
                    && c["location"]["file"].as_str().is_some_and(|f| {
                        package_root.join(f).canonicalize().ok()
                            == Path::new(&t.file).canonicalize().ok()
                    })
            })
            .count();
        if unrestricted && ensures > 0 && postconditions >= ensures {
            r.outcome = Outcome::Success;
            r.scope = Coverage::FullContract;
            r.reusable = fingerprint::tcb_tracked(Tool::Kani);
            r.summary="native proof_for_contract: all primitive inputs, all contract and safety obligations discharged".into();
            return Ok(());
        }
        r.scope = Coverage::Limited;
        r.restrictions.push(format!("{name}: harness instantiation, restrictions, project assumptions or full contract mapping could not be excluded"));
        r.summary = "successful harness retained; not promoted to full API verification".into();
    }
    Ok(())
}
fn kani_unrestricted_harness(h: &syn::ItemFn, f: &syn::ItemFn) -> bool {
    if !h.sig.inputs.is_empty()
        || !h.sig.generics.params.is_empty()
        || h.sig.unsafety.is_some()
        || h.sig.asyncness.is_some()
    {
        return false;
    }
    for a in &h.attrs {
        let text = a.to_token_stream().to_string().replace(' ', "");
        if text != "# [cfg(kani)]".replace(' ', "")
            && text != format!("#[kani::proof_for_contract({})]", f.sig.ident)
            && !a.path().is_ident("doc")
        {
            return false;
        }
    }
    if h.block.stmts.len() != f.sig.inputs.len() + 1 {
        return false;
    }
    let mut names = vec![];
    for (stmt, arg) in h.block.stmts.iter().zip(&f.sig.inputs) {
        let (syn::Stmt::Local(l), syn::FnArg::Typed(arg)) = (stmt, arg) else {
            return false;
        };
        if !l.attrs.is_empty() {
            return false;
        }
        let syn::Pat::Type(p) = &l.pat else {
            return false;
        };
        let syn::Pat::Ident(n) = p.pat.as_ref() else {
            return false;
        };
        if n.mutability.is_some() || n.by_ref.is_some() || n.subpat.is_some() {
            return false;
        }
        if p.ty.to_token_stream().to_string() != arg.ty.to_token_stream().to_string() {
            return false;
        }
        let Some(i) = &l.init else { return false };
        if i.diverge.is_some() {
            return false;
        }
        let syn::Expr::Call(c) = i.expr.as_ref() else {
            return false;
        };
        if c.func.to_token_stream().to_string().replace(' ', "") != "::kani::any"
            || !c.args.is_empty()
        {
            return false;
        }
        if names.contains(&n.ident.to_string()) {
            return false;
        }
        names.push(n.ident.to_string());
    }
    let Some(syn::Stmt::Expr(syn::Expr::Call(c), _)) = h.block.stmts.last() else {
        return false;
    };
    c.attrs.is_empty()
        && f.sig.ident == c.func.to_token_stream().to_string()
        && c.args.len() == names.len()
        && c.args
            .iter()
            .zip(names)
            .all(|(e, n)| e.to_token_stream().to_string() == n)
}
fn creusot_home() -> Result<PathBuf> {
    fingerprint::creusot_home().context("cannot locate Creusot installation; set CREUSOT_DATA_HOME")
}
fn why3_command(root: &Path, out: &Path) -> Result<Command> {
    if !root.join("why3find.json").is_file() {
        bail!(
            "why3find.json required in package root; configure standard creusot package and provers"
        );
    }
    let home = creusot_home()?;
    let why3 = fingerprint::version(
        home.join("bin/why3")
            .to_str()
            .context("invalid Why3 path")?,
        &["--version"],
    )?;
    let why3find = fingerprint::version(
        home.join("bin/why3find")
            .to_str()
            .context("invalid Why3find path")?,
        &["--version"],
    )?;
    if !why3.contains("1.8.2") || !why3find.contains("1.3.") {
        bail!("unsupported Why3/Why3find versions; Creusot 0.14 requires its pinned installation");
    }
    let mut c = Command::new(home.join("bin/why3find"));
    c.args([
        "prove",
        "--no-cache",
        "--no-autodetect-provers",
        "--summary",
    ]);
    for entry in walkdir::WalkDir::new(out) {
        let e = entry?;
        if e.path().extension().is_some_and(|e| e == "coma") {
            c.arg(e.path());
        }
    }
    c.current_dir(root)
        .env("WHY3CONFIG", home.join("creusot_why3.conf"))
        .env(
            "DUNE_DIR_LOCATIONS",
            format!("why3find:lib:{}", home.join("share/why3find").display()),
        )
        .env(
            "PATH",
            format!(
                "{}:{}",
                home.join("bin").display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        );
    Ok(c)
}
fn creusot_evidence(
    s: &State,
    t: &Target,
    root: &Path,
    out: &Path,
    r: &mut Verification,
) -> Result<()> {
    r.trusted_base = vec![
        "Creusot 0.14 compiler, erasure checks and standard models".into(),
        "Why3 1.8.2+git, Why3find 1.3 and configured SMT provers".into(),
    ];
    r.assumptions.extend(
        t.calls
            .iter()
            .map(|id| format!("callee proof coverage not certified by this adapter: {id}")),
    );
    if t.trusted {
        r.outcome = Outcome::Skipped;
        r.assumptions
            .push("project trusted declaration: body proof omitted".into());
        r.summary = "trusted is not a body proof".into();
        return Ok(());
    }
    let original = original_fn(t).ok();
    let closed = !["WHY3LOADPATH", "WHY3DATA", "WHY3LIB"]
        .iter()
        .any(|k| std::env::var_os(k).is_some())
        && t.kind == Kind::Function
        && t.closed_primitive
        && t.calls.is_empty()
        && t.unresolved_calls.is_empty()
        && safe_project(s)?
        && original
            .as_ref()
            .is_some_and(|f| contract_closed(f, Tool::Creusot));
    let config: Value = serde_json::from_slice(&fs::read(root.join("why3find.json"))?)?;
    let standard_config = config.as_object().is_some_and(|c| {
        c.keys().all(|k| {
            matches!(
                k.as_str(),
                "packages" | "provers" | "tactics" | "time" | "depth" | "level"
            )
        })
    }) && config["packages"] == serde_json::json!(["creusot"])
        && config["tactics"].as_array().is_none_or(|ts| {
            ts.iter().all(|t| {
                matches!(
                    t.as_str(),
                    Some("compute_specified" | "split_vc" | "simplify_trivial_quantification")
                )
            })
        });
    // Project axioms/assumptions are never promoted into the trusted base.
    let mut project_assumptions = false;
    for e in walkdir::WalkDir::new(out) {
        let e = e?;
        if e.path().extension().is_some_and(|v| v == "coma") {
            let text = fs::read_to_string(e.path())?;
            if text
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|v| matches!(v, "axiom" | "assume"))
            {
                project_assumptions = true;
            }
        }
    }
    if project_assumptions {
        r.assumptions
            .push("project Coma contains axiom or assume".into());
    }
    let mut mapped = false;
    for e in walkdir::WalkDir::new(out) {
        let e = e?;
        if !e.path().extension().is_some_and(|v| v == "coma") {
            continue;
        }
        let text = fs::read_to_string(e.path())?;
        let marker = format!("(* #\"{}\" {} ", t.file, t.line);
        if !text.starts_with(&marker)
            || !text
                .lines()
                .any(|l| l.starts_with(&format!("let {} ", t.name)))
        {
            continue;
        }
        mapped = true;
        let path = e.path().with_extension("").join("proof.json");
        let Ok(bytes) = fs::read(&path) else { continue };
        let Ok(proof) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        let Some(theories) = proof["proofs"].as_object() else {
            continue;
        };
        let goals = theories
            .values()
            .filter_map(Value::as_object)
            .flat_map(|m| m.iter())
            .filter(|(k, _)| k.as_str() == format!("vc_{}", t.name))
            .collect::<Vec<_>>();
        if goals.is_empty() {
            continue;
        }
        r.obligations.extend(
            goals
                .iter()
                .map(|(k, _)| format!("{}::{k}", e.path().strip_prefix(out).unwrap().display())),
        );
        if goals.iter().any(|(_, p)| !proof_complete(p)) {
            r.outcome = Outcome::Unresolved;
            r.summary = "Why3 proof tree contains unresolved goals".into();
            continue;
        }
        let ensures = original
            .as_ref()
            .map(|f| {
                f.attrs
                    .iter()
                    .filter(|a| {
                        a.path()
                            .segments
                            .last()
                            .is_some_and(|p| p.ident == "ensures")
                    })
                    .count()
            })
            .unwrap_or(0);
        let full = closed
            && standard_config
            && !project_assumptions
            && ensures > 0
            && text.matches(&format!("[@expl:{} ensures]", t.name)).count() == ensures;
        if full {
            r.outcome = Outcome::Success;
            r.scope = Coverage::FullContract;
            r.reusable = fingerprint::tcb_tracked(Tool::Creusot);
            r.summary="fresh Coma source mapping and complete Why3 proof tree cover the primitive function contract".into();
            return Ok(());
        }
        r.scope = Coverage::Limited;
        r.restrictions.push(
            "generic/method/helper scope, nonstandard models or contract mapping not established"
                .into(),
        );
        r.summary = "successful proof tree retained with unconfirmed API coverage".into();
    }
    if !mapped {
        r.outcome = Outcome::Skipped;
        r.summary = "selected target had no fresh mapped Coma body proof".into();
    }
    Ok(())
}
fn proof_complete(p: &Value) -> bool {
    if p["prover"].as_str().is_some_and(|v| !v.is_empty()) {
        return true;
    }
    let Some(t) = p["tactic"].as_str() else {
        return false;
    };
    if !matches!(
        t,
        "compute_specified" | "split_vc" | "simplify_trivial_quantification"
    ) {
        return false;
    }
    // A known sound transformation may discharge an existing VC completely.
    // The caller still requires a fresh mapped vc_FUNCTION and nonempty contract;
    // an absent VC or an empty run is never accepted.
    p["children"]
        .as_array()
        .is_some_and(|cs| cs.iter().all(proof_complete))
}

#[cfg(test)]
mod tests {
    #[test]
    fn interrupted_inspection_is_not_a_source_change() {
        use super::*;
        let failure: Result<String> = Err(anyhow::anyhow!("compiler interrupted"));
        assert_eq!(
            final_input_outcome(&failure, "old", true),
            Some(Outcome::Interrupted)
        );
        assert_eq!(
            final_input_outcome(&failure, "old", false),
            Some(Outcome::EnvironmentError)
        );
        assert_eq!(
            final_input_outcome(&Ok("new".into()), "old", false),
            Some(Outcome::InputChanged)
        );
        assert_eq!(final_input_outcome(&Ok("old".into()), "old", false), None);
    }

    use super::*;
    fn case() -> (tempfile::TempDir, State, Target, Value) {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir(d.path().join("src")).unwrap();
        fs::write(
            d.path().join("Cargo.toml"),
            "[package]\nname=\"fixture\"\nversion=\"0.0.0\"\n",
        )
        .unwrap();
        fs::write(
            d.path().join("src/lib.rs"),
            include_str!("../fixtures/kani/src/lib.rs"),
        )
        .unwrap();
        let t = Target {
            id: "fixture::identity::Function".into(),
            name: "identity".into(),
            kind: Kind::Function,
            visibility: Visibility::Public,
            file: d.path().join("src/lib.rs").to_string_lossy().into_owned(),
            line: 2,
            column: 7,
            aliases: vec![],
            spec_hash: "spec".into(),
            model_base_hash: String::new(),
            model_inputs: Default::default(),
            body_hash: "body".into(),
            source_hash: "source".into(),
            calls: Default::default(),
            unresolved_calls: vec![],
            external_calls: Default::default(),
            source_facts: None,
            native_contract: None,
            required_units: vec!["contract".into()],
            closed_primitive: true,
            trusted: false,
        };
        let mut s = State::new(
            Tool::Kani,
            Scope {
                package: "fixture".into(),
                manifest_path: d.path().join("Cargo.toml").to_string_lossy().into_owned(),
                target: "x86_64-unknown-linux-gnu".into(),
                ..Scope::default()
            },
        );
        s.targets.insert(t.id.clone(), t.clone());
        let e = serde_json::from_str(include_str!("../tests/data/kani-0.68-success.json")).unwrap();
        (d, s, t, e)
    }
    fn parse(s: &State, t: &Target, e: &Value) -> Verification {
        let mut r = base(s, &t.id, "0.68.0");
        r.outcome = Outcome::UnknownScope;
        kani_evidence(s, t, e, &mut r).unwrap();
        r
    }
    #[test]
    fn native_kani_schema_maps_original_contract() {
        let (_d, s, t, e) = case();
        let r = parse(&s, &t, &e);
        assert_eq!(r.outcome, Outcome::Success);
        assert_eq!(r.scope, Coverage::FullContract);
        assert_eq!(r.obligations.len(), 84);
    }
    #[test]
    fn zero_checks_and_unexecuted_are_never_proof() {
        let (_d, s, t, mut e) = case();
        e["verification_results"]["results"][0]["checks"] = serde_json::json!([]);
        assert_ne!(parse(&s, &t, &e).outcome, Outcome::Success);
        e["verification_results"]["summary"]["status"] = serde_json::json!("interrupted");
        assert_eq!(parse(&s, &t, &e).outcome, Outcome::Skipped);
    }
    #[test]
    fn bounded_and_generic_success_stays_partial() {
        let (_d, s, t, mut e) = case();
        e["harness_metadata"][0]["is_bounded"] = Value::Bool(true);
        assert_eq!(parse(&s, &t, &e).scope, Coverage::Limited);
        e["harness_metadata"][0]["is_bounded"] = Value::Bool(false);
        let mut t = t;
        t.closed_primitive = false;
        assert_eq!(parse(&s, &t, &e).scope, Coverage::Limited);
    }
    #[test]
    fn restricted_harness_or_stub_is_not_full_api() {
        let (d, s, t, e) = case();
        let p = d.path().join("src/lib.rs");
        let source = fs::read_to_string(&p).unwrap();
        fs::write(&p, source.replace("identity(x);", "identity(0);")).unwrap();
        assert_eq!(parse(&s, &t, &e).scope, Coverage::Limited);
        fs::write(
            &p,
            source.replace(
                "#[kani::proof_for_contract(identity)]",
                "#[kani::proof_for_contract(identity)]\n#[kani::stub_verified(helper)]",
            ),
        )
        .unwrap();
        assert_ne!(parse(&s, &t, &e).outcome, Outcome::Success);
    }
    #[test]
    fn failed_property_is_not_hidden_by_summary_success() {
        let (_d, s, t, mut e) = case();
        e["verification_results"]["results"][0]["checks"][10]["status"] =
            serde_json::json!("Failure");
        assert_eq!(parse(&s, &t, &e).outcome, Outcome::Unresolved);
    }
    #[test]
    fn contracts_cannot_hide_external_calls_or_variables() {
        let (_d, _s, t, _e) = case();
        let f = original_fn(&t).unwrap();
        assert!(contract_closed(&f, Tool::Kani));
        let f: syn::ItemFn = syn::parse_quote!(
            #[kani::ensures(|r| *r == MAGIC)]
            pub fn identity(x: u8) -> u8 {
                x
            }
        );
        assert!(!contract_closed(&f, Tool::Kani));
        let f: syn::ItemFn = syn::parse_quote!(
            #[kani::ensures(|r| helper(*r,x))]
            pub fn identity(x: u8) -> u8 {
                x
            }
        );
        assert!(!contract_closed(&f, Tool::Kani));
    }
    #[test]
    fn body_closure_rejects_implicit_custom_types_and_cfg() {
        let f: syn::ItemFn = syn::parse_quote!(
            pub fn f(x: u8) -> u8 {
                let y = Opaque;
                x + y.value
            }
        );
        assert!(!crate::analysis::closed_primitive(&f));
        let f: syn::ItemFn = syn::parse_quote!(
            pub fn f(x: u8) -> u8 {
                #[cfg(kani)]
                {
                    return 0;
                }
                x
            }
        );
        assert!(!crate::analysis::closed_primitive(&f));
    }
    #[test]
    fn creusot_trusted_and_absent_target_are_not_proof() {
        let (d, mut s, mut t, _) = case();
        s.tool = Tool::Creusot;
        t.trusted = true;
        let mut r = base(&s, &t.id, "0.14");
        creusot_evidence(&s, &t, d.path(), d.path(), &mut r).unwrap();
        assert_eq!(r.outcome, Outcome::Skipped);
        assert!(!r.assumptions.is_empty());
        t.trusted = false;
        fs::write(
            d.path().join("why3find.json"),
            r#"{"packages":["creusot"],"tactics":["compute_specified"]}"#,
        )
        .unwrap();
        let out = d.path().join("empty");
        fs::create_dir(&out).unwrap();
        let mut r = base(&s, &t.id, "0.14");
        creusot_evidence(&s, &t, d.path(), &out, &mut r).unwrap();
        assert_eq!(r.outcome, Outcome::Skipped);
        assert!(r.obligations.is_empty());
    }
    #[test]
    fn unknown_or_stuck_why3_tree_is_not_complete() {
        assert!(!proof_complete(&Value::Null));
        assert!(!proof_complete(
            &serde_json::json!({"tactic":"project_axiom","children":[]})
        ));
        assert!(!proof_complete(
            &serde_json::json!({"tactic":"split_vc","children":[{"prover":"Alt-Ergo@2.6.4","time":0.1},null]})
        ));
        assert!(proof_complete(
            &serde_json::json!({"prover":"Alt-Ergo@2.6.4","time":0.1})
        ));
    }
}
