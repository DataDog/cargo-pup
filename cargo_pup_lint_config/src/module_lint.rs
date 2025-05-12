use serde::{Deserialize, Serialize};
use crate::lint_builder::LintBuilder;
use super::{ConfiguredLint};


#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleMatch {
    NamespaceEquals(String),
    PathContains(String),
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
    
    // Helper method for feature #10: Empty Module Detection
    pub fn must_not_be_empty(self) -> Self {
        self.add_rule(ModuleRule::MustNotBeEmpty)
    }
    
    // Helper method for feature #5: Wildcard Imports Detection
    pub fn no_wildcard_imports(self) -> Self {
        self.add_rule(ModuleRule::NoWildcardImports)
    }
    
    // Helper method for feature #4: Fine-grained Module Import Rules
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