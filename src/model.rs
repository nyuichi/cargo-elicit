use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Tool {
    Creusot,
    Kani,
    Aeneas,
}
impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Creusot => "creusot",
            Self::Kani => "kani",
            Self::Aeneas => "aeneas",
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub agent: AgentConfig,
    pub verification: VerificationConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    pub provider: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationConfig {
    pub tool: Tool,
    pub required: bool,
}
impl Config {
    pub fn new(tool: Tool) -> Self {
        Self {
            agent: AgentConfig {
                provider: "codex".into(),
            },
            verification: VerificationConfig {
                tool,
                required: false,
            },
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.agent.provider != "codex" {
            bail!("only agent.provider = codex is supported");
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scope {
    pub package: String,
    pub manifest_path: String,
    pub workspace_root: String,
    pub target_directory: String,
    pub features: Vec<String>,
    pub resolved_features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub target: String,
    pub requested_target: Option<String>,
    pub local_roots: Vec<String>,
}
impl Scope {
    pub fn cargo_args(&self) -> Vec<String> {
        let mut a = vec![
            "--manifest-path".into(),
            self.manifest_path.clone(),
            "--package".into(),
            self.package.clone(),
            "--target".into(),
            self.target.clone(),
        ];
        if self.all_features {
            a.push("--all-features".into());
        }
        if self.no_default_features {
            a.push("--no-default-features".into());
        }
        if !self.features.is_empty() {
            a.extend(["--features".into(), self.features.join(",")]);
        }
        a
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Struct,
    Enum,
    Trait,
    Function,
    Method,
    TraitImpl,
}
impl Kind {
    pub fn proof(self) -> bool {
        matches!(self, Self::Function | Self::Method | Self::TraitImpl)
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Internal,
    Auxiliary,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blocker {
    pub reason: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    pub visibility: Visibility,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub aliases: Vec<String>,
    pub spec_hash: String,
    #[serde(default)]
    pub model_base_hash: String,
    #[serde(default)]
    pub model_inputs: BTreeMap<String, String>,
    #[serde(default)]
    pub model_bindings: Option<BTreeSet<String>>,
    pub body_hash: String,
    pub original_body: String,
    pub source_hash: String,
    pub specified: bool,
    #[serde(default)]
    pub blocker: Option<Blocker>,
    pub stale: Vec<String>,
    pub dependencies: BTreeMap<String, String>,
    pub calls: BTreeSet<String>,
    pub unresolved_calls: Vec<String>,
    pub required_units: Vec<String>,
    pub notes: Vec<String>,
    pub removed: bool,
    pub implementation_changed: bool,
    pub closed_primitive: bool,
    pub trusted: bool,
}
impl Target {
    pub fn model_hash(&self) -> String {
        // If a bound declaration disappeared, or model collection fell back to
        // a whole-source stamp, retain that stamp rather than hashing only
        // missing keys. Further semantic edits must still reopen blocked work.
        let inputs = match &self.model_bindings {
            Some(names)
                if names
                    .iter()
                    .all(|name| self.model_inputs.contains_key(name)) =>
            {
                names
                    .iter()
                    .map(|name| (name, self.model_inputs.get(name)))
                    .collect::<BTreeMap<_, _>>()
            }
            _ => self
                .model_inputs
                .iter()
                .map(|(name, value)| (name, Some(value)))
                .collect(),
        };
        crate::fingerprint::hash(serde_json::to_vec(&(&self.model_base_hash, inputs)).unwrap())
    }
    pub fn bind_models(&mut self) {
        if !self.model_base_hash.is_empty() && !self.kind.proof() {
            self.model_bindings = Some(self.model_inputs.keys().cloned().collect());
            self.spec_hash = self.model_hash();
        }
    }
    pub fn ready(&self) -> bool {
        self.specified
            && self.blocker.is_none()
            && self.stale.is_empty()
            && !self.removed
            && !self.implementation_changed
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    Unresolved,
    Timeout,
    Interrupted,
    EnvironmentError,
    Skipped,
    UnknownScope,
    InputChanged,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Coverage {
    FullContract,
    Limited,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub phase: String,
    pub cache: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub id: String,
    pub target: String,
    pub units: Vec<String>,
    pub obligations: Vec<String>,
    pub scope: Coverage,
    pub assumptions: Vec<String>,
    pub restrictions: Vec<String>,
    pub input: String,
    pub tool: Tool,
    pub tool_version: String,
    pub trusted_base: Vec<String>,
    pub reusable: bool,
    pub outcome: Outcome,
    pub summary: String,
    #[serde(default)]
    pub diagnostic: Option<Diagnostic>,
    pub elapsed_ms: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    Recheck,
    Specify,
    Verify,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub kind: TaskKind,
    pub targets: Vec<String>,
}
impl Task {
    pub fn key(&self) -> String {
        format!("{:?}:{}", self.kind, self.targets.join("|"))
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Attempt {
    pub total: u32,
    pub no_progress: u32,
    pub last_input: String,
    pub failed: bool,
    pub last_result: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Active {
    pub task: Task,
    pub input: String,
    pub started: u64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunState {
    pub thread: Option<String>,
    pub active: Option<Active>,
    pub attempts: BTreeMap<String, Attempt>,
    pub last_result: String,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub elapsed_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub schema_version: u32,
    pub revision: u64,
    pub tool: Tool,
    pub scope: Scope,
    pub input: String,
    pub processing_input: String,
    #[serde(default)]
    pub context_input: String,
    #[serde(default)]
    pub input_files: BTreeMap<String, String>,
    pub targets: BTreeMap<String, Target>,
    pub results: Vec<Verification>,
    pub analysis_incomplete: Vec<String>,
    pub run: RunState,
}
impl State {
    pub fn new(tool: Tool, scope: Scope) -> Self {
        Self {
            schema_version: 1,
            revision: 0,
            tool,
            scope,
            input: String::new(),
            processing_input: String::new(),
            context_input: String::new(),
            input_files: BTreeMap::new(),
            targets: BTreeMap::new(),
            results: vec![],
            analysis_incomplete: vec![],
            run: RunState::default(),
        }
    }
    pub fn unsupported(&self, id: &str) -> Option<&'static str> {
        (self.tool == Tool::Aeneas && self.targets.get(id).is_some_and(|t| t.kind == Kind::TraitImpl))
            .then_some("Aeneas trait implementation mapping is unsupported; explicit methods remain separate obligations")
    }
    pub fn target_input(&self, id: &str) -> String {
        let t = &self.targets[id];
        let context = if self.context_input.is_empty() {
            &self.input
        } else {
            &self.context_input
        };
        let dependencies: Vec<_> = t
            .dependencies
            .keys()
            .map(|id| {
                (
                    id,
                    self.targets.get(id).map(|d| (&d.spec_hash, &d.body_hash)),
                )
            })
            .collect();
        crate::fingerprint::hash(
            serde_json::to_vec(&(
                context,
                self.tool,
                &t.id,
                &t.spec_hash,
                &t.body_hash,
                dependencies,
            ))
            .unwrap(),
        )
    }
    pub fn schedulable(&self, id: &str) -> bool {
        self.unsupported(id).is_none() && self.targets[id].blocker.is_none()
    }
    pub fn dependencies_ready(&self, id: &str) -> bool {
        fn visit(s: &State, id: &str, path: &mut BTreeSet<String>) -> bool {
            if !path.insert(id.into()) {
                return false;
            }
            let ok = s.unsupported(id).is_none()
                && s.targets.get(id).is_some_and(|t| {
                    t.ready()
                        && t.dependencies.iter().all(|(d, h)| {
                            s.targets.get(d).is_some_and(|v| &v.spec_hash == h) && visit(s, d, path)
                        })
                });
            path.remove(id);
            ok
        }
        visit(self, id, &mut BTreeSet::new())
    }
    pub fn verified(&self, id: &str) -> bool {
        let Some(t) = self.targets.get(id) else {
            return false;
        };
        t.kind.proof()
            && self.dependencies_ready(id)
            && !t.required_units.is_empty()
            && t.required_units.iter().all(|u| {
                self.results.iter().any(|r| {
                    r.target == id
                        && r.input == self.input
                        && r.outcome == Outcome::Success
                        && r.scope == Coverage::FullContract
                        && r.reusable
                        && r.assumptions.is_empty()
                        && r.restrictions.is_empty()
                        && !r.trusted_base.is_empty()
                        && !r.obligations.is_empty()
                        && r.units.contains(u)
                })
            })
    }
    pub fn complete(&self, required: bool) -> bool {
        self.processing_input == self.input
            && self.analysis_incomplete.is_empty()
            && !self
                .targets
                .values()
                .any(|t| t.removed && t.visibility != Visibility::Auxiliary)
            && self
                .targets
                .values()
                .filter(|t| !t.removed && t.visibility != Visibility::Auxiliary)
                .all(|t| t.ready() && (!required || !t.kind.proof() || self.verified(&t.id)))
    }
    pub fn reconcile(&mut self, fresh: Vec<Target>, incomplete: Vec<String>, input: String) {
        let seen: BTreeSet<_> = fresh.iter().map(|t| t.id.clone()).collect();
        for mut new in fresh {
            if let Some(old) = self.targets.get(&new.id) {
                if old.specified && !new.kind.proof() && !new.model_base_hash.is_empty() {
                    new.model_bindings = old.model_bindings.clone();
                    new.spec_hash = new.model_hash();
                }
                new.specified = old.specified;
                new.blocker = old.blocker.clone();
                new.dependencies = old.dependencies.clone();
                new.notes = old.notes.clone();
                new.stale = old.stale.clone();
                new.required_units = old.required_units.clone();
                new.implementation_changed = old.implementation_changed;
                if old.visibility != Visibility::Auxiliary
                    && new.visibility == Visibility::Auxiliary
                {
                    new.visibility = old.visibility;
                    new.implementation_changed = true;
                    new.stale
                        .push("existing API reclassified as proof helper; review required".into());
                }
                if old.spec_hash != new.spec_hash {
                    new.stale.push("specification changed".into());
                }
                if old.body_hash != new.body_hash {
                    new.stale.push("implementation changed".into());
                }
                if old.removed {
                    new.stale
                        .push("definition reappeared; review required".into());
                }
                new.original_body = old.original_body.clone();
                if new.visibility != Visibility::Auxiliary && new.body_hash != old.original_body {
                    if self.run.active.is_some() || old.implementation_changed {
                        new.implementation_changed = true;
                    } else {
                        new.original_body = new.body_hash.clone();
                    }
                }
            }
            new.stale.sort();
            new.stale.dedup();
            self.targets.insert(new.id.clone(), new);
        }
        for (id, t) in &mut self.targets {
            if !seen.contains(id) {
                t.removed = true;
                if !t
                    .stale
                    .iter()
                    .any(|x| x == "definition removed or identity changed")
                {
                    t.stale
                        .push("definition removed or identity changed".into());
                }
            }
        }
        let hashes: BTreeMap<_, _> = self
            .targets
            .iter()
            .map(|(id, t)| (id.clone(), (t.spec_hash.clone(), t.removed)))
            .collect();
        for t in self.targets.values_mut() {
            for (d, h) in &t.dependencies {
                if !hashes.get(d).is_some_and(|(v, removed)| v == h && !removed) {
                    t.stale
                        .push(format!("specification dependency changed: {d}"));
                }
            }
            t.stale.sort();
            t.stale.dedup();
        }
        self.input = input;
        let unblocked: Vec<_> = self
            .targets
            .iter()
            .filter_map(|(id, t)| {
                t.blocker
                    .as_ref()
                    .filter(|b| b.input != self.target_input(id))
                    .map(|_| id.clone())
            })
            .collect();
        for id in unblocked {
            self.targets.get_mut(&id).unwrap().blocker = None;
        }
        self.analysis_incomplete = incomplete;
    }
}
