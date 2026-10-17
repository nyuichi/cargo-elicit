use crate::{
    model::{Scope, Tool},
    process,
};
use anyhow::{Result, bail};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use walkdir::WalkDir;
pub fn hash(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}
pub fn version(program: &str, args: &[&str]) -> Result<String> {
    let o = process::run(Command::new(program).args(args), Duration::from_secs(30))?;
    if o.code != Some(0) {
        bail!(
            "{program} {} unavailable: {}",
            args.join(" "),
            o.stderr.lines().take(3).collect::<Vec<_>>().join(" ")
        );
    }
    Ok(o.stdout.trim().to_string())
}
pub fn tool_version(tool: Tool) -> Result<String> {
    let v = match tool {
        Tool::Aeneas => crate::aeneas::version(),
        Tool::Kani => version("cargo", &["kani", "--version"]),
        Tool::Creusot => version("cargo", &["creusot", "version"]),
    }?;
    match tool {
        Tool::Kani if !v.contains("0.68.") => {
            bail!("unsupported Kani version {v}; adapter requires 0.68.x structured JSON output")
        }
        Tool::Creusot if !v.contains("0.14.") => {
            bail!("unsupported Creusot version {v}; adapter requires 0.14.x Coma/why3find output")
        }
        _ => {}
    }
    Ok(v)
}
pub fn input(scope: &Scope, tool: Tool, version: &str) -> Result<String> {
    Ok(inputs(scope, tool, version)?.0)
}
// Keep proof-module edits separate from the context used to retry blocked work.
pub fn inputs(
    scope: &Scope,
    tool: Tool,
    version: &str,
) -> Result<(String, String, std::collections::BTreeMap<String, String>)> {
    let mut files = BTreeSet::new();
    let target = Path::new(&scope.target_directory);
    for root in &scope.local_roots {
        for e in WalkDir::new(root).into_iter().filter_entry(|e| {
            e.path() != target
                && !matches!(
                    e.file_name().to_str(),
                    Some(".git" | ".elicit" | ".why3find")
                )
        }) {
            let e = e?;
            if e.file_type().is_symlink() {
                bail!("untracked symlink input: {}", e.path().display());
            }
            if e.file_type().is_file() {
                files.insert(e.path().to_path_buf());
            }
        }
    }
    for ancestor in Path::new(&scope.manifest_path).ancestors().skip(1) {
        for name in [
            "Cargo.lock",
            "Cargo.toml",
            "rust-toolchain",
            "rust-toolchain.toml",
            ".cargo/config",
            ".cargo/config.toml",
        ] {
            let p = ancestor.join(name);
            if p.is_file() {
                files.insert(p);
            }
        }
    }
    // User Cargo configuration can affect rustflags and dependencies too.
    if let Some(home) = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".cargo")))
    {
        for name in ["config", "config.toml"] {
            let p = home.join(name);
            if p.is_file() {
                files.insert(p);
            }
        }
    }
    let reports_are_independent = reports_are_independent(scope, &files)?;
    let mut input_files = std::collections::BTreeMap::new();
    let mut h = Sha256::new();
    let mut proofs = Sha256::new();
    let proof_root = Path::new(&scope.manifest_path)
        .parent()
        .unwrap()
        .join("elicit-proofs");
    for f in files {
        if f.file_name().is_some_and(|v| v == "elicit.toml") {
            continue;
        } // completion policy is not a verifier input
        if reports_are_independent
            && scope.local_roots.iter().any(|root| {
                f.parent() == Some(Path::new(root))
                    && matches!(
                        f.file_name().and_then(|n| n.to_str()),
                        Some("PROGRESS.md" | "verification-inventory.json")
                    )
            })
        {
            continue;
        }
        let digest = if tool == Tool::Aeneas && f.starts_with(&proof_root) {
            &mut proofs
        } else {
            &mut h
        };
        digest.update(f.to_string_lossy().as_bytes());
        digest.update([0]);
        let bytes = fs::read(&f)?;
        input_files.insert(f.to_string_lossy().into_owned(), hash(&bytes));
        digest.update(bytes);
        digest.update([0]);
    }
    let scope_bytes = serde_json::to_vec(scope)?;
    h.update(&scope_bytes);
    input_files.insert("@scope".into(), hash(&scope_bytes));
    h.update(tool.to_string());
    h.update(version);
    input_files.insert("@verifier_version".into(), hash(version));
    h.update(env!("CARGO_PKG_VERSION"));
    let binary = hash(fs::read(std::env::current_exe()?)?);
    h.update(&binary);
    input_files.insert("@binary".into(), binary);
    let compiler = version_cached()?;
    h.update(&compiler);
    input_files.insert("@rustc_version".into(), hash(compiler));
    let mut tcb = Sha256::new();
    tcb_inputs(scope, tool, &mut tcb)?;
    let tcb = format!("{:x}", tcb.finalize());
    h.update(&tcb);
    input_files.insert("@installed_tcb".into(), tcb);
    let environment = environment_inputs(tool, reports_are_independent, std::env::vars());
    for (key, value) in &environment {
        input_files.insert(format!("@env:{key}"), hash(value));
    }
    h.update(serde_json::to_vec(&environment)?);
    let context = format!("{:x}", h.finalize());
    proofs.update(context.as_bytes());
    Ok((format!("{:x}", proofs.finalize()), context, input_files))
}
// Canonicalize process environment order and selectors which cannot affect this
// invocation. Aeneas always overwrites LEAN_PATH. For simple crates without
// compile-time environment reads, the resolved compiler version and pinned TCB
// replace Rustup's Cargo-only selector variable. Complex crates keep its bytes.
fn environment_inputs(
    tool: Tool,
    simple: bool,
    vars: impl IntoIterator<Item = (String, String)>,
) -> std::collections::BTreeMap<String, String> {
    vars.into_iter()
        .filter(|(k, _)| {
            !(tool == Tool::Aeneas && (k == "LEAN_PATH" || (simple && k == "RUSTUP_TOOLCHAIN")))
                && (k == "AENEAS_HOME"
                    || k.starts_with("LEAN_")
                    || k.starts_with("ELAN_")
                    || k == "RUSTUP_TOOLCHAIN"
                    || k == "RUSTFLAGS"
                    || k == "RUSTDOCFLAGS"
                    || k == "CARGO_ENCODED_RUSTFLAGS"
                    || k == "RUSTC"
                    || k == "RUSTC_WRAPPER"
                    || k == "RUSTC_WORKSPACE_WRAPPER"
                    || k.starts_with("CARGO_FEATURE_")
                    || k.starts_with("CARGO_TARGET_")
                    || k == "CREUSOT_DATA_HOME"
                    || k.starts_with("WHY3")
                    || k.starts_with("CARGO_PROFILE_")
                    || k.starts_with("CARGO_BUILD_"))
        })
        .collect()
}

// Only ignore the two named operational reports in demonstrably simple crates.
// Arbitrary files, build scripts, dependencies/proc macros, file-input macros and
// custom compiler hooks retain conservative whole-package invalidation.
fn dependency_active(
    manifest: &toml::Value,
    scope: &Scope,
    name: &str,
    dependency: &toml::Value,
) -> bool {
    if dependency.get("optional").and_then(|v| v.as_bool()) != Some(true) {
        return true;
    }
    scope
        .resolved_features
        .iter()
        .chain(&scope.features)
        .any(|feature| {
            feature == name
                || feature == &format!("dep:{name}")
                || manifest
                    .get("features")
                    .and_then(|v| v.get(feature))
                    .and_then(|v| v.as_array())
                    .is_some_and(|values| {
                        values.iter().filter_map(|v| v.as_str()).any(|v| {
                            v == name
                                || v == format!("dep:{name}")
                                || v.starts_with(&format!("{name}/"))
                                || v.starts_with(&format!("{name}?/"))
                        })
                    })
        })
}
pub(crate) fn reports_are_independent(scope: &Scope, files: &BTreeSet<PathBuf>) -> Result<bool> {
    use syn::visit::Visit;
    struct Macros(bool);
    impl<'ast> Visit<'ast> for Macros {
        fn visit_macro(&mut self, m: &'ast syn::Macro) {
            let name = m
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            if ![
                "assert",
                "assert_eq",
                "assert_ne",
                "debug_assert",
                "debug_assert_eq",
                "debug_assert_ne",
                "panic",
                "unreachable",
                "unimplemented",
                "todo",
                "format_args",
                "write",
                "writeln",
                "vec",
                "matches",
                "cfg",
            ]
            .contains(&name.as_str())
            {
                self.0 = false;
            }
            syn::visit::visit_macro(self, m);
        }
    }
    fn file_input(tokens: proc_macro2::TokenStream) -> bool {
        tokens.into_iter().any(|t| match t {
            proc_macro2::TokenTree::Ident(i) => [
                "path",
                "include",
                "include_str",
                "include_bytes",
                "env",
                "option_env",
            ]
            .contains(&i.to_string().as_str()),
            proc_macro2::TokenTree::Group(g) => file_input(g.stream()),
            _ => false,
        })
    }
    if [
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
    ]
    .iter()
    .any(|k| std::env::var_os(k).is_some_and(|v| !v.is_empty()))
    {
        return Ok(false);
    }
    fn compiler_hook(v: &toml::Value) -> bool {
        v.as_table().is_some_and(|t| {
            t.iter().any(|(k, v)| {
                [
                    "rustc",
                    "rustdoc",
                    "rustc-wrapper",
                    "rustc-workspace-wrapper",
                    "rustflags",
                    "rustdocflags",
                    "linker",
                ]
                .contains(&k.as_str())
                    || compiler_hook(v)
            })
        })
    }
    for f in files.iter().filter(|f| {
        matches!(
            f.file_name().and_then(|n| n.to_str()),
            Some("config" | "config.toml")
        )
    }) {
        let Ok(config) = toml::from_str::<toml::Value>(&fs::read_to_string(f)?) else {
            return Ok(false);
        };
        if compiler_hook(&config) {
            return Ok(false);
        }
    }
    let proof_root = Path::new(&scope.manifest_path)
        .parent()
        .unwrap()
        .join("elicit-proofs");
    if proof_root.exists() {
        for e in WalkDir::new(&proof_root) {
            let e = e?;
            if e.path().extension().is_some_and(|x| x == "lean") {
                let source = fs::read_to_string(e.path())?;
                let words: Vec<_> = source
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .collect();
                if words.iter().any(|w| {
                    [
                        "IO",
                        "System",
                        "readFile",
                        "readBinFile",
                        "readDir",
                        "getEnv",
                        "run_cmd",
                        "run_tac",
                        "run_meta",
                        "run_elab",
                        "unsafe",
                        "implemented_by",
                        "extern",
                        "elab",
                        "macro",
                        "syntax",
                        "initialize",
                        "builtin_initialize",
                        "eval",
                        "include_str",
                    ]
                    .contains(w)
                }) {
                    return Ok(false);
                }
                for line in source
                    .lines()
                    .filter(|l| l.trim_start().starts_with("import "))
                {
                    for module in line.split_whitespace().skip(1) {
                        if ![
                            "ElicitGenerated",
                            "ElicitContracts",
                            "Specs",
                            "Aeneas",
                            "Mathlib",
                            "Std",
                            "Lean",
                            "Init",
                        ]
                        .iter()
                        .any(|m| module == *m || module.starts_with(&format!("{m}.")))
                        {
                            return Ok(false);
                        }
                    }
                }
            }
        }
    }
    for root in &scope.local_roots {
        let root = Path::new(root);
        let m: toml::Value = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)?;
        if root.join("build.rs").exists()
            || m.get("package")
                .and_then(|p| p.get("build"))
                .is_some_and(|v| v.as_bool() != Some(false))
            || m.get("lib")
                .and_then(|p| p.get("proc-macro"))
                .and_then(|v| v.as_bool())
                == Some(true)
            || ["target", "patch", "replace", "workspace"].iter().any(|k| {
                m.get(k)
                    .is_some_and(|v| v.as_table().is_none_or(|v| !v.is_empty()))
            })
        {
            return Ok(false);
        }
        // Default verification builds the library/binaries, not Cargo benches,
        // integration tests or examples. Custom target paths stay conservative.
        if m.get("lib")
            .and_then(|v| v.get("path"))
            .and_then(|v| v.as_str())
            .is_some_and(|p| !p.starts_with("src/"))
            || m.get("bin").and_then(|v| v.as_array()).is_some_and(|bins| {
                bins.iter().any(|b| {
                    b.get("path")
                        .and_then(|v| v.as_str())
                        .is_some_and(|p| !p.starts_with("src/"))
                })
            })
        {
            return Ok(false);
        }
        for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(ds) = m.get(key).and_then(|v| v.as_table()) {
                if ds
                    .iter()
                    .any(|(name, v)| dependency_active(&m, scope, name, v))
                {
                    return Ok(false);
                }
            }
        }
        if !root.join("src").is_dir() {
            return Ok(false);
        }
        for e in WalkDir::new(root.join("src")) {
            let e = e?;
            if e.path().extension().is_some_and(|e| e == "rs") {
                let source = fs::read_to_string(e.path())?;
                let Ok(ast) = syn::parse_file(&source) else {
                    return Ok(false);
                };
                let mut macros = Macros(true);
                macros.visit_file(&ast);
                if !macros.0 || file_input(source.parse().unwrap_or_default()) {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

fn version_cached() -> Result<String> {
    version("rustc", &["-vV"])
}

pub fn executable(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|p| p.join(name))
        .find(|p| p.is_file())
        .and_then(|p| p.canonicalize().ok())
}
pub fn kani_home() -> Option<PathBuf> {
    if let Some(bin) = executable("cargo-kani") {
        if let Some(root) = bin.parent().and_then(Path::parent) {
            if root.join("library/kani").is_dir() || root.join("library/kani_core").is_dir() {
                return Some(root.into());
            }
        }
    }
    let root = PathBuf::from(std::env::var_os("HOME")?).join(".kani/kani-0.68.0");
    root.is_dir().then_some(root)
}
pub fn creusot_home() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CREUSOT_DATA_HOME") {
        return Some(PathBuf::from(path));
    }
    // Match Creusot's directories::ProjectDirs::from("", "creusot", "creusot").
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join("Library/Application Support/creusot.creusot"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(|p| PathBuf::from(p).join("creusot"))
            .or_else(|| {
                std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share/creusot"))
            })
    }
}
pub fn tcb_tracked(tool: Tool) -> bool {
    match tool {
        Tool::Aeneas => crate::aeneas::home().is_ok(),
        Tool::Kani => kani_home().is_some(),
        Tool::Creusot => {
            creusot_home().is_some_and(|p| p.join("share/why3find/packages/creusot").is_dir())
        }
    }
}
fn tcb_inputs(scope: &Scope, tool: Tool, h: &mut Sha256) -> Result<()> {
    let cache_path = Path::new(&scope.manifest_path)
        .parent()
        .unwrap()
        .join(".elicit/cache/tcb-digests.json");
    let mut digests = if tool == Tool::Aeneas {
        InstalledDigests::load(cache_path)
    } else {
        InstalledDigests::default()
    };
    let roots = match tool {
        Tool::Aeneas => Some(crate::aeneas::tcb_roots()?),
        Tool::Kani => kani_home().map(|p| {
            vec![
                p.join("library"),
                p.join("lib"),
                p.join("bin"),
                p.join("rust-toolchain-version"),
            ]
        }),
        Tool::Creusot => creusot_home().map(|p| {
            let mut roots = vec![
                p.join("share/why3find/packages/creusot"),
                p.join("bin"),
                p.join("toolchains"),
                p.join("creusot_why3.conf"),
            ];
            if let Some(prefix) = p
                .join("bin/why3")
                .canonicalize()
                .ok()
                .and_then(|p| p.parent()?.parent().map(Path::to_path_buf))
            {
                roots.extend([prefix.join("share/why3"), prefix.join("lib/why3")]);
            }
            roots
        }),
    };
    if let Some(roots) = roots {
        for root in roots {
            if !root.exists() {
                continue;
            }
            if root.is_file() {
                h.update(digests.read(&root, tool)?);
            } else {
                let mut files = WalkDir::new(root)
                    .follow_links(true)
                    .into_iter()
                    .filter_entry(|e| e.file_name() != ".git")
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .into_iter()
                    .filter(|e| {
                        e.file_type().is_file()
                            && (tool != Tool::Aeneas || crate::aeneas::tcb_file(e.path()))
                    })
                    .map(|e| e.into_path())
                    .collect::<Vec<_>>();
                files.sort();
                for file in files {
                    h.update(file.to_string_lossy().as_bytes());
                    h.update(digests.read(&file, tool)?);
                }
            }
        }
    }
    digests.save();
    for name in if tool == Tool::Aeneas {
        vec!["aeneas", "charon", "charon-driver", "lake"]
    } else {
        vec!["cargo-kani", "cargo-creusot"]
    } {
        if let Some(p) = executable(name) {
            h.update(fs::read(p)?);
        }
    }
    Ok(())
}

// Installed Lean environments can contain gigabytes of immutable proof objects.
// Cache only their content digests, guarded by Unix inode, size, mtime AND ctime.
// Source inputs are always read directly. This is an acceleration cache, never
// proof evidence; deleting it reproduces exactly the same fingerprint.
#[derive(Default)]
struct InstalledDigests {
    path: Option<PathBuf>,
    entries: std::collections::BTreeMap<String, (Vec<u64>, String)>,
    changed: bool,
}
impl InstalledDigests {
    fn load(path: PathBuf) -> Self {
        Self {
            entries: fs::read(&path)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default(),
            path: Some(path),
            changed: false,
        }
    }
    fn read(&mut self, path: &Path, tool: Tool) -> Result<Vec<u8>> {
        if tool != Tool::Aeneas {
            return Ok(fs::read(path)?);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let m = fs::metadata(path)?;
            let stamp = vec![
                m.dev(),
                m.ino(),
                m.size(),
                m.mtime() as u64,
                m.mtime_nsec() as u64,
                m.ctime() as u64,
                m.ctime_nsec() as u64,
            ];
            let key = path.to_string_lossy().into_owned();
            if let Some((old, digest)) = self.entries.get(&key) {
                if old == &stamp {
                    return Ok(digest.as_bytes().to_vec());
                }
            }
            let digest = hash(fs::read(path)?);
            self.entries.insert(key, (stamp, digest.clone()));
            self.changed = true;
            Ok(digest.into_bytes())
        }
        #[cfg(not(unix))]
        {
            Ok(hash(fs::read(path)?).into_bytes())
        }
    }
    fn save(&self) {
        if !self.changed {
            return;
        }
        // Cache failures must not alter proof semantics or prevent verification.
        let _ = (|| -> Result<()> {
            let Some(path) = &self.path else {
                return Ok(());
            };
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent)?;
            let mut file = tempfile::NamedTempFile::new_in(parent)?;
            use std::io::Write;
            file.write_all(&serde_json::to_vec(&self.entries)?)?;
            file.persist(path)?;
            Ok(())
        })();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn installed_digest_cache_detects_same_size_edits_with_restored_mtime() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("model.olean");
        fs::write(&path, "old").unwrap();
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        let cache = d.path().join("digests.json");
        let mut digests = InstalledDigests::load(cache.clone());
        let original = digests.read(&path, Tool::Aeneas).unwrap();
        digests.save();
        let mut digests = InstalledDigests::load(cache);
        assert_eq!(original, digests.read(&path, Tool::Aeneas).unwrap());
        fs::write(&path, "new").unwrap();
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(modified))
            .unwrap();
        assert_ne!(original, digests.read(&path, Tool::Aeneas).unwrap());
        assert_eq!(
            digests.read(&path, Tool::Aeneas).unwrap(),
            hash(b"new").into_bytes()
        );
    }
}

#[cfg(test)]
mod report_dependencies {
    use super::*;
    #[test]
    fn namespaced_optional_dependencies_are_not_mistaken_for_inactive_inputs() {
        let manifest: toml::Value = toml::from_str("[features]\ndefault=['enabled']\nenabled=['dep:external']\n[dependencies.external]\nversion='1'\noptional=true\n").unwrap();
        let mut scope = Scope {
            resolved_features: vec!["default".into(), "enabled".into()],
            ..Scope::default()
        };
        assert!(dependency_active(
            &manifest,
            &scope,
            "external",
            &manifest["dependencies"]["external"]
        ));
        scope.resolved_features.clear();
        assert!(!dependency_active(
            &manifest,
            &scope,
            "external",
            &manifest["dependencies"]["external"]
        ));
        scope.features.push("dep:external".into());
        assert!(dependency_active(
            &manifest,
            &scope,
            "external",
            &manifest["dependencies"]["external"]
        ));
    }
}

#[cfg(test)]
mod environment_regressions {
    use super::*;
    #[test]
    fn order_and_overridden_paths_do_not_change_effective_inputs() {
        let a = vec![
            ("AENEAS_HOME".into(), "aeneas".into()),
            ("ELAN_HOME".into(), "elan".into()),
            ("LEAN_PATH".into(), "scratch".into()),
            ("RUSTUP_TOOLCHAIN".into(), "stable".into()),
        ];
        let mut b = a.clone();
        b.reverse();
        assert_eq!(
            environment_inputs(Tool::Aeneas, true, a.clone()),
            environment_inputs(Tool::Aeneas, true, b)
        );
        assert_eq!(
            environment_inputs(Tool::Aeneas, true, a.clone()),
            environment_inputs(
                Tool::Aeneas,
                true,
                a.iter()
                    .filter(|(k, _)| k != "RUSTUP_TOOLCHAIN" && k != "LEAN_PATH")
                    .cloned()
            )
        );
        assert_ne!(
            environment_inputs(Tool::Aeneas, false, a.clone()),
            environment_inputs(
                Tool::Aeneas,
                false,
                a.into_iter().filter(|(k, _)| k != "RUSTUP_TOOLCHAIN")
            )
        );
    }
}
