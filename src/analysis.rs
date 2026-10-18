use crate::{fingerprint::hash, model::*, process};
use anyhow::{Context, Result, bail};
use quote::ToTokens;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

#[derive(Default, Clone)]
pub struct Selection {
    pub package: Option<String>,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub target: Option<String>,
}
pub struct Package {
    pub scope: Scope,
    pub entries: Vec<PathBuf>,
    pub metadata: Value,
    pub cfg: BTreeSet<String>,
}
pub fn package(root: &Path, selection: &Selection) -> Result<Package> {
    let mut args = vec![
        "metadata".to_string(),
        "--format-version".into(),
        "1".into(),
        "--manifest-path".into(),
        root.join("Cargo.toml").to_string_lossy().into_owned(),
    ];
    if selection.all_features {
        args.push("--all-features".into());
    }
    if selection.no_default_features {
        args.push("--no-default-features".into());
    }
    if !selection.features.is_empty() {
        args.extend(["--features".into(), selection.features.join(",")]);
    }
    let target = match &selection.target {
        Some(t) => t.clone(),
        None => crate::fingerprint::version("rustc", &["-vV"])?
            .lines()
            .find_map(|l| l.strip_prefix("host: "))
            .context("rustc host missing")?
            .to_string(),
    };
    args.extend(["--filter-platform".into(), target.clone()]);
    let o = process::run(
        Command::new("cargo").args(&args).current_dir(root),
        Duration::from_secs(120),
    )?;
    if o.code != Some(0) {
        bail!("cargo metadata failed: {}", o.stderr);
    }
    let meta: Value = serde_json::from_str(&o.stdout)?;
    let packages = meta["packages"]
        .as_array()
        .context("metadata packages missing")?;
    let candidates: Vec<_> = packages
        .iter()
        .filter(|p| {
            meta["workspace_members"]
                .as_array()
                .is_some_and(|m| m.contains(&p["id"]))
                && selection
                    .package
                    .as_ref()
                    .is_none_or(|name| p["name"].as_str() == Some(name))
        })
        .collect();
    if candidates.len() != 1 {
        bail!(
            "select one package with --package; candidates: {}",
            candidates
                .iter()
                .map(|p| p["name"].as_str().unwrap_or("?"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let p = candidates[0];
    let features: Vec<String> = meta["resolve"]["nodes"]
        .as_array()
        .and_then(|ns| ns.iter().find(|n| n["id"] == p["id"]))
        .and_then(|n| n["features"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let roots: Vec<String> = packages
        .iter()
        .filter(|p| p["source"].is_null())
        .filter_map(|p| {
            p["manifest_path"]
                .as_str()
                .and_then(|s| Path::new(s).parent())
                .map(|p| p.to_string_lossy().into_owned())
        })
        .collect();
    let scope = Scope {
        config_path: root.join("elicit.toml").to_string_lossy().into_owned(),
        package: p["name"].as_str().unwrap().into(),
        manifest_path: p["manifest_path"].as_str().unwrap().into(),
        workspace_root: meta["workspace_root"].as_str().unwrap().into(),
        target_directory: meta["target_directory"].as_str().unwrap().into(),
        features: selection.features.clone(),
        resolved_features: features.clone(),
        all_features: selection.all_features,
        no_default_features: selection.no_default_features,
        target: target.clone(),
        requested_target: selection.target.clone(),
        local_roots: roots,
    };
    let entries = p["targets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| {
            t["kind"].as_array().is_some_and(|ks| {
                ks.iter().any(|v| {
                    matches!(
                        v.as_str(),
                        Some("lib" | "bin" | "rlib" | "cdylib" | "staticlib" | "proc-macro")
                    )
                })
            })
        })
        .filter_map(|t| t["src_path"].as_str().map(PathBuf::from))
        .collect();
    let o = process::run(
        Command::new("rustc").args(["--print", "cfg", "--target", &target]),
        Duration::from_secs(30),
    )?;
    if o.code != Some(0) {
        bail!("cannot obtain target cfg: {}", o.stderr);
    }
    let mut cfg: BTreeSet<_> = o.stdout.lines().map(|s| s.replace(' ', "")).collect();
    for f in features {
        cfg.insert(format!("feature=\"{f}\""));
    }
    Ok(Package {
        scope,
        entries,
        metadata: meta,
        cfg,
    })
}
#[derive(Default)]
pub struct Analyzer {
    pub ra: Option<Ra>,
}
impl Analyzer {
    pub fn analyze(&mut self, p: &Package, tool: Tool) -> Result<(Vec<Target>, Vec<String>)> {
        // A changed build configuration needs a fresh semantic workspace.
        if self.ra.as_ref().is_some_and(|r| r.scope != p.scope) {
            self.ra = None;
        }
        if self.ra.is_none() {
            self.ra = Some(Ra::start(&p.scope, tool)?);
        }
        let mut ix = Index {
            package: p.scope.package.clone(),
            cfg: p.cfg.clone(),
            tool,
            targets: vec![],
            calls: vec![],
            aliases: vec![],
            incomplete: vec![],
            visited: BTreeSet::new(),
        };
        if p.metadata["workspace_members"]
            .as_array()
            .is_some_and(|v| v.len() > 1)
        {
            if let Some(package) = p.metadata["packages"]
                .as_array()
                .and_then(|ps| ps.iter().find(|v| v["name"] == p.scope.package))
            {
                let definitions = package["features"].as_object();
                let mut expected = BTreeSet::new();
                if let Some(definitions) = definitions {
                    if p.scope.all_features {
                        expected.extend(definitions.keys().cloned());
                    }
                    if !p.scope.no_default_features && definitions.contains_key("default") {
                        expected.insert("default".to_string());
                    }
                    for feature in &p.scope.features {
                        if let Some((pkg, f)) = feature.split_once('/') {
                            if pkg == p.scope.package {
                                expected.insert(f.to_string());
                            }
                        } else {
                            expected.insert(feature.clone());
                        }
                    }
                    loop {
                        let old = expected.clone();
                        for feature in old.iter() {
                            if let Some(items) = definitions.get(feature).and_then(Value::as_array)
                            {
                                for value in items.iter().filter_map(Value::as_str) {
                                    if definitions.contains_key(value) {
                                        expected.insert(value.to_string());
                                    } else if let Some((dep, _)) = value.split_once('/') {
                                        if !dep.ends_with('?') && definitions.contains_key(dep) {
                                            expected.insert(dep.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        if expected == old {
                            break;
                        }
                    }
                }
                if expected != p.scope.resolved_features.iter().cloned().collect() {
                    ix.incomplete.push("workspace feature unification differs from the selected package configuration; scope review required".into());
                }
            }
        }
        if Path::new(&p.scope.manifest_path)
            .parent()
            .unwrap()
            .join("build.rs")
            .exists()
        {
            ix.incomplete.push(
                "build.rs may generate source or cfg; generated definitions require review".into(),
            );
        }
        if ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"]
            .iter()
            .any(|k| std::env::var(k).is_ok_and(|v| !v.trim().is_empty()))
        {
            ix.incomplete.push(
                "custom compiler flags may change cfg; semantic/structural scope needs review"
                    .into(),
            );
        }
        for parent in Path::new(&p.scope.manifest_path).ancestors().skip(1) {
            for name in [".cargo/config", ".cargo/config.toml"] {
                let path = parent.join(name);
                if path.is_file() && fs::read_to_string(path)?.contains("rustflags") {
                    ix.incomplete
                        .push("Cargo config rustflags may change cfg; scope needs review".into());
                }
            }
        }
        for entry in &p.entries {
            ix.file(entry, &[], true)?;
        }
        let ra = self.ra.as_mut().unwrap();
        let files: BTreeSet<_> = ix.targets.iter().map(|t| t.file.clone()).collect();
        for f in &files {
            ra.open(Path::new(f))?;
        }
        ra.settle()?;
        for (id, file, line, col) in &ix.calls {
            let locations = ra.definition(file, *line, *col)?;
            let matches: Vec<_> = ix
                .targets
                .iter()
                .filter(|t| {
                    locations.iter().any(|(f, l)| {
                        f == &t.file && *l >= t.line.saturating_sub(1) && *l <= t.line
                    })
                })
                .map(|t| t.id.clone())
                .collect();
            let external = !locations.is_empty()
                && locations.iter().all(|(f, _)| {
                    !p.scope
                        .local_roots
                        .iter()
                        .any(|r| Path::new(f).starts_with(r))
                });
            if let Some(t) = ix.targets.iter_mut().find(|t| &t.id == id) {
                if matches.is_empty() && !external {
                    t.unresolved_calls
                        .push(format!("{file}:{}:{}", line + 1, col + 1));
                }
                t.calls.extend(matches);
            }
        }
        for (alias, file, line, col) in &ix.aliases {
            for (f, l) in ra.definition(file, *line, *col)? {
                for t in &mut ix.targets {
                    if t.file == f && l >= t.line.saturating_sub(1) && l <= t.line {
                        t.visibility = Visibility::Public;
                        t.aliases.push(alias.clone());
                    }
                }
            }
        }
        for t in &ix.targets {
            if t.visibility != Visibility::Auxiliary && !t.unresolved_calls.is_empty() {
                ix.incomplete.push(format!(
                    "unresolved calls in {}: {}",
                    t.id,
                    t.unresolved_calls.join(", ")
                ));
            }
        }
        if tool == Tool::Aeneas {
            crate::aeneas::attach_specs(&p.scope, &mut ix.targets)?;
        }
        ix.targets.sort_by(|a, b| a.id.cmp(&b.id));
        ix.incomplete.sort();
        ix.incomplete.dedup();
        Ok((ix.targets, ix.incomplete))
    }
}
fn tokens(v: &impl ToTokens) -> String {
    v.to_token_stream().to_string()
}
fn attrs_hash(a: &[syn::Attribute]) -> String {
    a.iter()
        .filter(|a| !a.path().is_ident("doc"))
        .map(tokens)
        .collect::<Vec<_>>()
        .join(" ")
}
fn auxiliary(a: &[syn::Attribute]) -> bool {
    a.iter().any(|a| {
        let s = tokens(&a.meta);
        let name = a
            .path()
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        matches!(
            name.as_str(),
            "proof" | "proof_for_contract" | "logic" | "predicate" | "ghost" | "lemma"
        ) || name == "cfg_attr"
            && [
                "proof_for_contract",
                "kani :: proof",
                "creusot :: logic",
                "creusot_std :: logic",
                "predicate",
            ]
            .iter()
            .any(|p| s.contains(p))
    })
}
fn runtime_attrs(attrs: &[syn::Attribute]) -> String {
    fn proof_meta(m: &syn::Meta) -> bool {
        let Some(name) = m.path().segments.last().map(|s| s.ident.to_string()) else {
            return false;
        };
        if matches!(
            name.as_str(),
            "requires"
                | "ensures"
                | "invariant"
                | "variant"
                | "maintains"
                | "trusted"
                | "terminates"
                | "open"
                | "logic"
                | "predicate"
                | "ghost"
                | "lemma"
                | "proof"
                | "proof_for_contract"
        ) {
            return true;
        }
        if name == "cfg_attr" {
            use syn::parse::Parser;
            if let syn::Meta::List(l) = m {
                if let Ok(ms) =
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
                        .parse2(l.tokens.clone())
                {
                    return ms.len() >= 2
                        && matches!(&ms[0],syn::Meta::Path(p) if p.is_ident("kani") || p.is_ident("creusot"))
                        && ms.iter().skip(1).all(proof_meta);
                }
            }
        }
        false
    }
    attrs
        .iter()
        .filter(|a| !a.path().is_ident("doc") && !proof_meta(&a.meta))
        .map(tokens)
        .collect::<Vec<_>>()
        .join(" ")
}
fn trusted(a: &[syn::Attribute]) -> bool {
    a.iter().any(|a| {
        a.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "trusted")
            || (a.path().is_ident("cfg_attr") && tokens(a).contains("trusted"))
    })
}
fn known_attribute(meta: &syn::Meta) -> bool {
    let Some(name) = meta.path().segments.last().map(|s| s.ident.to_string()) else {
        return false;
    };
    if name == "cfg_attr" {
        use syn::parse::Parser;
        let syn::Meta::List(l) = meta else {
            return false;
        };
        let Ok(ms) = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
            .parse2(l.tokens.clone())
        else {
            return false;
        };
        return ms.len() >= 2
            && ms
                .iter()
                .skip(1)
                .all(|m| !m.path().is_ident("cfg") && known_attribute(m));
    }
    matches!(
        name.as_str(),
        "cfg"
            | "doc"
            | "allow"
            | "warn"
            | "deny"
            | "forbid"
            | "expect"
            | "inline"
            | "cold"
            | "must_use"
            | "deprecated"
            | "no_mangle"
            | "export_name"
            | "link_section"
            | "repr"
            | "non_exhaustive"
            | "requires"
            | "ensures"
            | "invariant"
            | "variant"
            | "maintains"
            | "trusted"
            | "terminates"
            | "open"
            | "logic"
            | "predicate"
            | "ghost"
            | "lemma"
            | "proof"
            | "proof_for_contract"
    )
}
struct Index {
    package: String,
    cfg: BTreeSet<String>,
    tool: Tool,
    targets: Vec<Target>,
    calls: Vec<(String, String, usize, usize)>,
    aliases: Vec<(String, String, usize, usize)>,
    incomplete: Vec<String>,
    visited: BTreeSet<PathBuf>,
}
impl Index {
    fn enabled(&mut self, a: &[syn::Attribute]) -> bool {
        for a in a {
            if a.path().is_ident("cfg") {
                if let syn::Meta::List(l) = &a.meta {
                    match syn::parse2::<syn::Meta>(l.tokens.clone())
                        .ok()
                        .and_then(|m| self.eval(&m))
                    {
                        Some(false) => return false,
                        Some(true) => {}
                        None => self
                            .incomplete
                            .push(format!("unsupported cfg: {}", tokens(a))),
                    }
                }
            }
        }
        for attr in a {
            if !known_attribute(&attr.meta) {
                self.incomplete.push(format!(
                    "attribute may generate or hide definitions; review required: {}",
                    tokens(attr)
                ));
            }
        }
        true
    }
    fn eval(&self, m: &syn::Meta) -> Option<bool> {
        match m {
            syn::Meta::Path(p) => {
                let s = tokens(p).replace(' ', "");
                Some(
                    self.cfg.contains(&s)
                        || (self.tool != Tool::Aeneas && s == self.tool.to_string()),
                )
            }
            syn::Meta::NameValue(_) => Some(self.cfg.contains(&tokens(m).replace(' ', ""))),
            syn::Meta::List(l) => {
                use syn::parse::Parser;
                let ms = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
                    .parse2(l.tokens.clone())
                    .ok()?;
                let vals: Option<Vec<_>> = ms.iter().map(|m| self.eval(m)).collect();
                let vals = vals?;
                if l.path.is_ident("all") {
                    Some(vals.into_iter().all(|v| v))
                } else if l.path.is_ident("any") {
                    Some(vals.into_iter().any(|v| v))
                } else if l.path.is_ident("not") && vals.len() == 1 {
                    Some(!vals[0])
                } else {
                    None
                }
            }
        }
    }
    fn file(&mut self, path: &Path, module: &[String], public: bool) -> Result<()> {
        let path = path
            .canonicalize()
            .with_context(|| format!("missing module {}", path.display()))?;
        if !self.visited.insert(path.clone()) {
            return Ok(());
        }
        let source = fs::read_to_string(&path)?;
        let file = syn::parse_file(&source).with_context(|| format!("parse {}", path.display()))?;
        let dir = if matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some("lib.rs" | "main.rs" | "mod.rs")
        ) {
            path.parent().unwrap().to_path_buf()
        } else {
            path.with_extension("")
        };
        self.items(&file.items, &path, &dir, module, public)
    }
    #[allow(clippy::too_many_arguments)] // Keeps source indexing fields visible at every definition kind.
    fn target(
        &mut self,
        file: &Path,
        module: &[String],
        name: &str,
        kind: Kind,
        vis: Visibility,
        attrs: &[syn::Attribute],
        spec: String,
        body: String,
        line: usize,
        col: usize,
        closed: bool,
    ) -> String {
        let id = format!(
            "{}::{}{}::{kind:?}",
            self.package,
            if module.is_empty() {
                String::new()
            } else {
                format!("{}::", module.join("::"))
            },
            name
        );
        let aux = auxiliary(attrs);
        let body_hash = hash(format!("{} {body}", runtime_attrs(attrs)));
        let spec_hash = hash(format!(
            "{} {spec} {}",
            attrs_hash(attrs),
            if aux { &body } else { "" }
        ));
        self.targets.push(Target {
            id: id.clone(),
            name: name.into(),
            kind,
            visibility: if auxiliary(attrs) {
                Visibility::Auxiliary
            } else {
                vis
            },
            file: file.to_string_lossy().into_owned(),
            line,
            column: col,
            aliases: vec![],
            spec_hash: spec_hash.clone(),
            model_base_hash: String::new(),
            model_inputs: BTreeMap::new(),
            body_hash: body_hash.clone(),
            source_hash: hash(format!("{spec_hash}{body}")),
            calls: BTreeSet::new(),
            unresolved_calls: vec![],
            required_units: if kind.proof() {
                vec!["contract".into()]
            } else {
                vec![]
            },
            closed_primitive: closed,
            trusted: trusted(attrs),
        });
        id
    }
    fn items(
        &mut self,
        items: &[syn::Item],
        file: &Path,
        dir: &Path,
        module: &[String],
        public: bool,
    ) -> Result<()> {
        for item in items {
            let attrs = match item {
                syn::Item::Fn(v) => &v.attrs,
                syn::Item::Struct(v) => &v.attrs,
                syn::Item::Enum(v) => &v.attrs,
                syn::Item::Trait(v) => &v.attrs,
                syn::Item::Impl(v) => &v.attrs,
                syn::Item::Mod(v) => &v.attrs,
                syn::Item::Use(v) => &v.attrs,
                syn::Item::Macro(v) => &v.attrs,
                _ => continue,
            };
            if !self.enabled(attrs) {
                continue;
            }
            let is_pub = |v: &syn::Visibility| public && matches!(v, syn::Visibility::Public(_));
            let visibility = |v: &syn::Visibility| {
                if is_pub(v) {
                    Visibility::Public
                } else {
                    Visibility::Internal
                }
            };
            match item {
                syn::Item::Fn(f) => {
                    let pos = f.sig.ident.span().start();
                    let id = self.target(
                        file,
                        module,
                        &f.sig.ident.to_string(),
                        Kind::Function,
                        visibility(&f.vis),
                        attrs,
                        tokens(&f.sig),
                        format!("{} {}", tokens(&f.sig), tokens(&f.block)),
                        pos.line,
                        pos.column,
                        closed_primitive(f),
                    );
                    self.calls(&id, file, &f.block);
                }
                syn::Item::Struct(v) => {
                    let p = v.ident.span().start();
                    self.target(
                        file,
                        module,
                        &v.ident.to_string(),
                        Kind::Struct,
                        visibility(&v.vis),
                        attrs,
                        format!("{} {}", tokens(&v.generics), tokens(&v.fields)),
                        format!("{} {}", tokens(&v.generics), tokens(&v.fields)),
                        p.line,
                        p.column,
                        false,
                    );
                }
                syn::Item::Enum(v) => {
                    let p = v.ident.span().start();
                    self.target(
                        file,
                        module,
                        &v.ident.to_string(),
                        Kind::Enum,
                        visibility(&v.vis),
                        attrs,
                        format!("{} {}", tokens(&v.generics), tokens(&v.variants)),
                        format!("{} {}", tokens(&v.generics), tokens(&v.variants)),
                        p.line,
                        p.column,
                        false,
                    );
                }
                syn::Item::Trait(v) => {
                    let p = v.ident.span().start();
                    let mut contract = tokens(v);
                    for i in &v.items {
                        if let syn::TraitItem::Fn(f) = i {
                            if let Some(b) = &f.default {
                                contract = contract.replace(&tokens(b), "");
                            }
                        }
                    }
                    self.target(
                        file,
                        module,
                        &v.ident.to_string(),
                        Kind::Trait,
                        visibility(&v.vis),
                        attrs,
                        contract,
                        String::new(),
                        p.line,
                        p.column,
                        false,
                    );
                    let mut m = module.to_vec();
                    m.push(v.ident.to_string());
                    for i in &v.items {
                        if let syn::TraitItem::Fn(f) = i {
                            if let Some(b) = &f.default {
                                let p = f.sig.ident.span().start();
                                let id = self.target(
                                    file,
                                    &m,
                                    &f.sig.ident.to_string(),
                                    Kind::Method,
                                    visibility(&v.vis),
                                    &f.attrs,
                                    tokens(&f.sig),
                                    format!("{} {}", tokens(&f.sig), tokens(b)),
                                    p.line,
                                    p.column,
                                    false,
                                );
                                self.calls(&id, file, b);
                            }
                        }
                    }
                }
                syn::Item::Impl(v) => {
                    let header = format!(
                        "{} {} {}",
                        tokens(&v.generics),
                        v.trait_
                            .as_ref()
                            .map(|(_, p, _)| tokens(p))
                            .unwrap_or_default(),
                        tokens(&v.self_ty)
                    );
                    let key = format!("impl[{}]", &hash(&header)[..16]);
                    let mut m = module.to_vec();
                    m.push(key.clone());
                    if v.trait_.is_some() {
                        let p = v.impl_token.span.start();
                        self.target(
                            file,
                            module,
                            &key,
                            Kind::TraitImpl,
                            Visibility::Internal,
                            attrs,
                            header.clone(),
                            String::new(),
                            p.line,
                            p.column,
                            false,
                        );
                    }
                    for i in &v.items {
                        if let syn::ImplItem::Fn(f) = i {
                            if !self.enabled(&f.attrs) {
                                continue;
                            }
                            let p = f.sig.ident.span().start();
                            let id = self.target(
                                file,
                                &m,
                                &f.sig.ident.to_string(),
                                Kind::Method,
                                if v.trait_.is_some() && public {
                                    Visibility::Public
                                } else {
                                    visibility(&f.vis)
                                },
                                &f.attrs,
                                format!("{header} {}", tokens(&f.sig)),
                                format!("{} {}", tokens(&f.sig), tokens(&f.block)),
                                p.line,
                                p.column,
                                false,
                            );
                            if let Some(t) = self.targets.iter_mut().find(|t| t.id == id) {
                                t.aliases.push(format!(
                                    "{}::{}",
                                    tokens(&v.self_ty).replace(' ', ""),
                                    f.sig.ident
                                ));
                            }
                            self.calls(&id, file, &f.block);
                        }
                    }
                }
                syn::Item::Mod(v) => {
                    let mut m = module.to_vec();
                    m.push(v.ident.to_string());
                    let sub = dir.join(v.ident.to_string());
                    if let Some((_, items)) = &v.content {
                        self.items(items, file, &sub, &m, is_pub(&v.vis))?;
                    } else {
                        let explicit = v.attrs.iter().find_map(|a| {
                            if a.path().is_ident("path") {
                                if let syn::Meta::NameValue(n) = &a.meta {
                                    if let syn::Expr::Lit(e) = &n.value {
                                        if let syn::Lit::Str(s) = &e.lit {
                                            return Some(file.parent().unwrap().join(s.value()));
                                        }
                                    }
                                }
                                None
                            } else {
                                None
                            }
                        });
                        let path = explicit.unwrap_or_else(|| {
                            if sub.with_extension("rs").exists() {
                                sub.with_extension("rs")
                            } else {
                                sub.join("mod.rs")
                            }
                        });
                        self.file(&path, &m, is_pub(&v.vis))?;
                    }
                }
                syn::Item::Use(v) if is_pub(&v.vis) => {
                    fn walk(tree: &syn::UseTree, ix: &mut Index, file: &Path, prefix: String) {
                        match tree {
                            syn::UseTree::Path(p) => {
                                walk(&p.tree, ix, file, format!("{prefix}{}::", p.ident))
                            }
                            syn::UseTree::Name(n) => {
                                let p = n.ident.span().start();
                                ix.aliases.push((
                                    format!("{prefix}{}", n.ident),
                                    file.to_string_lossy().into_owned(),
                                    p.line - 1,
                                    p.column,
                                ));
                            }
                            syn::UseTree::Rename(n) => {
                                let p = n.ident.span().start();
                                ix.aliases.push((
                                    n.rename.to_string(),
                                    file.to_string_lossy().into_owned(),
                                    p.line - 1,
                                    p.column,
                                ));
                            }
                            syn::UseTree::Group(g) => {
                                for t in &g.items {
                                    walk(t, ix, file, prefix.clone());
                                }
                            }
                            syn::UseTree::Glob(_) => ix
                                .incomplete
                                .push("glob re-export requires public reachability review".into()),
                        }
                    }
                    walk(&v.tree, self, file, String::new());
                }
                syn::Item::Macro(v) if !auxiliary(attrs) => {
                    self.incomplete.push(format!(
                        "macro-generated definitions need review: {}:{} ({})",
                        file.display(),
                        v.span().start().line,
                        tokens(&v.mac.path)
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn calls(&mut self, id: &str, file: &Path, block: &syn::Block) {
        struct Calls(Vec<(usize, usize)>);
        impl<'a> Visit<'a> for Calls {
            fn visit_expr_call(&mut self, e: &'a syn::ExprCall) {
                let p = match e.func.as_ref() {
                    syn::Expr::Path(p) => p.path.segments.last().unwrap().ident.span(),
                    _ => e.func.span(),
                }
                .start();
                self.0.push((p.line - 1, p.column));
                visit::visit_expr_call(self, e);
            }
            fn visit_expr_method_call(&mut self, e: &'a syn::ExprMethodCall) {
                let p = e.method.span().start();
                self.0.push((p.line - 1, p.column));
                visit::visit_expr_method_call(self, e);
            }
            fn visit_expr_macro(&mut self, e: &'a syn::ExprMacro) {
                let p = e.span().start();
                self.0.push((p.line - 1, p.column));
            }
        }
        let mut c = Calls(vec![]);
        c.visit_block(block);
        for (l, c) in c.0 {
            self.calls
                .push((id.into(), file.to_string_lossy().into_owned(), l, c));
        }
    }
}
// Deliberately narrow: complete evidence for an API is currently restricted to
// closed primitive free functions. A successful tool run for anything else is retained as partial evidence.
pub fn primitive(t: &syn::Type) -> bool {
    match t {
        syn::Type::Path(p) => {
            p.qself.is_none()
                && p.path.segments.len() == 1
                && matches!(
                    p.path.segments[0].ident.to_string().as_str(),
                    "bool"
                        | "char"
                        | "u8"
                        | "u16"
                        | "u32"
                        | "u64"
                        | "u128"
                        | "usize"
                        | "i8"
                        | "i16"
                        | "i32"
                        | "i64"
                        | "i128"
                        | "isize"
                )
        }
        syn::Type::Tuple(t) => t.elems.is_empty(),
        _ => false,
    }
}
pub fn closed_primitive(f: &syn::ItemFn) -> bool {
    if !f.sig.generics.params.is_empty()
        || f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.abi.is_some()
        || f.sig.variadic.is_some()
    {
        return false;
    }
    if let syn::ReturnType::Type(_, t) = &f.sig.output {
        if !primitive(t) {
            return false;
        }
    }
    let mut names = BTreeSet::new();
    for i in &f.sig.inputs {
        if let syn::FnArg::Typed(p) = i {
            if !primitive(&p.ty) {
                return false;
            }
            if let syn::Pat::Ident(i) = p.pat.as_ref() {
                names.insert(i.ident.to_string());
            } else {
                return false;
            }
        } else {
            return false;
        }
    }
    struct Closed {
        names: BTreeSet<String>,
        ok: bool,
    }
    impl<'a> Visit<'a> for Closed {
        fn visit_local(&mut self, l: &'a syn::Local) {
            if let Some(i) = &l.init {
                self.visit_expr(&i.expr);
                if i.diverge.is_some() {
                    self.ok = false;
                }
            }
            match &l.pat {
                syn::Pat::Ident(i) => {
                    self.names.insert(i.ident.to_string());
                }
                syn::Pat::Type(t) if primitive(&t.ty) => {
                    if let syn::Pat::Ident(i) = t.pat.as_ref() {
                        self.names.insert(i.ident.to_string());
                    } else {
                        self.ok = false;
                    }
                }
                _ => self.ok = false,
            }
        }
        fn visit_expr(&mut self, e: &'a syn::Expr) {
            if !match e {
                syn::Expr::Binary(_)
                | syn::Expr::Unary(_)
                | syn::Expr::Paren(_)
                | syn::Expr::Group(_)
                | syn::Expr::Lit(_)
                | syn::Expr::If(_)
                | syn::Expr::Block(_)
                | syn::Expr::Return(_)
                | syn::Expr::Assign(_)
                | syn::Expr::Loop(_)
                | syn::Expr::While(_)
                | syn::Expr::Break(_)
                | syn::Expr::Continue(_) => true,
                syn::Expr::Path(p) => {
                    p.qself.is_none()
                        && p.path.segments.len() == 1
                        && self.names.contains(&p.path.segments[0].ident.to_string())
                }
                syn::Expr::Tuple(t) => t.elems.is_empty(),
                syn::Expr::Cast(c) => primitive(&c.ty),
                _ => false,
            } {
                self.ok = false;
            }
            visit::visit_expr(self, e);
        }
        fn visit_attribute(&mut self, _: &'a syn::Attribute) {
            self.ok = false;
        }
        fn visit_item(&mut self, _: &'a syn::Item) {
            self.ok = false;
        }
    }
    let mut c = Closed { names, ok: true };
    c.visit_block(&f.block);
    c.ok
}

pub struct Ra {
    child: Child,
    input: ChildStdin,
    rx: Receiver<Value>,
    seq: u64,
    opened: BTreeMap<String, i32>,
    pub scope: Scope,
    quiescent: bool,
}
impl Drop for Ra {
    fn drop(&mut self) {
        process::kill_group(self.child.id());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Ra {
    pub fn start(scope: &Scope, tool: Tool) -> Result<Self> {
        let mut cmd = Command::new("rust-analyzer");
        process::group(&mut cmd);
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("rust-analyzer is required; rustup component add rust-analyzer rust-src")?;
        let input = child.stdin.take().unwrap();
        let mut out = BufReader::new(child.stdout.take().unwrap());
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    if out.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(l) = line.strip_prefix("Content-Length:") {
                        length = l.trim().parse().unwrap_or(0);
                    }
                }
                if length == 0 || length > 64 * 1024 * 1024 {
                    return;
                }
                let mut b = vec![0; length];
                if out.read_exact(&mut b).is_err() {
                    return;
                }
                if let Ok(v) = serde_json::from_slice(&b) {
                    if tx.send(v).is_err() {
                        return;
                    }
                }
            }
        });
        let mut ra = Self {
            child,
            input,
            rx,
            seq: 0,
            opened: BTreeMap::new(),
            scope: scope.clone(),
            quiescent: false,
        };
        let root = url::Url::from_directory_path(&scope.workspace_root)
            .map_err(|_| anyhow::anyhow!("invalid workspace path"))?;
        ra.request("initialize",json!({"processId":std::process::id(),"rootUri":root,"capabilities":{"experimental":{"serverStatusNotification":true}},"initializationOptions":{"cargo":{"features":if scope.all_features{json!("all")}else{json!(scope.features)},"noDefaultFeatures":scope.no_default_features,"target":scope.target,"cfgs":if tool == Tool::Aeneas {vec![]} else {vec![tool.to_string()]}},"checkOnSave":false,"diagnostics":{"enable":false}}}))?;
        ra.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}))?;
        ra.settle()?;
        Ok(ra)
    }
    fn send(&mut self, v: Value) -> Result<()> {
        let b = serde_json::to_vec(&v)?;
        write!(self.input, "Content-Length: {}\r\n\r\n", b.len())?;
        self.input.write_all(&b)?;
        self.input.flush()?;
        Ok(())
    }
    fn receive(&mut self, timeout: Duration) -> Result<Value> {
        let v = self
            .rx
            .recv_timeout(timeout)
            .context("rust-analyzer did not respond")?;
        if v["method"] == "experimental/serverStatus" {
            self.quiescent = v["params"]["quiescent"] == true;
        }
        if v.get("method").is_some() && v.get("id").is_some() {
            let result = if v["method"] == "workspace/configuration" {
                Value::Array(
                    v["params"]["items"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|_| Value::Null)
                        .collect(),
                )
            } else {
                Value::Null
            };
            self.send(json!({"jsonrpc":"2.0","id":v["id"],"result":result}))?;
        }
        Ok(v)
    }
    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.seq += 1;
        let id = self.seq;
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))?;
        let until = Instant::now() + Duration::from_secs(120);
        loop {
            if process::interrupted() {
                bail!("interrupted");
            }
            let v = self.receive(until.saturating_duration_since(Instant::now()))?;
            if v["id"] == id {
                if let Some(e) = v.get("error") {
                    bail!("rust-analyzer {method}: {e}");
                }
                return Ok(v["result"].clone());
            }
        }
    }
    fn settle(&mut self) -> Result<()> {
        let end = Instant::now() + Duration::from_secs(120);
        loop {
            if process::interrupted() {
                bail!("interrupted");
            }
            let wait = if self.quiescent {
                Duration::from_millis(150)
            } else {
                Duration::from_millis(1000)
            };
            match self.receive(wait) {
                Ok(_) => {}
                Err(e) => {
                    if self.quiescent {
                        return Ok(());
                    }
                    if Instant::now() >= end {
                        return Err(e.context("rust-analyzer workspace loading timeout"));
                    }
                }
            }
        }
    }
    fn open(&mut self, path: &Path) -> Result<()> {
        let uri = url::Url::from_file_path(path)
            .map_err(|_| anyhow::anyhow!("invalid file path"))?
            .to_string();
        let n = self.opened.get(&uri).copied().unwrap_or(0) + 1;
        if n > 1 {
            self.send(json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}))?;
        }
        self.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"rust","version":n,"text":fs::read_to_string(path)?}}}))?;
        self.opened.insert(uri, n);
        Ok(())
    }
    fn definition(
        &mut self,
        file: &str,
        line: usize,
        column: usize,
    ) -> Result<Vec<(String, usize)>> {
        let text = fs::read_to_string(file)?;
        let utf16 = text
            .lines()
            .nth(line)
            .unwrap_or("")
            .get(..column)
            .unwrap_or("")
            .encode_utf16()
            .count();
        let uri =
            url::Url::from_file_path(file).map_err(|_| anyhow::anyhow!("invalid file path"))?;
        let v = self.request(
            "textDocument/definition",
            json!({"textDocument":{"uri":uri},"position":{"line":line,"character":utf16}}),
        )?;
        let vs = if v.is_array() {
            v.as_array().unwrap().clone()
        } else if v.is_object() {
            vec![v]
        } else {
            vec![]
        };
        Ok(vs
            .iter()
            .filter_map(|v| {
                let uri = v["uri"].as_str().or_else(|| v["targetUri"].as_str())?;
                let p = url::Url::parse(uri).ok()?.to_file_path().ok()?;
                let line = v
                    .pointer("/range/start/line")
                    .or_else(|| v.pointer("/targetSelectionRange/start/line"))?
                    .as_u64()?;
                Some((p.to_string_lossy().into_owned(), line as usize))
            })
            .collect())
    }
}
