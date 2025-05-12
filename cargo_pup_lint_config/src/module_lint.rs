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
    pub rule: ModuleRule,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleRule {
    MustBeNamed(String),
    MustNotBeNamed(String),
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
        }
    }
}

pub struct ModuleConstraintBuilder<'a> {
    parent: &'a mut LintBuilder,
    match_: ModuleMatch,
}

impl<'a> ModuleConstraintBuilder<'a> {
    pub fn constraints(self, rule: ModuleRule) -> &'a mut LintBuilder {
        let lint = ConfiguredLint::Module(ModuleLint {
            name: "module_lint".into(),
            matches: self.match_,
            rule,
        });
        self.parent.push(lint);
        self.parent
    }
}