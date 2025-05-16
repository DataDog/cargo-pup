use crate::lint_builder::LintBuilder;
use crate::{ConfiguredLint, Severity};
use super::types::{FunctionLint, FunctionMatch, FunctionRule};
use super::matcher::{matcher, FunctionMatcher, FunctionMatchNode};

// Fluent Builder for Function Lints
pub trait FunctionLintExt {
    fn function<'a>(&'a mut self) -> FunctionLintBuilder<'a>;
}

impl FunctionLintExt for LintBuilder {
    fn function<'a>(&'a mut self) -> FunctionLintBuilder<'a> {
        FunctionLintBuilder { parent: self }
    }
}

// First builder to establish a named lint
pub struct FunctionLintBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> FunctionLintBuilder<'a> {
    // Required step to name the lint
    pub fn lint_named(self, name: impl Into<String>) -> FunctionNamedBuilder<'a> {
        FunctionNamedBuilder { 
            parent: self.parent,
            name: name.into()
        }
    }
}

// Builder after the name is provided
pub struct FunctionNamedBuilder<'a> {
    parent: &'a mut LintBuilder,
    name: String,
}

impl<'a> FunctionNamedBuilder<'a> {
    // Original matches method now on NamedBuilder
    pub fn matches(self, m: FunctionMatch) -> FunctionConstraintBuilder<'a> {
        FunctionConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
            name: self.name,
        }
    }
    
    // New matcher method using the DSL
    pub fn matching<F>(self, f: F) -> FunctionConstraintBuilder<'a>
    where
        F: FnOnce(&FunctionMatcher) -> FunctionMatchNode
    {
        let matcher = matcher(f);
        self.matches(matcher)
    }
}

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
    
    // Public API method that takes and returns self
    pub fn add_rule(mut self, rule: FunctionRule) -> Self {
        self.add_rule_internal(rule);
        self
    }
    
    pub fn build(self) -> &'a mut LintBuilder {
        let lint = ConfiguredLint::Function(FunctionLint {
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
    
    // Helper method for function length limit
    pub fn max_length(mut self, length: usize) -> Self {
        self.add_rule_internal(FunctionRule::MaxLength(length, self.current_severity));
        self
    }
    
    // Helper method for Result error type check
    pub fn enforce_error_trait_implementation(mut self) -> Self {
        self.add_rule_internal(FunctionRule::ResultErrorMustImplementError(self.current_severity));
        self
    }
    
    // Create a MaxLength rule that can be used in combinations
    pub fn create_max_length_rule(&self, length: usize) -> FunctionRule {
        FunctionRule::MaxLength(length, self.current_severity)
    }
}