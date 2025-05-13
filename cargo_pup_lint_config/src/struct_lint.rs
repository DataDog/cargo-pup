use serde::{Deserialize, Serialize};
use crate::ConfiguredLint;
use super::Severity;
use regex::Regex;

// === Struct Matcher DSL === //

pub struct StructMatcher;

impl StructMatcher {
    /// Matches structs by name
    /// 
    /// The name parameter can be either:
    /// - An exact struct name (e.g., "User")
    /// - A regular expression pattern (e.g., "^[A-Z][a-z]+Model$")
    ///
    /// The implementation will determine if it's a regex based on the presence of special regex characters.
    pub fn name(&self, name: impl Into<String>) -> StructMatchNode {
        StructMatchNode::Leaf(StructMatch::Name(name.into()))
    }
    
    /// Matches structs by attribute
    /// 
    /// The attribute parameter can be either:
    /// - An exact attribute (e.g., "derive(Debug)")
    /// - A regular expression pattern (e.g., "derive\\(.*Debug.*\\)")
    ///
    /// The implementation will determine if it's a regex based on the presence of special regex characters.
    pub fn has_attribute(&self, attr: impl Into<String>) -> StructMatchNode {
        StructMatchNode::Leaf(StructMatch::HasAttribute(attr.into()))
    }
}

#[derive(Clone)]
pub enum StructMatchNode {
    Leaf(StructMatch),
    And(Box<StructMatchNode>, Box<StructMatchNode>),
    Or(Box<StructMatchNode>, Box<StructMatchNode>),
    Not(Box<StructMatchNode>),
}

impl StructMatchNode {
    pub fn and(self, other: StructMatchNode) -> Self {
        StructMatchNode::And(Box::new(self), Box::new(other))
    }
    
    pub fn or(self, other: StructMatchNode) -> Self {
        StructMatchNode::Or(Box::new(self), Box::new(other))
    }
    
    pub fn not(self) -> Self {
        StructMatchNode::Not(Box::new(self))
    }
    
    // Converts the DSL tree to the actual StructMatch
    pub fn build(self) -> StructMatch {
        match self {
            StructMatchNode::Leaf(matcher) => matcher,
            StructMatchNode::And(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                StructMatch::AndMatches(Box::new(a_match), Box::new(b_match))
            },
            StructMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                StructMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            },
            StructMatchNode::Not(m) => {
                let inner = m.build();
                StructMatch::NotMatch(Box::new(inner))
            }
        }
    }
}

// Factory function to create a matcher DSL
pub fn matcher<F>(f: F) -> StructMatch 
where 
    F: FnOnce(&StructMatcher) -> StructMatchNode 
{
    let matcher = StructMatcher;
    let node = f(&matcher);
    node.build()
}

// === Struct Lint Types === //

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructMatch {
    Name(String),
    HasAttribute(String),
    // Logical operations
    AndMatches(Box<StructMatch>, Box<StructMatch>),
    OrMatches(Box<StructMatch>, Box<StructMatch>),
    NotMatch(Box<StructMatch>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructLint {
    pub name: String,
    pub matches: StructMatch,
    pub rules: Vec<StructRule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StructRule {
    MustBeNamed(String, Severity),
    MustNotBeNamed(String, Severity),
    And(Box<StructRule>, Box<StructRule>),
    Or(Box<StructRule>, Box<StructRule>),
    Not(Box<StructRule>),
}

// === Fluent Builder for Struct Lints === //

pub trait StructLintExt {
    fn struct_lint<'a>(&'a mut self) -> StructMatchBuilder<'a>;
}

impl StructLintExt for crate::lint_builder::LintBuilder {
    fn struct_lint<'a>(&'a mut self) -> StructMatchBuilder<'a> {
        StructMatchBuilder { parent: self }
    }
}

pub struct StructMatchBuilder<'a> {
    parent: &'a mut crate::lint_builder::LintBuilder,
}

impl<'a> StructMatchBuilder<'a> {
    // Original matches method
    pub fn matches(self, m: StructMatch) -> StructConstraintBuilder<'a> {
        StructConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
        }
    }
    
    // New matcher method using the DSL
    pub fn matching<F>(self, f: F) -> StructConstraintBuilder<'a>
    where
        F: FnOnce(&StructMatcher) -> StructMatchNode
    {
        let matcher = matcher(f);
        self.matches(matcher)
    }
}

pub struct StructConstraintBuilder<'a> {
    parent: &'a mut crate::lint_builder::LintBuilder,
    match_: StructMatch,
    rules: Vec<StructRule>,
    current_severity: Severity,
}

impl<'a> StructConstraintBuilder<'a> {
    // Private method to add a rule directly to self
    fn add_rule_internal(&mut self, rule: StructRule) {
        self.rules.push(rule);
    }
    
    // Public API method that takes and returns self
    pub fn add_rule(mut self, rule: StructRule) -> Self {
        self.add_rule_internal(rule);
        self
    }
    
    pub fn build(self) -> &'a mut crate::lint_builder::LintBuilder {
        let lint = ConfiguredLint::Struct(StructLint {
            name: "struct_lint".into(),
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
    
    pub fn must_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(StructRule::MustBeNamed(name, self.current_severity));
        self
    }
    
    pub fn must_not_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(StructRule::MustNotBeNamed(name, self.current_severity));
        self
    }
}
