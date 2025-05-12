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
pub enum FunctionMatch {
    NameEquals(String),
    NameRegex(String),
    InModule(String),
    // Logical operations
    AndMatches(Box<FunctionMatch>, Box<FunctionMatch>),
    OrMatches(Box<FunctionMatch>, Box<FunctionMatch>),
    NotMatch(Box<FunctionMatch>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionLint {
    pub name: String,
    pub matches: FunctionMatch,
    pub rules: Vec<FunctionRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FunctionRule {
    MaxLength(usize, Severity),
    And(Box<FunctionRule>, Box<FunctionRule>),
    Or(Box<FunctionRule>, Box<FunctionRule>),
    Not(Box<FunctionRule>),
}

// Fluent Builder for Function Lints
pub trait FunctionLintExt {
    fn function<'a>(&'a mut self) -> FunctionMatchBuilder<'a>;
}

impl FunctionLintExt for LintBuilder {
    fn function<'a>(&'a mut self) -> FunctionMatchBuilder<'a> {
        FunctionMatchBuilder { parent: self }
    }
}

pub struct FunctionMatchBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> FunctionMatchBuilder<'a> {
    // Original matches method
    pub fn matches(self, m: FunctionMatch) -> FunctionConstraintBuilder<'a> {
        FunctionConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
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
            name: "function_lint".into(),
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
} 