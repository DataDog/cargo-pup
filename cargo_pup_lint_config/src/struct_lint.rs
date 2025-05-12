use serde::{Deserialize, Serialize};
use crate::ConfiguredLint;

// === Struct Matcher DSL === //

pub struct StructMatcher;

impl StructMatcher {
    pub fn name(&self, name: impl Into<String>) -> StructMatchNode {
        StructMatchNode::Leaf(StructMatch::NameEquals(name.into()))
    }
    
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
    NameEquals(String),
    HasAttribute(String),
    // Add logical operations
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

#[derive(Debug, Serialize, Deserialize)]
pub enum StructRule {
    MustBeNamed(String),
    MustNotBeNamed(String),
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
}

impl<'a> StructConstraintBuilder<'a> {
    pub fn add_rule(mut self, rule: StructRule) -> Self {
        self.rules.push(rule);
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
    
    pub fn must_be_named(self, name: String) -> Self {
        self.add_rule(StructRule::MustBeNamed(name))
    }
    
    pub fn must_not_be_named(self, name: String) -> Self {
        self.add_rule(StructRule::MustNotBeNamed(name))
    }
}
