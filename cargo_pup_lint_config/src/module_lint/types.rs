// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use serde::{Deserialize, Serialize};
use crate::Severity;

/// Specifies how to match modules for linting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleMatch {
    /// Match modules by path (exact path or regex pattern)
    Module(String),
    /// Logical AND - both patterns must match
    AndMatches(Box<ModuleMatch>, Box<ModuleMatch>),
    /// Logical OR - either pattern must match
    OrMatches(Box<ModuleMatch>, Box<ModuleMatch>),
    /// Logical NOT - inverts the match
    NotMatch(Box<ModuleMatch>),
}

/// A complete module lint definition with matching criteria and rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleLint {
    pub name: String,
    pub matches: ModuleMatch,
    pub rules: Vec<ModuleRule>,
}

/// Rules that can be applied to modules matching specific criteria
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleRule {
    /// Enforces that the module name matches the specified pattern
    MustBeNamed(String, Severity),
    /// Enforces that the module name does not match the specified pattern
    MustNotBeNamed(String, Severity),
    /// Enforces that the module contains at least one item
    MustNotBeEmpty(Severity),
    /// Enforces that the module contains no items
    MustBeEmpty(Severity),
    /// Enforces that the module.rs file only re-exports other modules
    MustHaveEmptyModFile(Severity),
    /// Controls which modules can be imported
    RestrictImports {
        allowed_only: Option<Vec<String>>,
        denied: Option<Vec<String>>,
        severity: Severity,
    },
    /// Prevents use of wildcard imports (use path::*)
    NoWildcardImports(Severity),
    /// Prevents specific items from being defined in the module
    DeniedItems {
        items: Vec<String>,
        severity: Severity,
    },
    /// Logical AND - both rules must pass
    And(Box<ModuleRule>, Box<ModuleRule>),
    /// Logical OR - either rule must pass
    Or(Box<ModuleRule>, Box<ModuleRule>),
    /// Logical NOT - inverts the rule check
    Not(Box<ModuleRule>),
}

