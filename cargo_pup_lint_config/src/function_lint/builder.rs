use super::matcher::{FunctionMatchNode, FunctionMatcher, matcher};
use super::types::{FunctionLint, FunctionMatch, FunctionRule};
use crate::lint_builder::LintBuilder;
use crate::{ConfiguredLint, Severity};

/// Extension trait that adds function linting capabilities to LintBuilder
pub trait FunctionLintExt {
    /// Build a lint rule targeting functions
    fn function_lint(&mut self) -> FunctionLintBuilder<'_>;
}

impl FunctionLintExt for LintBuilder {
    fn function_lint(&mut self) -> FunctionLintBuilder<'_> {
        FunctionLintBuilder { parent: self }
    }
}

/// Initial builder for creating a function lint
pub struct FunctionLintBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> FunctionLintBuilder<'a> {
    /// Give the lint a name
    pub fn lint_named(self, name: impl Into<String>) -> FunctionNamedBuilder<'a> {
        FunctionNamedBuilder {
            parent: self.parent,
            name: name.into(),
        }
    }
}

/// Builder used after naming the lint
pub struct FunctionNamedBuilder<'a> {
    parent: &'a mut LintBuilder,
    name: String,
}

impl<'a> FunctionNamedBuilder<'a> {
    /// Directly provide a function matcher
    pub fn matches(self, m: FunctionMatch) -> FunctionConstraintBuilder<'a> {
        FunctionConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
            name: self.name,
        }
    }

    /// Define function matching using the fluent DSL
    /// 
    /// # Example
    /// ```
    /// lint_builder.function_lint()
    ///     .lint_named("result_error_impl")
    ///     .matching(|m| m.returns_result())
    ///     .enforce_error_trait_implementation()
    ///     .build();
    /// ```
    pub fn matching<F>(self, f: F) -> FunctionConstraintBuilder<'a>
    where
        F: FnOnce(&FunctionMatcher) -> FunctionMatchNode,
    {
        let matcher = matcher(f);
        self.matches(matcher)
    }
}

/// Builder for adding rules to a function lint
pub struct FunctionConstraintBuilder<'a> {
    parent: &'a mut LintBuilder,
    match_: FunctionMatch,
    rules: Vec<FunctionRule>,
    current_severity: Severity,
    name: String,
}

impl<'a> FunctionConstraintBuilder<'a> {
    // Private method to add a rule directly to self
    fn add_rule_internal(&mut self, rule: FunctionRule) {
        self.rules.push(rule);
    }

    /// Add a custom rule to the function lint
    pub fn add_rule(mut self, rule: FunctionRule) -> Self {
        self.add_rule_internal(rule);
        self
    }

    /// Finalize the function lint and return to the parent builder
    pub fn build(self) -> &'a mut LintBuilder {
        let lint = ConfiguredLint::Function(FunctionLint {
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

    /// Limit function length to the specified number of lines
    pub fn max_length(mut self, length: usize) -> Self {
        self.add_rule_internal(FunctionRule::MaxLength(length, self.current_severity));
        self
    }

    /// Require Result error types to implement the Error trait
    pub fn enforce_error_trait_implementation(mut self) -> Self {
        self.add_rule_internal(FunctionRule::ResultErrorMustImplementError(
            self.current_severity,
        ));
        self
    }

    /// Create a new MaxLength rule with the current severity
    pub fn create_max_length_rule(&self, length: usize) -> FunctionRule {
        FunctionRule::MaxLength(length, self.current_severity)
    }
}
