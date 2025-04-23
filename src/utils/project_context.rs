use serde::{Deserialize, Serialize};

/// Context for configuration generation containing compile-time discoverable
/// context about the project we're running cargo-pup on. 
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectContext {
    /// List of all modules, fully qualified
    pub modules: Vec<String>,
    /// The top-level crate name (root module)
    pub module_root: String,
    /// List of all traits, fully qualified, and their implementations
    pub traits: Vec<TraitInfo>,
}

/// Information about a trait and its implementations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraitInfo {
    /// Fully qualified trait name
    pub name: String,
    /// List of types implementing this trait
    pub implementors: Vec<String>,
} 