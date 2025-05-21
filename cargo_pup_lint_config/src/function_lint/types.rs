use serde::{Deserialize, Serialize};
use crate::Severity;

/// Defines patterns for matching function return types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ReturnTypePattern {
    /// Match any Result<T, E>
    Result,
    /// Match any Option<T>
    Option,
    /// Match a specific named type
    Named(String),
    /// Match types by regex pattern
    Regex(String),
    /// Match Result<T, E> where E implements Error trait
    ResultWithErrorImpl,
}

/// Specifies how to match functions for linting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FunctionMatch {
    /// Match functions with exactly this name
    NameEquals(String),
    /// Match functions whose name matches this regex pattern
    NameRegex(String),
    /// Match functions inside a specific module
    InModule(String),
    /// Match functions that return a specific type pattern
    ReturnsType(ReturnTypePattern),
    /// Logical AND - both patterns must match
    AndMatches(Box<FunctionMatch>, Box<FunctionMatch>),
    /// Logical OR - either pattern must match
    OrMatches(Box<FunctionMatch>, Box<FunctionMatch>),
    /// Logical NOT - inverts the match
    NotMatch(Box<FunctionMatch>),
}

/// A complete function lint definition with matching criteria and rules
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionLint {
    pub name: String,
    pub matches: FunctionMatch,
    pub rules: Vec<FunctionRule>,
}

/// Rules that can be applied to functions matching specific criteria
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FunctionRule {
    /// Enforces maximum function length in lines of code
    MaxLength(usize, Severity),
    /// Enforces that Result error types must implement the Error trait
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
