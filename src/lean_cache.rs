//! Reuse only products created by this checker for identical tracked inputs.
use crate::{fingerprint, lean_modules, model::State, process, store};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
#[derive(Clone, Default, Serialize, Deserialize)]
struct Entry {
    key: String,
    objects: BTreeMap<String, String>,
}
#[derive(Default, Serialize, Deserialize)]
struct Manifest {
    modules: BTreeMap<String, Entry>,
}
pub struct Compiler<'a> {
    state: &'a State,
    root: &'a Path,
    enabled: bool,
    conservative: bool,
    manifest: Manifest,
}
impl<'a> Compiler<'a> {
    pub fn new(state: &'a State, root: &'a Path, enabled: bool) -> Result<Self> {
        let manifest = fs::read(root.join("module-cache.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let sources = crate::aeneas::proof_root(&state.scope);
        let project = lean_modules::Modules::load(&sources)?;
        // Old products must not satisfy imports whose local sources disappeared.
        // External libraries are loaded only from the pinned backend, never here.
        if root.exists() {
            for entry in walkdir::WalkDir::new(root).into_iter().filter_entry(|e| {
                e.path() != root.join("rust-build") && e.path() != root.join("translation")
            }) {
                let entry = entry?;
                if entry.file_type().is_symlink() {
                    anyhow::bail!("symlink in compiler cache");
                }
                if !entry.file_type().is_file() {
                    continue;
                }
                let relative = entry.path().strip_prefix(root)?.to_string_lossy();
                if let Some(index) = relative.find(".olean") {
                    let module = relative[..index].replace('/', ".");
                    if !project.contains(&module)
                        && ![
                            "ElicitGenerated",
                            "ElicitAdapter",
                            "ElicitContracts",
                            "Audit",
                        ]
                        .contains(&module.as_str())
                    {
                        fs::remove_file(entry.path())?;
                    }
                }
            }
        }
        let conservative = walkdir::WalkDir::new(sources)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|v| v == "lean"))
            .any(|e| {
                fs::read_to_string(e.path()).map_or(true, |s| {
                    [
                        "run_elab",
                        "#eval",
                        "readFile",
                        "IO.",
                        "unsafe ",
                        "initialize ",
                        "macro ",
                        "elab ",
                        "syntax ",
                    ]
                    .iter()
                    .any(|v| s.contains(v))
                })
            });
        Ok(Self {
            state,
            root,
            enabled,
            conservative,
            manifest,
        })
    }
    fn objects(&self, module: &str) -> Result<BTreeMap<String, String>> {
        let path = lean_modules::module_path(self.root, module, "olean");
        if !path.is_file() {
            return Ok(BTreeMap::new());
        }
        let stem = path
            .file_name()
            .context("missing module name")?
            .to_string_lossy();
        let mut objects = BTreeMap::new();
        for entry in fs::read_dir(path.parent().unwrap())? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == stem || name.starts_with(&format!("{stem}.")) {
                if entry.file_type()?.is_symlink() {
                    anyhow::bail!("symlink compiler product");
                }
                objects.insert(
                    entry
                        .path()
                        .strip_prefix(self.root)?
                        .to_string_lossy()
                        .into_owned(),
                    fingerprint::hash(fs::read(entry.path())?),
                );
            }
        }
        Ok(objects)
    }
    pub fn run(&mut self, command: &mut Command, seconds: u64) -> Result<process::Output> {
        let source = command
            .get_args()
            .last()
            .map(PathBuf::from)
            .context("missing compiler source")?;
        let module = source
            .strip_prefix(self.root)?
            .with_extension("")
            .components()
            .map(|p| p.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join(".");
        let text = fs::read_to_string(&source)?;
        let imports = lean_modules::imports(&text)?;
        let dependencies = imports
            .iter()
            .map(|name| {
                (
                    name,
                    self.manifest
                        .modules
                        .get(name)
                        .map(|e| (&e.key, &e.objects)),
                )
            })
            .collect::<Vec<_>>();
        let context = if self.conservative {
            &self.state.input
        } else {
            &self.state.context_input
        };
        let key = fingerprint::hash(serde_json::to_vec(&(
            context,
            &module,
            &text,
            dependencies,
        ))?);
        let reusable = self.enabled && module != "Audit" && module != "ElicitModelInputs";
        if reusable
            && self.manifest.modules.get(&module).is_some_and(|entry| {
                entry.key == key
                    && !entry.objects.is_empty()
                    && self
                        .objects(&module)
                        .is_ok_and(|objects| objects == entry.objects)
            })
        {
            return Ok(process::Output {
                code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                timeout: false,
                interrupted: false,
                elapsed_ms: 0,
            });
        }
        self.manifest.modules.remove(&module);
        let output = process::run(command, Duration::from_secs(seconds))?;
        fs::create_dir_all(self.root.join("logs"))?;
        fs::write(
            self.root.join("logs").join(format!("{module}.log")),
            format!("{}\n{}", output.stdout, output.stderr),
        )?;
        if output.code == Some(0) && !output.timeout && !output.interrupted {
            let objects = self.objects(&module)?;
            self.manifest.modules.insert(module, Entry { key, objects });
        }
        store::atomic_write(
            &self.root.join("module-cache.json"),
            &serde_json::to_vec_pretty(&self.manifest)?,
        )?;
        Ok(output)
    }
}
