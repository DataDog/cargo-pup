use super::matcher::{StructMatchNode, StructMatcher, matcher};
use super::types::{StructLint, StructMatch, StructRule};
use crate::lint_builder::LintBuilder;
use crate::{ConfiguredLint, Severity};

// === Fluent Builder for Struct Lints === //

// This is the trait for struct lint operations, ideally these would be implemented
// for LintBuilder, but for now we'll use the builder pattern approach
pub trait StructLintExt {
    fn struct_lint(&mut self) -> StructLintBuilder;
}

impl StructLintExt for LintBuilder {
    fn struct_lint(&mut self) -> StructLintBuilder {
        StructLintBuilder { parent: self }
    }
}

// First builder to establish a named lint
pub struct StructLintBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> StructLintBuilder<'a> {
    // Required step to name the lint
    pub fn lint_named(self, name: impl Into<String>) -> StructNamedBuilder<'a> {
        StructNamedBuilder {
            parent: self.parent,
            name: name.into(),
        }
    }
}

// Builder after the name is provided
pub struct StructNamedBuilder<'a> {
    parent: &'a mut LintBuilder,
    name: String,
}

impl<'a> StructNamedBuilder<'a> {
    // Original matches method now on NamedBuilder
    pub fn matches(self, m: StructMatch) -> StructConstraintBuilder<'a> {
        StructConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
            name: self.name,
        }
    }

    // New matcher method using the DSL
    pub fn matching<F>(self, f: F) -> StructConstraintBuilder<'a>
    where
        F: FnOnce(&StructMatcher) -> StructMatchNode,
    {
        let matcher = matcher(f);
        self.matches(matcher)
    }
}

pub struct StructConstraintBuilder<'a> {
    parent: &'a mut LintBuilder,
    match_: StructMatch,
    rules: Vec<StructRule>,
    current_severity: Severity,
    name: String,
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

    pub fn build(self) -> &'a mut LintBuilder {
        let lint = ConfiguredLint::Struct(StructLint {
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

    pub fn must_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(StructRule::MustBeNamed(name, self.current_severity));
        self
    }

    pub fn must_not_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(StructRule::MustNotBeNamed(name, self.current_severity));
        self
    }

    // Add new visibility rule methods
    pub fn must_be_private(mut self) -> Self {
        self.add_rule_internal(StructRule::MustBePrivate(self.current_severity));
        self
    }

    pub fn must_be_public(mut self) -> Self {
        self.add_rule_internal(StructRule::MustBePublic(self.current_severity));
        self
    }
}
