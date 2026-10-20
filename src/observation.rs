//! Disposable disk cache for Cargo discovery and Rust inventory. No daemon.
use crate::{
    analysis::{self, Analyzer, Package, Selection},
    fingerprint,
    model::{Target, Tool},
    project::Config,
    store::{self, Store},
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};
use walkdir::WalkDir;

type Inputs = (String, String, BTreeMap<String, String>);
#[derive(Serialize, Deserialize)]
struct Cache {
    schema: u32,
    selection: Selection,
    tool: Tool,
    package: Package,
    inventory: Vec<Target>,
    limitations: Vec<String>,
    key: String,
    proof_independent: bool,
}
#[derive(Serialize, Deserialize)]
struct Envelope {
    checksum: String,
    payload: Cache,
}
pub struct Observation {
    pub package: Package,
    pub inventory: Vec<Target>,
    pub limitations: Vec<String>,
    pub inputs: Inputs,
    pub version: String,
    pub reused: bool,
}
fn read(path: &Path) -> Option<Cache> {
    let e: Envelope = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    (e.payload.schema == 1 && fingerprint::hash(serde_json::to_vec(&e.payload).ok()?) == e.checksum)
        .then_some(e.payload)
}
// Exemption is deliberately limited to Lean source. Builds/macros that could read
// arbitrary files keep the full file binding, including proof sources.
fn proof_independent(package: &Package, files: &BTreeMap<String, String>) -> Result<bool> {
    if package.metadata["packages"].as_array().is_some_and(|ps| {
        ps.iter().any(|p| {
            p["targets"].as_array().is_some_and(|ts| {
                ts.iter().any(|t| {
                    t["kind"].as_array().is_some_and(|ks| {
                        ks.iter().any(|k| k == "custom-build" || k == "proc-macro")
                    })
                })
            })
        })
    }) {
        return Ok(false);
    }
    for file in files
        .keys()
        .filter(|p| !p.starts_with('@') && Path::new(p).extension().is_some_and(|e| e == "rs"))
    {
        let source = fs::read_to_string(file)?;
        let Ok(ast) = syn::parse_file(&source) else {
            return Ok(false);
        };
        struct Reads(bool);
        impl<'ast> syn::visit::Visit<'ast> for Reads {
            fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
                // A module can use a non-.rs source path, including .lean.
                if a.path().is_ident("path") || a.path().is_ident("cfg_attr") {
                    self.0 = true;
                }
                syn::visit::visit_attribute(self, a);
            }
            fn visit_macro(&mut self, m: &'ast syn::Macro) {
                // Declarative macros can hide file reads or alias include!;
                // do not infer separation through arbitrary macro expansion.
                self.0 = true;
                syn::visit::visit_macro(self, m);
            }
        }
        let mut reads = Reads(false);
        syn::visit::Visit::visit_file(&mut reads, &ast);
        if reads.0 {
            return Ok(false);
        }
    }
    Ok(true)
}
fn key(package: &Package, inputs: &Inputs, config: &Config, independent: bool) -> Result<String> {
    let mut files: BTreeMap<_, _> = inputs
        .2
        .iter()
        .filter(|(p, _)| {
            !(independent
                && (p.as_str() == "@declarations"
                    || p.as_str() == "@verifier_version"
                    || (!p.starts_with('@')
                        && Path::new(p).extension().is_some_and(|e| e == "lean"))))
        })
        .map(|(p, d)| (p.clone(), d.clone()))
        .collect();
    // Workspace globs can discover a newly added member absent from old metadata.
    // Observe manifests across the workspace as well as cached dependency roots.
    for e in WalkDir::new(&package.scope.workspace_root)
        .into_iter()
        .filter_entry(|e| {
            e.path() != Path::new(&package.scope.target_directory)
                && !matches!(
                    e.file_name().to_str(),
                    Some(".git" | ".elicit" | ".lake" | ".why3find")
                )
        })
    {
        let e = e?;
        if e.file_type().is_symlink() {
            files.insert(
                e.path().to_string_lossy().into_owned(),
                format!("symlink:{}", fs::read_link(e.path())?.display()),
            );
        }
        if e.file_type().is_file() && e.file_name() == "Cargo.toml" {
            files.insert(
                e.path().to_string_lossy().into_owned(),
                fingerprint::hash(fs::read(e.path())?),
            );
        }
    }
    if let Some(packages) = package.metadata["packages"].as_array() {
        for p in packages {
            if let Some(path) = p["manifest_path"].as_str() {
                files.insert(
                    path.into(),
                    match fs::read(path) {
                        Ok(b) => fingerprint::hash(b),
                        Err(_) => "missing".into(),
                    },
                );
            }
        }
    }
    for program in ["cargo", "rustc", "rust-analyzer"] {
        let digest = match fingerprint::executable(program) {
            Some(p) => format!("{}:{}", p.display(), fingerprint::hash(fs::read(&p)?)),
            None => "missing".into(),
        };
        files.insert(format!("@analysis_tool:{program}"), digest);
    }
    Ok(fingerprint::hash(serde_json::to_vec(&(
        files,
        &config.package,
        config.verification.tool,
        independent,
    ))?))
}
pub fn refresh(store: &Store, config: &Config, selection: &Selection) -> Result<Observation> {
    let path = store.root.join(".elicit/cache/analysis.json");
    let version = fingerprint::tool_version(config.verification.tool)?;
    if let Some(c) =
        read(&path).filter(|c| c.selection == *selection && c.tool == config.verification.tool)
    {
        if let Ok(before) = fingerprint::inputs(&c.package.scope, c.tool, &version) {
            if key(&c.package, &before, config, c.proof_independent)? == c.key {
                let after = fingerprint::inputs(&c.package.scope, c.tool, &version)?;
                ensure!(
                    before == after
                        && key(&c.package, &after, config, c.proof_independent)? == c.key,
                    "inputs changed during cached observation; retry on stable files"
                );
                return Ok(Observation {
                    package: c.package,
                    inventory: c.inventory,
                    limitations: c.limitations,
                    inputs: after,
                    version,
                    reused: true,
                });
            }
        }
    }
    let package = analysis::package(&store.root, selection)?;
    let before = fingerprint::inputs(&package.scope, config.verification.tool, &version)?;
    let independent = proof_independent(&package, &before.2)?;
    let before_key = key(&package, &before, config, independent)?;
    let (inventory, limitations) =
        Analyzer::default().analyze(&package, config.verification.tool)?;
    let resolved = analysis::package(&store.root, selection)?;
    ensure!(
        resolved == package,
        "configuration/build scope changed during discovery; retry on stable files"
    );
    let after = fingerprint::inputs(&package.scope, config.verification.tool, &version)?;
    ensure!(
        before == after && key(&package, &after, config, independent)? == before_key,
        "inputs changed during discovery; retry on stable files"
    );
    let cache = Cache {
        schema: 1,
        selection: selection.clone(),
        tool: config.verification.tool,
        package,
        inventory,
        limitations,
        key: before_key,
        proof_independent: independent,
    };
    let envelope = Envelope {
        checksum: fingerprint::hash(serde_json::to_vec(&cache)?),
        payload: cache,
    };
    // Cache persistence is optional. Current observations do not depend on it.
    let _ = store::atomic_write(&path, &serde_json::to_vec(&envelope)?);
    let c = envelope.payload;
    Ok(Observation {
        package: c.package,
        inventory: c.inventory,
        limitations: c.limitations,
        inputs: after,
        version,
        reused: false,
    })
}
