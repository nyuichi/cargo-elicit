//! Tracked local Lean source graph. Compiled objects are never imported from it.
use anyhow::{Context, Result, bail};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const GENERATED: &[&str] = &[
    "ElicitGenerated",
    "ElicitAdapter",
    "ElicitContracts",
    "ElicitAudit",
    "ElicitModelInputs",
    "Audit",
];
#[derive(Default)]
pub struct Modules {
    sources: BTreeMap<String, String>,
    imports: BTreeMap<String, Vec<String>>,
}
pub fn module_path(root: &Path, name: &str, extension: &str) -> PathBuf {
    root.join(name.replace('.', "/")).with_extension(extension)
}
fn module_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('.').all(|part| {
            !part.is_empty()
                && part.chars().enumerate().all(|(i, c)| {
                    c.is_ascii_alphabetic() || c == '_' || i > 0 && c.is_ascii_digit()
                })
        })
}
fn external(name: &str) -> bool {
    ["Aeneas", "Mathlib", "Std", "Lean", "Init"]
        .iter()
        .any(|p| name == *p || name.starts_with(&format!("{p}.")))
}
// Parse the ordinary import header, failing closed on unsupported header forms.
// Nested comments and quoted strings are preserved/removed without treating
// an import-looking line inside a comment as an actual dependency.
fn imports(source: &str) -> Result<Vec<String>> {
    let bytes = source.as_bytes();
    let mut clean = Vec::new();
    let mut i = 0;
    let mut depth = 0;
    let mut quoted = false;
    while i < bytes.len() {
        if depth > 0 {
            if bytes[i..].starts_with(b"/-") {
                depth += 1;
                i += 2;
            } else if bytes[i..].starts_with(b"-/") {
                depth -= 1;
                i += 2;
            } else {
                if bytes[i] == b'\n' {
                    clean.push(b'\n');
                }
                i += 1;
            }
        } else if quoted {
            clean.push(bytes[i]);
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 1;
                clean.push(bytes[i]);
            } else if bytes[i] == b'"' {
                quoted = false;
            }
            i += 1;
        } else if bytes[i..].starts_with(b"--") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i..].starts_with(b"/-") {
            clean.push(b' ');
            depth = 1;
            i += 2;
        } else {
            if bytes[i] == b'"' {
                quoted = true;
            }
            clean.push(bytes[i]);
            i += 1;
        }
    }
    if depth != 0 {
        bail!("unterminated Lean comment");
    }
    let clean = String::from_utf8(clean)?;
    let mut result = vec![];
    for line in clean.lines() {
        let line = line.trim();
        if line.is_empty() || line == "prelude" {
            continue;
        }
        let line = line.strip_prefix("public ").unwrap_or(line);
        let Some(line) = line.strip_prefix("import ") else {
            if line == "import" {
                bail!("put imported module names on the import line");
            }
            break;
        };
        let line = line.strip_prefix("all ").unwrap_or(line);
        for name in line.split_whitespace() {
            if !module_name(name) {
                bail!("unsupported Lean import {name:?}");
            }
            result.push(name.to_owned());
        }
    }
    Ok(result)
}
impl Modules {
    pub fn load(root: &Path) -> Result<Self> {
        let mut project = Self::default();
        if !root.exists() {
            return Ok(project);
        }
        for entry in WalkDir::new(root) {
            let entry = entry?;
            if entry.file_type().is_symlink() {
                bail!("symlink in Lean source tree: {}", entry.path().display());
            }
            if !entry.file_type().is_file() || entry.path().extension().is_none_or(|s| s != "lean")
            {
                continue;
            }
            let relative = entry.path().strip_prefix(root)?.with_extension("");
            let name = relative
                .components()
                .map(|p| p.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join(".");
            if !module_name(&name) || GENERATED.contains(&name.as_str()) || external(&name) {
                bail!("reserved or unsupported local Lean module {name:?}");
            }
            let source = fs::read_to_string(entry.path())?;
            project.imports.insert(
                name.clone(),
                imports(&source).with_context(|| format!("imports of {name}"))?,
            );
            if project.sources.insert(name, source).is_some() {
                bail!("duplicate local Lean module");
            }
        }
        Ok(project)
    }
    pub fn contains(&self, name: &str) -> bool {
        self.sources.contains_key(name)
    }
    pub fn len(&self) -> usize {
        self.sources.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
    pub fn standard_layout(&self) -> bool {
        self.sources
            .keys()
            .all(|s| ["Specs", "Lemmas", "Proofs"].contains(&s.as_str()))
    }
    pub fn model_digest(&self) -> String {
        if !self.contains("Specs") {
            return crate::fingerprint::hash([]);
        }
        let built = BTreeSet::from(["ElicitGenerated".into(), "ElicitAdapter".into()]);
        match self.order(&["Specs"], &built) {
            Ok(names) => crate::fingerprint::hash(
                serde_json::to_vec(
                    &names
                        .iter()
                        .map(|n| (n, &self.sources[n]))
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            ),
            Err(_) => self.source_digest(),
        }
    }
    pub fn source_digest(&self) -> String {
        crate::fingerprint::hash(serde_json::to_vec(&self.sources).unwrap())
    }
    pub fn copy(&self, name: &str, out: &Path) -> Result<()> {
        let path = module_path(out, name, "lean");
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(
            path,
            self.sources.get(name).context("unknown local module")?,
        )?;
        Ok(())
    }
    pub fn order(&self, roots: &[&str], built: &BTreeSet<String>) -> Result<Vec<String>> {
        fn visit(
            p: &Modules,
            name: &str,
            built: &BTreeSet<String>,
            active: &mut BTreeSet<String>,
            done: &mut BTreeSet<String>,
            out: &mut Vec<String>,
        ) -> Result<()> {
            if built.contains(name) || done.contains(name) || external(name) {
                return Ok(());
            }
            let deps = p.imports.get(name).with_context(|| {
                format!(
                    "unavailable Lean module {name}; local imports must have tracked .lean sources"
                )
            })?;
            if !active.insert(name.to_owned()) {
                bail!("cyclic Lean import involving {name}");
            }
            for dep in deps {
                visit(p, dep, built, active, done, out)?;
            }
            active.remove(name);
            done.insert(name.to_owned());
            out.push(name.to_owned());
            Ok(())
        }
        let mut out = vec![];
        let mut active = BTreeSet::new();
        let mut done = BTreeSet::new();
        for root in roots {
            visit(self, root, built, &mut active, &mut done, &mut out)?;
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graph_orders_nested_imports_and_rejects_cycles_missing_sources_and_reserved_names() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir(d.path().join("Support")).unwrap();
        fs::write(
            d.path().join("Support/Facts.lean"),
            "/- import Missing /- nested -/ -/\nimport Aeneas\ntheorem fact : True := True.intro",
        )
        .unwrap();
        fs::write(
            d.path().join("Proofs.lean"),
            "import Support.Facts ElicitContracts\n",
        )
        .unwrap();
        let p = Modules::load(d.path()).unwrap();
        let built = BTreeSet::from(["ElicitContracts".into()]);
        assert_eq!(
            p.order(&["Proofs"], &built).unwrap(),
            vec!["Support.Facts", "Proofs"]
        );
        fs::write(d.path().join("Support/Facts.lean"), "import Proofs\n").unwrap();
        assert!(
            Modules::load(d.path())
                .unwrap()
                .order(&["Proofs"], &built)
                .is_err()
        );
        fs::write(d.path().join("Proofs.lean"), "import Missing\n").unwrap();
        assert!(
            Modules::load(d.path())
                .unwrap()
                .order(&["Proofs"], &built)
                .is_err()
        );
        fs::write(d.path().join("ElicitGenerated.lean"), "").unwrap();
        assert!(Modules::load(d.path()).is_err());
    }
    #[test]
    fn models_track_imported_sources_but_not_independent_proof_edits() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir(d.path().join("Models")).unwrap();
        fs::write(d.path().join("Models/Value.lean"), "def value := 1\n").unwrap();
        fs::write(d.path().join("Specs.lean"), "import Models.Value\n").unwrap();
        fs::write(d.path().join("Proofs.lean"), "import ElicitContracts\n").unwrap();
        let digest = Modules::load(d.path()).unwrap().model_digest();
        fs::write(
            d.path().join("Proofs.lean"),
            "import ElicitContracts\n-- changed proof\n",
        )
        .unwrap();
        assert_eq!(digest, Modules::load(d.path()).unwrap().model_digest());
        fs::write(d.path().join("Models/Value.lean"), "def value := 2\n").unwrap();
        assert_ne!(digest, Modules::load(d.path()).unwrap().model_digest());
    }
}
