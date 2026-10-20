//! Advisory Lean import graph. Never controls command execution or acceptance.
use anyhow::{Context, Result, bail};
use std::{collections::BTreeMap, fs, path::Path};
use walkdir::WalkDir;

#[derive(Default)]
pub struct Modules {
    sources: BTreeMap<String, String>,
    imports: BTreeMap<String, Vec<String>>,
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
        if line.is_empty() || line == "prelude" || line == "module" {
            continue;
        }
        let mut words = line.split_whitespace().peekable();
        if words.peek() == Some(&"public") {
            words.next();
        }
        if words.next() != Some("import") {
            let first = line.split_whitespace().next().unwrap();
            if ["meta", "private", "public"].contains(&first)
                || (module_name(first)
                    && first.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    && line.split_whitespace().count() == 1)
            {
                bail!(
                    "unsupported Lean import/header continuation; use single-line import or public import"
                );
            }
            break;
        }
        if words.peek() == Some(&"all") {
            words.next();
        }
        if words.peek().is_none() {
            bail!("put imported module names on the import line");
        }
        for name in words {
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
        for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
            !matches!(
                e.file_name().to_str(),
                Some(".git" | ".elicit" | ".lake" | "target")
            )
        }) {
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
    pub fn planning_view(&self) -> serde_json::Value {
        serde_json::json!({"kind":"Lean module imports; not theorem-use dependencies",
            "modules":self.imports.iter().map(|(name,imports)|serde_json::json!({"name":name,"imports":imports,
                "imported_by":self.imports.iter().filter(|(_,ds)|ds.contains(name)).map(|(n,_)|n).collect::<Vec<_>>()
            })).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imports_handle_whitespace_and_reject_unsupported_headers() {
        assert_eq!(
            imports("public\timport\tall\tSupport.Facts\nimport\tPeer\n").unwrap(),
            vec!["Support.Facts", "Peer"]
        );
        for source in [
            "import\nSupport.Facts",
            "meta\timport Support.Facts",
            "public\nimport Support.Facts",
            "import Aeneas\n  Support.Facts",
        ] {
            assert!(imports(source).is_err(), "{source}");
        }
    }
}
