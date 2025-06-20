// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use crate::Severity;
use serde::{Deserialize, Serialize};

/// Specifies how to match structs for linting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructMatch {
    /// Match structs by name (exact name or regex pattern)
    Name(String),
    /// Match structs that have a specific attribute (e.g., #[derive(Debug)])
    HasAttribute(String),
    /// Match structs that implement a specific trait
    ImplementsTrait(String),
    /// Logical AND - both patterns must match
    AndMatches(Box<StructMatch>, Box<StructMatch>),
    /// Logical OR - either pattern must match
    OrMatches(Box<StructMatch>, Box<StructMatch>),
    /// Logical NOT - inverts the match
    NotMatch(Box<StructMatch>),
}

/// A complete struct lint definition with matching criteria and rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructLint {
    pub name: String,
    pub matches: StructMatch,
    pub rules: Vec<StructRule>,
}

/// Rules that can be applied to structs matching specific criteria
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructRule {
    /// Enforces that the struct name matches the specified pattern
    MustBeNamed(String, Severity),
    /// Enforces that the struct name does not match the specified pattern
    MustNotBeNamed(String, Severity),
    /// Enforces that the struct has private visibility
    MustBePrivate(Severity),
    /// Enforces that the struct has public visibility
    MustBePublic(Severity),
    /// Enforces that the struct implements a specific trait
    ImplementsTrait(String, Severity),
    /// Logical AND - both rules must pass
    And(Box<StructRule>, Box<StructRule>),
    /// Logical OR - either rule must pass
    Or(Box<StructRule>, Box<StructRule>),
    /// Logical NOT - inverts the rule check
    Not(Box<StructRule>),
}
