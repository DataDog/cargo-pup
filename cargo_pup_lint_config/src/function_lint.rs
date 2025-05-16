use serde::{Deserialize, Serialize};
use crate::lint_builder::LintBuilder;
use super::{ConfiguredLint, Severity};
use regex::Regex;

// === Function Matcher DSL === //

pub struct FunctionMatcher;

impl FunctionMatcher {
    pub fn name(&self, name: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::NameEquals(name.into()))
    }
    
    pub fn name_regex(&self, pattern: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::NameRegex(pattern.into()))
    }
    
    /// Matches functions in a specific module
    /// 
    /// The module parameter can be either:
    /// - An exact module path (e.g., "core::utils")
    /// - A regular expression pattern (e.g., "^core::(utils|models)::[a-zA-Z]+$")
    ///
    /// The implementation will determine if it's a regex based on the presence of special regex characters.
    pub fn in_module(&self, module: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::InModule(module.into()))
    }
    
    /// Matches functions that return a Result<T, E>
    pub fn returns_result(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Result))
    }
    
    /// Matches functions that return a Result type where the error type implements the Error trait
    pub fn returns_result_with_error_impl(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::ResultWithErrorImpl))
    }
    
    /// Matches functions that return an Option<T>
    pub fn returns_option(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Option))
    }
    
    /// Matches functions that return a specific named type
    pub fn returns_type(&self, name: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Named(name.into())))
    }
    
    /// Matches functions that return a type matching a regex pattern
    pub fn returns_type_regex(&self, pattern: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Regex(pattern.into())))
    }
}

#[derive(Clone)]
pub enum FunctionMatchNode {
    Leaf(FunctionMatch),
    And(Box<FunctionMatchNode>, Box<FunctionMatchNode>),
    Or(Box<FunctionMatchNode>, Box<FunctionMatchNode>),
    Not(Box<FunctionMatchNode>),
}

impl FunctionMatchNode {
    pub fn and(self, other: FunctionMatchNode) -> Self {
        FunctionMatchNode::And(Box::new(self), Box::new(other))
    }
    
    pub fn or(self, other: FunctionMatchNode) -> Self {
        FunctionMatchNode::Or(Box::new(self), Box::new(other))
    }
    
    pub fn not(self) -> Self {
        FunctionMatchNode::Not(Box::new(self))
    }
    
    // Converts the DSL tree to the actual FunctionMatch
    pub fn build(self) -> FunctionMatch {
        match self {
            FunctionMatchNode::Leaf(matcher) => matcher,
            FunctionMatchNode::And(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                FunctionMatch::AndMatches(Box::new(a_match), Box::new(b_match))
            },
            FunctionMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                FunctionMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            },
            FunctionMatchNode::Not(m) => {
                let inner = m.build();
                FunctionMatch::NotMatch(Box::new(inner))
            }
        }
    }
}

// Factory function to create a matcher DSL
pub fn matcher<F>(f: F) -> FunctionMatch 
where 
    F: FnOnce(&FunctionMatcher) -> FunctionMatchNode 
{
    let matcher = FunctionMatcher;
    let node = f(&matcher);
    node.build()
}

// === Function Lint Types === //

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

// Fluent Builder for Function Lints
pub trait FunctionLintExt {
    fn function<'a>(&'a mut self) -> FunctionLintBuilder<'a>;
}

impl FunctionLintExt for LintBuilder {
    fn function<'a>(&'a mut self) -> FunctionLintBuilder<'a> {
        FunctionLintBuilder { parent: self }
    }
}

// First builder to establish a named lint
pub struct FunctionLintBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> FunctionLintBuilder<'a> {
    // Required step to name the lint
    pub fn lint_named(self, name: impl Into<String>) -> FunctionNamedBuilder<'a> {
        FunctionNamedBuilder { 
            parent: self.parent,
            name: name.into()
        }
    }
}

// Builder after the name is provided
pub struct FunctionNamedBuilder<'a> {
    parent: &'a mut LintBuilder,
    name: String,
}

impl<'a> FunctionNamedBuilder<'a> {
    // Original matches method now on NamedBuilder
    pub fn matches(self, m: FunctionMatch) -> FunctionConstraintBuilder<'a> {
        FunctionConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
            name: self.name,
        }
    }
    
    // New matcher method using the DSL
    pub fn matching<F>(self, f: F) -> FunctionConstraintBuilder<'a>
    where
        F: FnOnce(&FunctionMatcher) -> FunctionMatchNode
    {
        let matcher = matcher(f);
        self.matches(matcher)
    }
}

pub struct FunctionConstraintBuilder<'a> {
    parent: &'a mut LintBuilder,
    match_: FunctionMatch,
    rules: Vec<FunctionRule>,
    current_severity: Severity,
    name: String,
}

impl<'a> FunctionConstraintBuilder<'a> {
    // Private method to add a rule directly to self
    fn add_rule_internal(&mut self, rule: FunctionRule) {
        self.rules.push(rule);
    }
    
    // Public API method that takes and returns self
    pub fn add_rule(mut self, rule: FunctionRule) -> Self {
        self.add_rule_internal(rule);
        self
    }
    
    pub fn build(self) -> &'a mut LintBuilder {
        let lint = ConfiguredLint::Function(FunctionLint {
            name: self.name,
            matches: self.match_,
            rules: self.rules,
        });
        self.parent.push(lint);
        self.parent
    }
    
    // Set the severity level for subsequent rules
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.current_severity = severity;
        self
    }
    
    // Helper method for function length limit
    pub fn max_length(mut self, length: usize) -> Self {
        self.add_rule_internal(FunctionRule::MaxLength(length, self.current_severity));
        self
    }
    
    // Helper method for Result error type check
    pub fn enforce_error_trait_implementation(mut self) -> Self {
        self.add_rule_internal(FunctionRule::ResultErrorMustImplementError(self.current_severity));
        self
    }
    
    // Create a MaxLength rule that can be used in combinations
    pub fn create_max_length_rule(&self, length: usize) -> FunctionRule {
        FunctionRule::MaxLength(length, self.current_severity)
    }
} 