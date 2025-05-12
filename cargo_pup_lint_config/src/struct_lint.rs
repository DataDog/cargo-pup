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
    pub rule: StructRule,
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
        }
    }
}

pub struct StructConstraintBuilder<'a> {
    parent: &'a mut crate::lint_builder::LintBuilder,
    match_: StructMatch,
}

impl<'a> StructConstraintBuilder<'a> {
    pub fn constraints(self, rule: StructRule) -> &'a mut crate::lint_builder::LintBuilder {
        let lint = ConfiguredLint::Struct(StructLint {
            name: "struct_lint".into(),
            matches: self.match_,
            rule,
        });
        self.parent.push(lint);
        self.parent
    }
}
