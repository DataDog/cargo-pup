use super::matcher::{ModuleMatchNode, ModuleMatcher, matcher};
use super::types::{ModuleLint, ModuleMatch, ModuleRule};
use crate::lint_builder::LintBuilder;
use crate::{ConfiguredLint, Severity};

// Fluent Builder for Module Lints
pub trait ModuleLintExt {
    fn module(&mut self) -> ModuleLintBuilder;
}

impl ModuleLintExt for LintBuilder {
    fn module(&mut self) -> ModuleLintBuilder {
        ModuleLintBuilder { parent: self }
    }
}

// First builder to establish a named lint
pub struct ModuleLintBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> ModuleLintBuilder<'a> {
    // Required step to name the lint
    pub fn lint_named(self, name: impl Into<String>) -> ModuleNamedBuilder<'a> {
        ModuleNamedBuilder {
            parent: self.parent,
            name: name.into(),
        }
    }
}

// Builder after the name is provided
pub struct ModuleNamedBuilder<'a> {
    parent: &'a mut LintBuilder,
    name: String,
}

impl<'a> ModuleNamedBuilder<'a> {
    // Original matches method now on NamedBuilder
    pub fn matches(self, m: ModuleMatch) -> ModuleConstraintBuilder<'a> {
        ModuleConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
            name: self.name,
        }
    }

    // New matcher method using the DSL
    pub fn matching<F>(self, f: F) -> ModuleConstraintBuilder<'a>
    where
        F: FnOnce(&ModuleMatcher) -> ModuleMatchNode,
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
    name: String,
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

    // Helper method for feature #10: Empty Module Detection
    pub fn must_not_be_empty(mut self) -> Self {
        self.add_rule_internal(ModuleRule::MustNotBeEmpty(self.current_severity));
        self
    }

    // Helper method for requiring a module to be empty
    pub fn must_be_empty(mut self) -> Self {
        self.add_rule_internal(ModuleRule::MustBeEmpty(self.current_severity));
        self
    }

    // Helper method for requiring a module.rs file to be empty (only allowed to export other modules)
    pub fn must_have_empty_mod_file(mut self) -> Self {
        self.add_rule_internal(ModuleRule::MustHaveEmptyModFile(self.current_severity));
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
        denied: Option<Vec<String>>,
    ) -> Self {
        self.add_rule_internal(ModuleRule::RestrictImports {
            allowed_only,
            denied,
            severity: self.current_severity,
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
            severity: self.current_severity,
        });
        self
    }
}
