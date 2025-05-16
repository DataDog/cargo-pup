use serde::{Deserialize, Serialize};
use crate::Severity;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructMatch {
    Name(String),
    HasAttribute(String),
    ImplementsTrait(String),
    // Logical operations
    AndMatches(Box<StructMatch>, Box<StructMatch>),
    OrMatches(Box<StructMatch>, Box<StructMatch>),
    NotMatch(Box<StructMatch>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructLint {
    pub name: String,
    pub matches: StructMatch,
    pub rules: Vec<StructRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructRule {
    MustBeNamed(String, Severity),
    MustNotBeNamed(String, Severity),
    MustBePrivate(Severity),
    MustBePublic(Severity),
    ImplementsTrait(String, Severity),
    And(Box<StructRule>, Box<StructRule>),
    Or(Box<StructRule>, Box<StructRule>),
    Not(Box<StructRule>),
}

