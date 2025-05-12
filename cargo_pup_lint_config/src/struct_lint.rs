use serde::{Deserialize, Serialize};
use crate::ConfiguredLint;

#[derive(Debug, Serialize, Deserialize)]
pub enum StructMatch {
    NameEquals(String),
    HasAttribute(String),
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
    pub fn matches(self, m: StructMatch) -> StructConstraintBuilder<'a> {
        StructConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
        }
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
