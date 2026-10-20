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
    // Observation only: native commands own version compatibility and proof policy.
    let observed = match tool {
        Tool::Aeneas => version("lean", &["--version"]),
        Tool::Kani => version("cargo", &["kani", "--version"]),
        Tool::Creusot => version("cargo", &["creusot", "version"]),
    }
    .unwrap_or_else(|_| "not observed".into());
    Ok(format!("native-command-v1; {tool}: {observed}"))
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
                    Some(".git" | ".elicit" | ".why3find" | ".lake")
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
    let config_path = if scope.config_path.is_empty() {
        Path::new(&scope.manifest_path)
            .parent()
            .unwrap()
            .join("elicit.toml")
    } else {
        PathBuf::from(&scope.config_path)
    };
    if config_path.is_file() {
        files.insert(config_path.clone());
    }

    let mut input_files = std::collections::BTreeMap::new();
    let mut h = Sha256::new();
    let mut proofs = Sha256::new();
    let proof_root = Path::new(&scope.manifest_path)
        .parent()
        .unwrap()
        .join("elicit-proofs");
    for f in files {
        if f == config_path {
            // Formal declarations and scope are inputs; editorial descriptions
            // describe intent and must not invalidate machine proofs.
            let mut config: toml::Value = toml::from_str(&fs::read_to_string(&f)?)?;
            if let Some(goals) = config.get_mut("obligations").and_then(|v| v.as_array_mut()) {
                for goal in goals {
                    if let Some(table) = goal.as_table_mut() {
                        table.remove("description");
                    }
                }
            }
            let bytes = serde_json::to_vec(&config)?;
            input_files.insert("@declarations".into(), hash(&bytes));
            proofs.update(bytes);
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
    let environment = environment_inputs(std::env::vars());
    for (key, value) in &environment {
        input_files.insert(format!("@env:{key}"), hash(value));
    }
    h.update(serde_json::to_vec(&environment)?);
    let context = format!("{:x}", h.finalize());
    proofs.update(context.as_bytes());
    Ok((format!("{:x}", proofs.finalize()), context, input_files))
}
// Native commands inherit the environment unchanged. Retain hashes, never values,
// so arbitrary script environment dependencies participate in freshness checks.
fn environment_inputs(
    vars: impl IntoIterator<Item = (String, String)>,
) -> std::collections::BTreeMap<String, String> {
    vars.into_iter()
        .filter(|(k, _)| k != "_" && k != "SHLVL")
        .collect()
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
fn tcb_inputs(scope: &Scope, tool: Tool, h: &mut Sha256) -> Result<()> {
    let root = Path::new(&scope.config_path)
        .parent()
        .filter(|_| !scope.config_path.is_empty())
        .unwrap_or_else(|| Path::new(&scope.manifest_path).parent().unwrap());
    let config: crate::project::Config =
        toml::from_str(&fs::read_to_string(root.join("elicit.toml"))?)?;
    let mut digests = InstalledDigests::load(root.join(".elicit/cache/installed-digests.json"));
    let mut commands: Vec<_> = config
        .obligations
        .iter()
        .map(|g| {
            (
                if g.command.is_empty() {
                    &config.verification.command
                } else {
                    &g.command
                },
                g.directory
                    .as_deref()
                    .or(config.verification.directory.as_deref())
                    .unwrap_or("."),
            )
        })
        .collect();
    commands.push((
        &config.verification.command,
        config.verification.directory.as_deref().unwrap_or("."),
    ));
    for (argv, directory) in commands {
        let Some(program) = argv.first() else {
            continue;
        };
        let path =
            if Path::new(program).components().count() > 1 || Path::new(program).is_absolute() {
                Some(root.join(directory).join(program))
            } else {
                executable(program)
            };
        h.update(program);
        if let Some(path) = path.filter(|p| p.is_file()) {
            h.update(path.to_string_lossy().as_bytes());
            h.update(digests.read(&path, tool)?);
        } else {
            h.update(b"missing executable");
        }
    }
    for input in &config.verification.inputs {
        let path = root.join(input);
        if !path.exists() {
            bail!("configured input does not exist: {}", path.display());
        }
        for e in WalkDir::new(&path).follow_links(false).sort_by_file_name() {
            let e = e?;
            if e.file_type().is_symlink() {
                bail!("symlink in configured inputs: {}", e.path().display());
            }
            if e.file_type().is_file() {
                h.update(e.path().to_string_lossy().as_bytes());
                h.update(digests.read(e.path(), tool)?);
            }
        }
    }
    digests.save();
    Ok(())
}

// Installed Lean environments can contain gigabytes of immutable proof objects.
// Cache only their content digests, guarded by Unix inode, size, mtime AND ctime.
// Source inputs are always read directly. This is an acceleration cache, never
// command evidence; deleting it reproduces exactly the same fingerprint.
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
        // Cache failures must not alter input identity or prevent execution.
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
mod environment_regressions {
    use super::*;
    #[test]
    fn environment_order_is_stable_and_native_selectors_are_tracked() {
        let a = vec![
            ("LEAN_PATH".into(), "one".into()),
            ("CUSTOM_PROOF_INPUT".into(), "two".into()),
        ];
        let mut b = a.clone();
        b.reverse();
        assert_eq!(environment_inputs(a.clone()), environment_inputs(b));
        assert_ne!(
            environment_inputs(a.clone()),
            environment_inputs(a.into_iter().take(1))
        );
    }
}
