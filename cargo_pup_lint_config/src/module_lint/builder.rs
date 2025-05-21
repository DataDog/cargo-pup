use super::matcher::{ModuleMatchNode, ModuleMatcher, matcher};
use super::types::{ModuleLint, ModuleMatch, ModuleRule};
use crate::lint_builder::LintBuilder;
use crate::{ConfiguredLint, Severity};

/// Extension trait that adds module linting capabilities to LintBuilder
pub trait ModuleLintExt {
    /// Start building a module lint rule
    fn module_lint(&mut self) -> ModuleLintBuilder;
}

impl ModuleLintExt for LintBuilder {
    fn module_lint(&mut self) -> ModuleLintBuilder {
        ModuleLintBuilder { parent: self }
    }
}

/// Initial builder for creating a module lint
pub struct ModuleLintBuilder<'a> {
    parent: &'a mut LintBuilder,
}

impl<'a> ModuleLintBuilder<'a> {
    /// Give the lint a name
    pub fn lint_named(self, name: impl Into<String>) -> ModuleNamedBuilder<'a> {
        ModuleNamedBuilder {
            parent: self.parent,
            name: name.into(),
        }
    }
}

/// Builder used after naming the lint
pub struct ModuleNamedBuilder<'a> {
    parent: &'a mut LintBuilder,
    name: String,
}

impl<'a> ModuleNamedBuilder<'a> {
    /// Directly provide a module matcher
    pub fn matches(self, m: ModuleMatch) -> ModuleConstraintBuilder<'a> {
        ModuleConstraintBuilder {
            parent: self.parent,
            match_: m,
            rules: Vec::new(),
            current_severity: Severity::default(),
            name: self.name,
        }
    }

    /// Define module matching using the fluent DSL
    /// 
    /// # Example
    /// ```
    /// use cargo_pup_lint_config::{LintBuilder, ModuleLintExt};
    /// let mut lint_builder = LintBuilder::new();
    /// lint_builder.module_lint()
    ///     .lint_named("empty_handlers")
    ///     .matching(|m| m.module("handlers"))
    ///     .must_be_empty()
    ///     .build();
    /// ```
    pub fn matching<F>(self, f: F) -> ModuleConstraintBuilder<'a>
    where
        F: FnOnce(&ModuleMatcher) -> ModuleMatchNode,
    {
        let matcher = matcher(f);
        self.matches(matcher)
    }
}

/// Builder for adding rules to a module lint
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

    /// Add a custom rule to the module lint
    pub fn add_rule(mut self, rule: ModuleRule) -> Self {
        self.add_rule_internal(rule);
        self
    }

    /// Finalize the module lint and return to the parent builder
    pub fn build(self) -> &'a mut LintBuilder {
        let lint = ConfiguredLint::Module(ModuleLint {
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

    /// Add a rule requiring the module to have at least one item
    pub fn must_not_be_empty(mut self) -> Self {
        self.add_rule_internal(ModuleRule::MustNotBeEmpty(self.current_severity));
        self
    }

    /// Add a rule requiring the module to be empty
    pub fn must_be_empty(mut self) -> Self {
        self.add_rule_internal(ModuleRule::MustBeEmpty(self.current_severity));
        self
    }

    /// Add a rule requiring the module.rs file to only re-export other modules
    pub fn must_have_empty_mod_file(mut self) -> Self {
        self.add_rule_internal(ModuleRule::MustHaveEmptyModFile(self.current_severity));
        self
    }

    /// Add a rule prohibiting wildcard imports (use path::*)
    pub fn no_wildcard_imports(mut self) -> Self {
        self.add_rule_internal(ModuleRule::NoWildcardImports(self.current_severity));
        self
    }

    /// Add a rule to restrict imports by specifying allowed/denied modules
    /// 
    /// @param allowed_only - If provided, only these imports are allowed
    /// @param denied - If provided, these imports are explicitly prohibited
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

    /// Add a rule requiring the module to have a specific name
    pub fn must_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(ModuleRule::MustBeNamed(name, self.current_severity));
        self
    }

    /// Add a rule prohibiting the module from having a specific name
    pub fn must_not_be_named(mut self, name: String) -> Self {
        self.add_rule_internal(ModuleRule::MustNotBeNamed(name, self.current_severity));
        self
    }

    /// Add a rule prohibiting specific items from being defined in the module
    pub fn denied_items(mut self, items: Vec<String>) -> Self {
        self.add_rule_internal(ModuleRule::DeniedItems {
            items,
            severity: self.current_severity,
        });
        self
    }
}
