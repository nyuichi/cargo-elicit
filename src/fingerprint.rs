use crate::{model::{Scope,Tool},process};
use anyhow::{Result, bail};
use sha2::{Digest,Sha256};
use std::{collections::BTreeSet,fs,path::{Path,PathBuf},process::Command,time::Duration};
use walkdir::WalkDir;
pub fn hash(bytes: impl AsRef<[u8]>) -> String { format!("{:x}",Sha256::digest(bytes.as_ref())) }
pub fn version(program:&str,args:&[&str])->Result<String> {
    let o=process::run(Command::new(program).args(args),Duration::from_secs(30))?;
    if o.code!=Some(0) { bail!("{program} {} unavailable: {}",args.join(" "),o.stderr.lines().take(3).collect::<Vec<_>>().join(" ")); }
    Ok(o.stdout.trim().to_string())
}
pub fn tool_version(tool:Tool)->Result<String> {
    let v=version("cargo",&[&tool.to_string(),"--version"]).or_else(|_|version("cargo",&[&tool.to_string(),"version"]))?;
    match tool {
        Tool::Kani if !v.contains("0.68.")=>bail!("unsupported Kani version {v}; adapter requires 0.68.x structured JSON output"),
        Tool::Creusot if !v.contains("0.14.")=>bail!("unsupported Creusot version {v}; adapter requires 0.14.x Coma/why3find output"),_=>{}
    } Ok(v)
}
pub fn input(scope:&Scope,tool:Tool,version:&str)->Result<String> {
    let mut files=BTreeSet::new(); let target=Path::new(&scope.target_directory);
    for root in &scope.local_roots {
        for e in WalkDir::new(root).into_iter().filter_entry(|e| {
            e.path()!=target && !matches!(e.file_name().to_str(),Some(".git"|".elicit"|".why3find"))
        }) {
            let e=e?; if e.file_type().is_symlink() {bail!("untracked symlink input: {}",e.path().display());}
            if e.file_type().is_file() { files.insert(e.path().to_path_buf()); }
        }
    }
    for ancestor in Path::new(&scope.manifest_path).ancestors().skip(1) {
        for name in ["Cargo.lock","Cargo.toml","rust-toolchain","rust-toolchain.toml",".cargo/config",".cargo/config.toml"] {let p=ancestor.join(name);if p.is_file(){files.insert(p);}}
    }
    // User Cargo configuration can affect rustflags and dependencies too.
    if let Some(home)=std::env::var_os("CARGO_HOME").map(PathBuf::from).or_else(||std::env::var_os("HOME").map(|p|PathBuf::from(p).join(".cargo"))) {
        for name in ["config","config.toml"] {let p=home.join(name);if p.is_file(){files.insert(p);}}
    }
    let mut h=Sha256::new();
    for f in files {
        if f.file_name().is_some_and(|v|v=="elicit.toml") {continue;} // completion policy is not a verifier input
        h.update(f.to_string_lossy().as_bytes());h.update([0]);h.update(fs::read(f)?);h.update([0]);
    }
    h.update(serde_json::to_vec(scope)?);h.update(tool.to_string());h.update(version);h.update(env!("CARGO_PKG_VERSION"));
    h.update(version_cached()?);
    for (k,v) in std::env::vars().filter(|(k,_)| k=="RUSTUP_TOOLCHAIN" || k=="RUSTFLAGS" || k=="RUSTDOCFLAGS" || k=="CARGO_ENCODED_RUSTFLAGS" || k=="RUSTC" || k=="RUSTC_WRAPPER" || k=="RUSTC_WORKSPACE_WRAPPER" || k.starts_with("CARGO_FEATURE_") || k.starts_with("CARGO_TARGET_") || k=="CREUSOT_DATA_HOME") {h.update(k);h.update(v);}
    Ok(format!("{:x}",h.finalize()))
}
fn version_cached()->Result<String>{version("rustc",&["-vV"])}
