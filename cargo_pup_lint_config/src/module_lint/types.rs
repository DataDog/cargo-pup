use serde::{Deserialize, Serialize};
use crate::Severity;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleMatch {
    Module(String),
    // Logical operations
    AndMatches(Box<ModuleMatch>, Box<ModuleMatch>),
    OrMatches(Box<ModuleMatch>, Box<ModuleMatch>),
    NotMatch(Box<ModuleMatch>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleLint {
    pub name: String,
    pub matches: ModuleMatch,
    pub rules: Vec<ModuleRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleRule {
    MustBeNamed(String, Severity),
    MustNotBeNamed(String, Severity),
    MustNotBeEmpty(Severity),
    MustBeEmpty(Severity),
    MustHaveEmptyModFile(Severity),
    RestrictImports {
        allowed_only: Option<Vec<String>>,
        denied: Option<Vec<String>>,
        severity: Severity,
    },
    NoWildcardImports(Severity),
    DeniedItems {
        items: Vec<String>,
        severity: Severity,
    },
    And(Box<ModuleRule>, Box<ModuleRule>),
    Or(Box<ModuleRule>, Box<ModuleRule>),
    Not(Box<ModuleRule>),
}

