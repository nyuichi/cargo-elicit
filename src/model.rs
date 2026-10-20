use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
    pub body_hash: String,
    pub source_hash: String,
    pub calls: BTreeSet<String>,
    pub unresolved_calls: Vec<String>,
    #[serde(default)]
    pub external_calls: BTreeSet<String>,
    #[serde(default)]
    pub source_facts: Option<serde_json::Value>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub phase: String,
    pub cache: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Relative to cache, which is relocated into the durable receipt directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub target: String,
    pub command: Vec<String>,
    pub directory: String,
    pub exit_code: Option<i32>,
    pub outcome: Outcome,
    pub input: String,
    pub summary: String,
    #[serde(default)]
    pub diagnostic: Option<Diagnostic>,
    pub elapsed_ms: u64,
}
