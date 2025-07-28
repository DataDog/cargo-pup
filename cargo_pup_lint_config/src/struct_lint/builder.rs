// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use super::matcher::{StructMatchNode, StructMatcher, matcher};
use super::types::{StructLint, StructMatch, StructRule};
use crate::lint_builder::LintBuilder;
use crate::{ConfiguredLint, Severity};

/// Extension trait that adds struct linting capabilities to LintBuilder
pub trait StructLintExt {
    /// Start building a struct lint rule
    fn struct_lint(&mut self) -> StructLintBuilder<'_>;
}

impl StructLintExt for LintBuilder {
    fn struct_lint(&mut self) -> StructLintBuilder<'_> {
        StructLintBuilder { parent: self }
    }
}

/// Initial builder for creating a struct lint
pub struct StructLintBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> StructLintBuilder<'a> {
    /// Give the lint a name
    pub fn lint_named(self, name: impl Into<String>) -> StructNamedBuilder<'a> {
        StructNamedBuilder {
            parent: self.parent,
            name: name.into(),
        }
    }
}

/// Builder used after naming the lint
pub struct StructNamedBuilder<'a> {
    parent: &'a mut LintBuilder,
    name: String,
}

impl<'a> StructNamedBuilder<'a> {
    /// Directly provide a struct matcher
    pub fn matches(self, m: StructMatch) -> StructConstraintBuilder<'a> {
        StructConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
            name: self.name,
        }
    }

    /// Define struct matching using the fluent DSL
    ///
    /// # Example
    /// ```
    /// use cargo_pup_lint_config::{LintBuilder, StructLintExt};
    /// let mut lint_builder = LintBuilder::new();
    /// lint_builder.struct_lint()
    ///     .lint_named("model_visibility")
    ///     .matching(|m| m.name(".*Model"))
    ///     .must_be_private()
    ///     .build();
    /// ```
    pub fn matching<F>(self, f: F) -> StructConstraintBuilder<'a>
    where
        F: FnOnce(&StructMatcher) -> StructMatchNode,
    {
        let matcher = matcher(f);
        self.matches(matcher)
    }
}

/// Builder for adding rules to a struct lint
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

    /// Add a custom rule to the struct lint
    pub fn add_rule(mut self, rule: StructRule) -> Self {
        self.add_rule_internal(rule);
        self
    }

    /// Finalize the struct lint and return to the parent builder
    pub fn build(self) -> &'a mut LintBuilder {
        let lint = ConfiguredLint::Struct(StructLint {
            name: self.name,
            matches: self.match_,
            rules: self.rules,
        });
        self.parent.push(lint);
        self.parent
    }

    /// Set the severity level for all subsequently added rules
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.current_severity = severity;
        self
    }

    /// Add a rule requiring the struct to have a specific name
    pub fn must_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(StructRule::MustBeNamed(name, self.current_severity));
        self
    }

    /// Add a rule prohibiting the struct from having a specific name
    pub fn must_not_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(StructRule::MustNotBeNamed(name, self.current_severity));
        self
    }

    /// Add a rule requiring the struct to have private visibility
    pub fn must_be_private(mut self) -> Self {
        self.add_rule_internal(StructRule::MustBePrivate(self.current_severity));
        self
    }

    /// Add a rule requiring the struct to implement a given trait
    pub fn must_implement_trait(mut self, trait_path: impl Into<String>) -> Self {
        self.add_rule_internal(StructRule::ImplementsTrait(
            trait_path.into(),
            self.current_severity,
        ));
        self
    }

    /// Add a rule requiring the struct NOT to implement a given trait
    pub fn must_not_implement_trait(mut self, trait_path: impl Into<String>) -> Self {
        let inner = StructRule::ImplementsTrait(trait_path.into(), self.current_severity);
        self.add_rule_internal(StructRule::Not(Box::new(inner)));
        self
    }

    /// Add a rule requiring the struct to have public visibility
    pub fn must_be_public(mut self) -> Self {
        self.add_rule_internal(StructRule::MustBePublic(self.current_severity));
        self
    }
}
