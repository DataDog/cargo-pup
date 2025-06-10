// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use crate::ArchitectureLintRule;
use crate::declare_variable_severity_lint;
use crate::helpers::lint_helpers::span_lint_and_help;
use crate::helpers::queries::get_full_module_name;
use cargo_pup_lint_config::{ConfiguredLint, ModuleMatch, ModuleRule, Severity};
use cargo_pup_lint_config::module_lint::ModuleLint as ConfigModuleLint;
use regex::Regex;
use rustc_hir::{Item, ItemKind, UseKind};
use rustc_lint::{LateContext, LateLintPass, LintContext, LintStore};
use rustc_session::impl_lint_pass;

pub struct ModuleLint {
    // Store the original configuration
    config: ConfigModuleLint,
}

impl ModuleLint {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: &ConfiguredLint) -> Box<dyn ArchitectureLintRule + Send> {
        match config {
            ConfiguredLint::Module(module_config) => {
                // Simply clone the original configuration
                Box::new(Self {
                    config: module_config.clone(),
                })
            }
            _ => panic!("Expected a Module lint configuration"),
        }
    }

    // Method to check if a module_lint path matches our configured module_lint patterns
    fn matches_module(&self, module_path: &str) -> bool {
        Self::evaluate_module_match(&self.config.matches, module_path)
    }

    // Helper method to evaluate a ModuleMatch against a module_lint path
    fn evaluate_module_match(module_match: &ModuleMatch, module_path: &str) -> bool {
        match module_match {
            ModuleMatch::Module(pattern) => {
                // Try to compile the pattern as a regex and match against module_lint path
                match Regex::new(pattern) {
                    Ok(regex) => regex.is_match(module_path),
                    Err(_) => {
                        // Log error and return false for invalid regex
                        eprintln!("Invalid regex pattern: {}", pattern);
                        false
                    }
                }
            }
            ModuleMatch::AndMatches(left, right) => {
                Self::evaluate_module_match(left, module_path)
                    && Self::evaluate_module_match(right, module_path)
            }
            ModuleMatch::OrMatches(left, right) => {
                Self::evaluate_module_match(left, module_path)
                    || Self::evaluate_module_match(right, module_path)
            }
            ModuleMatch::NotMatch(inner) => !Self::evaluate_module_match(inner, module_path),
        }
    }

    // Helper method to determine if a string matches a pattern (using regex only)
    fn string_matches_pattern(&self, string: &str, pattern: &str) -> bool {
        match Regex::new(pattern) {
            Ok(regex) => regex.is_match(string),
            Err(_) => {
                // Log error and return false for invalid regex
                eprintln!("Invalid regex pattern: {}", pattern);
                false
            }
        }
    }

    // Helper method to check for wildcard imports in a module
    fn check_for_wildcard_imports(
        &self,
        ctx: &LateContext<'_>,
        module: &rustc_hir::Mod,
        severity: Severity,
    ) {
        for &item_id in module.item_ids.iter() {
            let def_id = item_id.owner_id.to_def_id();
            if let Some(local_def_id) = def_id.as_local() {
                let item = ctx.tcx.hir_node_by_def_id(local_def_id).expect_item();

                // If this is a wildcard import, report it
                if let ItemKind::Use(_, UseKind::Glob) = &item.kind {
                    span_lint_and_help(
                        ctx,
                        MODULE_WILDCARD_IMPORT::get_by_severity(severity),
                        self.name().as_str(),
                        item.span,
                        "Wildcard imports are not allowed",
                        None,
                        "Import specific items instead of using a wildcard",
                    );
                }
            }
        }
    }

    // Helper function to check if an item should be disallowed in an "empty" module context
    fn is_disallowed_in_empty_module(&self, item_kind: &ItemKind<'_>) -> bool {
        match item_kind {
            // These items are not allowed in "empty" modules
            ItemKind::Static(..)
            | ItemKind::Struct(..)
            | ItemKind::Union(..)
            | ItemKind::Trait(..)
            | ItemKind::Fn { .. }
            | ItemKind::Const(..)
            | ItemKind::Enum(..) => true,
            ItemKind::Impl(impl_data) if impl_data.of_trait.is_none() => true,
            // Everything else is allowed (re-exports, module declarations, etc.)
            _ => false,
        }
    }

    // Helper to check if a file is a mod.rs file based on its path
    fn is_mod_rs_file(&self, ctx: &LateContext<'_>, span: &rustc_span::Span) -> bool {
        let filename = ctx.sess().source_map().span_to_filename(*span);
        if let rustc_span::FileName::Real(filename) = filename {
            let filename_str =
                filename.to_string_lossy(rustc_span::FileNameDisplayPreference::Local);
            filename_str.ends_with("/mod.rs")
        } else {
            false
        }
    }

    // Helper function to check for disallowed items in a module and call the callback when found
    fn check_for_disallowed_items<C>(
        &self,
        ctx: &LateContext<'_>,
        module_data: &rustc_hir::Mod<'_>,
        on_disallowed_item: C,
    ) where
        C: Fn(&Self, &LateContext<'_>, &rustc_hir::Item<'_>, &str, bool),
    {
        for &item_id in module_data.item_ids.iter() {
            let nested_item = ctx.tcx.hir_item(item_id);

            // Skip if the item is allowed in empty modules
            if !self.is_disallowed_in_empty_module(&nested_item.kind) {
                continue;
            }

            // Get item name from HIR for error messages
            let def_id = nested_item.owner_id.to_def_id();
            let item_name = if let Some(name) = ctx.tcx.opt_item_name(def_id) {
                name.to_ident_string()
            } else {
                // Fallback for items without names (like impl blocks)
                format!("<unnamed item at {:?}>", nested_item.span)
            };

            // Check if this is in a mod.rs file (pass to callback so it can decide what to do)
            let is_mod_rs = self.is_mod_rs_file(ctx, &nested_item.span);

            // Call the callback to handle the disallowed item, passing only necessary context
            on_disallowed_item(self, ctx, nested_item, &item_name, is_mod_rs);
        }
    }
}

// Define specific lints for different rule types
declare_variable_severity_lint!(
    pub,
    MODULE_MUST_BE_NAMED,
    MODULE_MUST_BE_NAMED_LINT_DENY,
    MODULE_MUST_BE_NAMED_LINT_WARN,
    "Module must match a specific naming pattern"
);

declare_variable_severity_lint!(
    pub,
    MODULE_MUST_NOT_BE_NAMED,
    MODULE_MUST_NOT_BE_NAMED_LINT_DENY,
    MODULE_MUST_NOT_BE_NAMED_LINT_WARN,
    "Module must not match a specific naming pattern"
);

declare_variable_severity_lint!(
    pub,
    MODULE_MUST_NOT_BE_EMPTY,
    MODULE_MUST_NOT_BE_EMPTY_LINT_DENY,
    MODULE_MUST_NOT_BE_EMPTY_LINT_WARN,
    "Module must not be empty"
);

declare_variable_severity_lint!(
    pub,
    MODULE_RESTRICT_IMPORTS,
    MODULE_RESTRICT_IMPORTS_LINT_DENY,
    MODULE_RESTRICT_IMPORTS_LINT_WARN,
    "Module has import restrictions"
);

declare_variable_severity_lint!(
    pub,
    MODULE_WILDCARD_IMPORT,
    MODULE_WILDCARD_IMPORT_LINT_DENY,
    MODULE_WILDCARD_IMPORT_LINT_WARN,
    "Wildcard imports are not allowed"
);

// Define specific lints for denied item types
declare_variable_severity_lint!(
    pub,
    MODULE_DENIED_ITEMS,
    MODULE_DENIED_ITEMS_LINT_DENY,
    MODULE_DENIED_ITEMS_LINT_WARN,
    "Module contains denied item types"
);

declare_variable_severity_lint!(
    pub,
    MODULE_MUST_BE_EMPTY,
    MODULE_MUST_BE_EMPTY_LINT_DENY,
    MODULE_MUST_BE_EMPTY_LINT_WARN,
    "Module must be empty"
);

declare_variable_severity_lint!(
    pub,
    MODULE_MUST_HAVE_EMPTY_MOD_FILE,
    MODULE_MUST_HAVE_EMPTY_MOD_FILE_LINT_DENY,
    MODULE_MUST_HAVE_EMPTY_MOD_FILE_LINT_WARN,
    "Module's mod.rs file must be empty (only allowed to re-export other modules)"
);

impl_lint_pass!(ModuleLint => [
    MODULE_MUST_BE_NAMED_LINT_DENY, MODULE_MUST_BE_NAMED_LINT_WARN,
    MODULE_MUST_NOT_BE_NAMED_LINT_DENY, MODULE_MUST_NOT_BE_NAMED_LINT_WARN,
    MODULE_MUST_NOT_BE_EMPTY_LINT_DENY, MODULE_MUST_NOT_BE_EMPTY_LINT_WARN,
    MODULE_MUST_BE_EMPTY_LINT_DENY, MODULE_MUST_BE_EMPTY_LINT_WARN,
    MODULE_MUST_HAVE_EMPTY_MOD_FILE_LINT_DENY, MODULE_MUST_HAVE_EMPTY_MOD_FILE_LINT_WARN,
    MODULE_RESTRICT_IMPORTS_LINT_DENY, MODULE_RESTRICT_IMPORTS_LINT_WARN,
    MODULE_WILDCARD_IMPORT_LINT_DENY, MODULE_WILDCARD_IMPORT_LINT_WARN,
    MODULE_DENIED_ITEMS_LINT_DENY, MODULE_DENIED_ITEMS_LINT_WARN
]);

impl ArchitectureLintRule for ModuleLint {
    fn name(&self) -> String {
        self.config.name.clone()
    }

    fn applies_to_module(&self, namespace: &str) -> bool {
        self.matches_module(namespace)
    }

    fn applies_to_trait(&self, _trait_path: &str) -> bool {
        false // Module lints don't apply to traits
    }

    fn register_late_pass(&self, lint_store: &mut LintStore) {
        let config_clone = self.config.clone();

        lint_store.register_late_pass(move |_| {
            // Create a new instance of ModuleLint to be used as LateLintPass
            Box::new(ModuleLint {
                config: config_clone.clone()
            })
        });
    }
}

impl<'tcx> LateLintPass<'tcx> for ModuleLint {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let parent_item = ctx.tcx.hir_get_parent_item(item.hir_id());
        let parent_module_path = get_full_module_name(&ctx.tcx, &parent_item);

        // Check if the parent module matches our patterns
        if !self.matches_module(&parent_module_path) {
            // Get the full path of the current item for module-specific rules
            if let ItemKind::Mod(_, _) = item.kind {
                let full_item_path = get_full_module_name(&ctx.tcx, &item.owner_id);
                // If neither the parent nor the full item path match, return
                if !self.matches_module(&full_item_path) {
                    return;
                }
            } else {
                return;
            }
        }

        // Apply each rule
        for rule in &self.config.rules {
            match rule {
                ModuleRule::MustBeNamed(pattern, severity) => {
                    if let ItemKind::Mod(_, _) = item.kind {
                        let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id());
                        let item_name_str = item_name.to_string();

                        // Check if module name matches the pattern
                        if !self.string_matches_pattern(&item_name_str, &pattern) {
                            let message = format!(
                                "Module must match pattern '{}', found '{}'",
                                pattern, item_name_str
                            );

                            span_lint_and_help(
                                ctx,
                                MODULE_MUST_BE_NAMED::get_by_severity(*severity),
                                self.name().as_str(),
                                item.span,
                                message,
                                None,
                                format!("Rename this module to match the pattern '{}'", pattern),
                            );
                        }
                    }
                }
                ModuleRule::MustNotBeNamed(pattern, severity) => {
                    if let ItemKind::Mod(_, _) = item.kind {
                        let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id());
                        let item_name_str = item_name.to_string();

                        // Check if module name matches the pattern (which it shouldn't)
                        if self.string_matches_pattern(&item_name_str, pattern) {
                            let message =
                                format!("Module must not match pattern '{}'", pattern);

                            span_lint_and_help(
                                ctx,
                                MODULE_MUST_NOT_BE_NAMED::get_by_severity(*severity),
                                self.name().as_str(),
                                item.span,
                                message,
                                None,
                                "Choose a name that doesn't match this pattern",
                            );
                        }
                    }
                }
                ModuleRule::MustNotBeEmpty(severity) => {
                    if let ItemKind::Mod(_, module_data) = item.kind {
                        if module_data.item_ids.is_empty() {
                            span_lint_and_help(
                                ctx,
                                MODULE_MUST_NOT_BE_EMPTY::get_by_severity(*severity),
                                self.name().as_str(),
                                item.span,
                                "Module must not be empty",
                                None,
                                "Add content to this module or remove it",
                            );
                        }
                    }
                }
                ModuleRule::MustBeEmpty(severity) => {
                    if let ItemKind::Mod(_, module_data) = item.kind {
                        let sev = severity; // Use severity in closure
                        self.check_for_disallowed_items(
                            ctx,
                            module_data,
                            |slf, ctx, item, item_name, _is_mod_rs| {
                                // For MustBeEmpty, we don't care if it's a mod.rs file or not
                                span_lint_and_help(
                                    ctx,
                                    MODULE_MUST_BE_EMPTY::get_by_severity(*sev),
                                    slf.name().as_str(),
                                    item.span,
                                    format!("Item '{}' not allowed in empty module", item_name),
                                    None,
                                    "Remove this item from the module, which must be empty",
                                );
                            },
                        );
                    }
                }
                ModuleRule::MustHaveEmptyModFile(severity) => {
                    if let ItemKind::Mod(_, module_data) = item.kind {
                        let sev = severity; // Use severity in closure
                        self.check_for_disallowed_items(
                            ctx,
                            module_data,
                            |slf, ctx, item, item_name, is_mod_rs| {
                                // Only emit the lint if this is in a mod.rs file
                                if is_mod_rs {
                                    span_lint_and_help(
                                        ctx,
                                        MODULE_MUST_HAVE_EMPTY_MOD_FILE::get_by_severity(*sev),
                                        slf.name().as_str(),
                                        item.span,
                                        format!("Item '{}' disallowed in mod.rs due to empty-mod-file policy", item_name),
                                        None,
                                        "Remove this item from the mod.rs file or move it to a submodule"
                                    );
                                }
                            }
                        );
                    }
                }
                ModuleRule::RestrictImports { allowed_only, denied, severity } => {
                    if let ItemKind::Use(path, _) = &item.kind {
                        let import_path: Vec<_> = path
                            .segments
                            .iter()
                            .map(|s| s.ident.as_str().to_string())
                            .collect();
                        let import_module = import_path.join("::");

                        // Check allowed imports if specified
                        if let Some(allowed) = allowed_only {
                            let is_allowed =
                                allowed.iter().any(|pattern| match Regex::new(pattern) {
                                    Ok(re) => re.is_match(&import_module),
                                    Err(_) => {
                                        eprintln!("Invalid regex pattern: {}", pattern);
                                        false
                                    }
                                });

                            if !is_allowed {
                                let message = format!(
                                    "Use of module '{}' is not allowed; only {:?} are permitted",
                                    import_module, allowed
                                );

                                span_lint_and_help(
                                    ctx,
                                    MODULE_RESTRICT_IMPORTS::get_by_severity(*severity),
                                    self.name().as_str(),
                                    item.span,
                                    message,
                                    None,
                                    "Use only allowed module imports",
                                );
                            }
                        }

                        // Check denied imports if specified
                        if let Some(denied_list) = denied {
                            let is_denied =
                                denied_list.iter().any(|pattern| match Regex::new(pattern) {
                                    Ok(re) => re.is_match(&import_module),
                                    Err(_) => {
                                        eprintln!("Invalid regex pattern: {}", pattern);
                                        false
                                    }
                                });

                            if is_denied {
                                let message =
                                    format!("Use of module '{}' is denied", import_module);

                                span_lint_and_help(
                                    ctx,
                                    MODULE_RESTRICT_IMPORTS::get_by_severity(*severity),
                                    self.name().as_str(),
                                    item.span,
                                    message,
                                    None,
                                    "Remove this import",
                                );
                            }
                        }
                    }
                }
                ModuleRule::NoWildcardImports(severity) => {
                    // Check if the current item is a wildcard import
                    if let ItemKind::Use(_, UseKind::Glob) = &item.kind {
                        span_lint_and_help(
                            ctx,
                            MODULE_WILDCARD_IMPORT::get_by_severity(*severity),
                            self.name().as_str(),
                            item.span,
                            "Wildcard imports are not allowed",
                            None,
                            "Import specific items instead of using a wildcard",
                        );
                    }

                    // Also check nested modules for wildcard imports
                    if let ItemKind::Mod(_, module) = &item.kind {
                        self.check_for_wildcard_imports(ctx, module, *severity);
                    }
                }
                ModuleRule::DeniedItems { items, severity } => {
                    // Get the item type as a string
                    let item_type = match &item.kind {
                        ItemKind::Enum(..) => "enum",
                        ItemKind::Struct(..) => "struct",
                        ItemKind::Trait(..) => "trait",
                        ItemKind::Impl(..) => "impl",
                        ItemKind::Fn { .. } => "function",
                        ItemKind::Mod(..) => "module",
                        ItemKind::Static(..) => "static",
                        ItemKind::Const(..) => "const",
                        ItemKind::Union(..) => "union",
                        _ => "",
                    };

                    // Check if the current item type is in the denied list
                    if !item_type.is_empty() && items.contains(&item_type.to_string()) {
                        let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id());
                        span_lint_and_help(
                            ctx,
                            MODULE_DENIED_ITEMS::get_by_severity(*severity),
                            self.name().as_str(),
                            item.span,
                            format!(
                                "{} '{}' is not allowed in this module",
                                item_type, item_name
                            ),
                            None,
                            "Consider moving this item to a different module",
                        );
                    }
                }
                // Skip logical combinations for now (And, Or, Not)
                ModuleRule::And(_, _) | ModuleRule::Or(_, _) | ModuleRule::Not(_) => {}
            }
        }
    }
}
