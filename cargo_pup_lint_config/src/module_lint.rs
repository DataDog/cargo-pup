use serde::{Deserialize, Serialize};
use crate::lint_builder::LintBuilder;
use super::{ConfiguredLint};

// === Module Matcher DSL === //

pub struct ModuleMatcher;

impl ModuleMatcher {
    pub fn namespace(&self, ns: impl Into<String>) -> ModuleMatchNode {
        ModuleMatchNode::Leaf(ModuleMatch::NamespaceEquals(ns.into()))
    }
    
    pub fn path_contains(&self, path: impl Into<String>) -> ModuleMatchNode {
        ModuleMatchNode::Leaf(ModuleMatch::PathContains(path.into()))
    }
}

#[derive(Clone)]
pub enum ModuleMatchNode {
    Leaf(ModuleMatch),
    And(Box<ModuleMatchNode>, Box<ModuleMatchNode>),
    Or(Box<ModuleMatchNode>, Box<ModuleMatchNode>),
    Not(Box<ModuleMatchNode>),
}

impl ModuleMatchNode {
    pub fn and(self, other: ModuleMatchNode) -> Self {
        ModuleMatchNode::And(Box::new(self), Box::new(other))
    }
    
    pub fn or(self, other: ModuleMatchNode) -> Self {
        ModuleMatchNode::Or(Box::new(self), Box::new(other))
    }
    
    pub fn not(self) -> Self {
        ModuleMatchNode::Not(Box::new(self))
    }
    
    // Converts the DSL tree to the actual ModuleMatch
    pub fn build(self) -> ModuleMatch {
        match self {
            ModuleMatchNode::Leaf(matcher) => matcher,
            ModuleMatchNode::And(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                ModuleMatch::AndMatches(Box::new(a_match), Box::new(b_match))
            },
            ModuleMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                ModuleMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            },
            ModuleMatchNode::Not(m) => {
                let inner = m.build();
                ModuleMatch::NotMatch(Box::new(inner))
            }
        }
    }
}

// Factory function to create a matcher DSL
pub fn matcher<F>(f: F) -> ModuleMatch 
where 
    F: FnOnce(&ModuleMatcher) -> ModuleMatchNode 
{
    let matcher = ModuleMatcher;
    let node = f(&matcher);
    node.build()
}

// === Module Lint Types === //

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleMatch {
    NamespaceEquals(String),
    PathContains(String),
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleRule {
    MustBeNamed(String),
    MustNotBeNamed(String),
    MustNotBeEmpty,
    RestrictImports {
        allowed_only: Option<Vec<String>>,
        denied: Option<Vec<String>>,
    },
    NoWildcardImports,
    And(Box<ModuleRule>, Box<ModuleRule>),
    Or(Box<ModuleRule>, Box<ModuleRule>),
    Not(Box<ModuleRule>),
}

// Fluent Builder for Module Lints
pub trait ModuleLintExt {
    fn module<'a>(&'a mut self) -> ModuleMatchBuilder<'a>;
}

impl ModuleLintExt for LintBuilder {
    fn module<'a>(&'a mut self) -> ModuleMatchBuilder<'a> {
        ModuleMatchBuilder { parent: self }
    }
}

pub struct ModuleMatchBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> ModuleMatchBuilder<'a> {
    pub fn matches(self, m: ModuleMatch) -> ModuleConstraintBuilder<'a> {
        ModuleConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
        }
    }
    
    pub fn matching<F>(self, f: F) -> ModuleConstraintBuilder<'a>
    where
        F: FnOnce(&ModuleMatcher) -> ModuleMatchNode
    {
        let matcher = matcher(f);
        self.matches(matcher)
    }
}

pub struct ModuleConstraintBuilder<'a> {
    parent: &'a mut LintBuilder,
    match_: ModuleMatch,
    rules: Vec<ModuleRule>,
}

impl<'a> ModuleConstraintBuilder<'a> {
    pub fn add_rule(mut self, rule: ModuleRule) -> Self {
        self.rules.push(rule);
        self
    }
    
    pub fn build(self) -> &'a mut LintBuilder {
        let lint = ConfiguredLint::Module(ModuleLint {
            name: "module_lint".into(),
            matches: self.match_,
            rules: self.rules,
        });
        self.parent.push(lint);
        self.parent
    }
    
    pub fn must_not_be_empty(self) -> Self {
        self.add_rule(ModuleRule::MustNotBeEmpty)
    }
    
    pub fn no_wildcard_imports(self) -> Self {
        self.add_rule(ModuleRule::NoWildcardImports)
    }
    
    pub fn restrict_imports(
        self, 
        allowed_only: Option<Vec<String>>, 
        denied: Option<Vec<String>>
    ) -> Self {
        self.add_rule(ModuleRule::RestrictImports { 
            allowed_only, 
            denied 
        })
    }
    
    pub fn must_be_named(self, name: String) -> Self {
        self.add_rule(ModuleRule::MustBeNamed(name))
    }
    
    pub fn must_not_be_named(self, name: String) -> Self {
        self.add_rule(ModuleRule::MustNotBeNamed(name))
    }
}