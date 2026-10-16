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
    let mut h = Sha256::new();
    for f in files {
        if f.file_name().is_some_and(|v| v == "elicit.toml") {
            continue;
        } // completion policy is not a verifier input
        h.update(f.to_string_lossy().as_bytes());
        h.update([0]);
        h.update(fs::read(f)?);
        h.update([0]);
    }
    h.update(serde_json::to_vec(scope)?);
    h.update(tool.to_string());
    h.update(version);
    h.update(env!("CARGO_PKG_VERSION"));
    h.update(hash(fs::read(std::env::current_exe()?)?));
    h.update(version_cached()?);
    tcb_inputs(tool, &mut h)?;
    for (k, v) in std::env::vars().filter(|(k, _)| {
        k == "RUSTUP_TOOLCHAIN"
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
            || k.starts_with("CARGO_BUILD_")
    }) {
        h.update(k);
        h.update(v);
    }
    Ok(format!("{:x}", h.finalize()))
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
        Tool::Kani => kani_home().is_some(),
        Tool::Creusot => {
            creusot_home().is_some_and(|p| p.join("share/why3find/packages/creusot").is_dir())
        }
    }
}
fn tcb_inputs(tool: Tool, h: &mut Sha256) -> Result<()> {
    let roots = match tool {
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
                h.update(fs::read(root)?);
            } else {
                let mut files = WalkDir::new(root)
                    .follow_links(true)
                    .into_iter()
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .into_iter()
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.into_path())
                    .collect::<Vec<_>>();
                files.sort();
                for file in files {
                    h.update(file.to_string_lossy().as_bytes());
                    h.update(fs::read(file)?);
                }
            }
        }
    }
    for name in ["cargo-kani", "cargo-creusot"] {
        if let Some(p) = executable(name) {
            h.update(fs::read(p)?);
        }
    }
    Ok(())
}
