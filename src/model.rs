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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scope {
    #[serde(default)]
    pub config_path: String,
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
    pub body_hash: String,
    pub source_hash: String,
    pub calls: BTreeSet<String>,
    pub unresolved_calls: Vec<String>,
    #[serde(default)]
    pub external_calls: BTreeSet<String>,
    #[serde(default)]
    pub source_facts: Option<serde_json::Value>,
    #[serde(default)]
    pub native_contract: Option<serde_json::Value>,
    pub required_units: Vec<String>,
    pub closed_primitive: bool,
    pub trusted: bool,
}
impl Target {
    pub fn model_hash(&self) -> String {
        crate::fingerprint::hash(
            serde_json::to_vec(&(&self.model_base_hash, &self.model_inputs)).unwrap(),
        )
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

/// Private verifier input, not the user-facing project ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub revision: u64,
    pub tool: Tool,
    pub scope: Scope,
    pub input: String,
    pub context_input: String,
    pub targets: BTreeMap<String, Target>,
    pub results: Vec<Verification>,
    pub claims: Vec<Claim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub statement: String,
    pub theorem: String,
}

#[cfg(test)]
impl State {
    pub(crate) fn new(tool: Tool, scope: Scope) -> Self {
        Self {
            revision: 0,
            tool,
            scope,
            input: String::new(),
            context_input: String::new(),
            targets: BTreeMap::new(),
            results: vec![],
            claims: vec![],
        }
    }
}
