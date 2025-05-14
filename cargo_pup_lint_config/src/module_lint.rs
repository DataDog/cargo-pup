use serde::{Deserialize, Serialize};
use crate::lint_builder::LintBuilder;
use super::{ConfiguredLint, Severity};
use regex::Regex;

// === Module Matcher DSL === //

pub struct ModuleMatcher;

impl ModuleMatcher {

    pub fn module(&self, module: impl Into<String>) -> ModuleMatchNode {
        ModuleMatchNode::Leaf(ModuleMatch::Module(module.into()))
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
    Module(String),
    // Logical operations
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleRule {
    MustBeNamed(String, Severity),
    MustNotBeNamed(String, Severity),
    MustNotBeEmpty(Severity),
    RestrictImports {
        allowed_only: Option<Vec<String>>,
        denied: Option<Vec<String>>,
        severity: Severity,
    },
    NoWildcardImports(Severity),
    DeniedItems {
        items: Vec<String>,
        severity: Severity,
    },
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
    // Original matches method
    pub fn matches(self, m: ModuleMatch) -> ModuleConstraintBuilder<'a> {
        ModuleConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
        }
    }
    
    // New matcher method using the DSL
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
    current_severity: Severity,
}

impl<'a> ModuleConstraintBuilder<'a> {
    // Private method to add a rule directly to self
    fn add_rule_internal(&mut self, rule: ModuleRule) {
        self.rules.push(rule);
    }
    
    // Public API method that takes and returns self
    pub fn add_rule(mut self, rule: ModuleRule) -> Self {
        self.add_rule_internal(rule);
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
    
    // Set the severity level for subsequent rules
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.current_severity = severity;
        self
    }
    
    // Helper method for feature #10: Empty Module Detection
    pub fn must_not_be_empty(mut self) -> Self {
        self.add_rule_internal(ModuleRule::MustNotBeEmpty(self.current_severity));
        self
    }
    
    // Helper method for feature #5: Wildcard Imports Detection
    pub fn no_wildcard_imports(mut self) -> Self {
        self.add_rule_internal(ModuleRule::NoWildcardImports(self.current_severity));
        self
    }
    
    // Helper method for feature #4: Fine-grained Module Import Rules
    pub fn restrict_imports(
        mut self, 
        allowed_only: Option<Vec<String>>, 
        denied: Option<Vec<String>>
    ) -> Self {
        self.add_rule_internal(ModuleRule::RestrictImports { 
            allowed_only, 
            denied,
            severity: self.current_severity
        });
        self
    }
    
    pub fn must_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(ModuleRule::MustBeNamed(name, self.current_severity));
        self
    }
    
    pub fn must_not_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(ModuleRule::MustNotBeNamed(name, self.current_severity));
        self
    }
    
    // Helper method for DeniedItems rule
    pub fn denied_items(mut self, items: Vec<String>) -> Self {
        self.add_rule_internal(ModuleRule::DeniedItems { 
            items, 
            severity: self.current_severity 
        });
        self
    }
}