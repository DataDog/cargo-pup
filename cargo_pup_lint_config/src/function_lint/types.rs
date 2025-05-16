use serde::{Deserialize, Serialize};
use crate::Severity;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReturnTypePattern {
    Result,           // Match any Result<T, E>
    Option,           // Match any Option<T>
    Named(String),    // Match a specific named type
    Regex(String),    // Match types by regex pattern
    ResultWithErrorImpl, // Match Result<T, E> where E implements Error trait
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FunctionMatch {
    NameEquals(String),
    NameRegex(String),
    InModule(String),
    // New variant to match by return type
    ReturnsType(ReturnTypePattern),
    // Logical operations
    AndMatches(Box<FunctionMatch>, Box<FunctionMatch>),
    OrMatches(Box<FunctionMatch>, Box<FunctionMatch>),
    NotMatch(Box<FunctionMatch>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionLint {
    pub name: String,
    pub matches: FunctionMatch,
    pub rules: Vec<FunctionRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FunctionRule {
    MaxLength(usize, Severity),
    ResultErrorMustImplementError(Severity),
}

// Helper methods for FunctionRule
impl FunctionRule {
    // pub fn and(self, other: FunctionRule) -> Self {
    //     FunctionRule::And(Box::new(self), Box::new(other))
    // }
    
    // pub fn or(self, other: FunctionRule) -> Self {
    //     FunctionRule::Or(Box::new(self), Box::new(other))
    // }
    
    // pub fn not(self) -> Self {
    //     FunctionRule::Not(Box::new(self))
    // }
}
