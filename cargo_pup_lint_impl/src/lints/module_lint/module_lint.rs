use crate::ArchitectureLintRule;
use crate::declare_variable_severity_lint_new;
use crate::helpers::clippy_utils::span_lint_and_help;
use crate::helpers::queries::get_full_module_name;
use cargo_pup_lint_config::{ConfiguredLint, ModuleMatch, ModuleRule, Severity};
use regex::Regex;
use rustc_hir::{Item, ItemKind, UseKind};
use rustc_lint::{LateContext, LateLintPass, LintContext, LintStore};
use rustc_session::impl_lint_pass;

pub struct ModuleLint {
    name: String,
    matches: ModuleMatch,
    // Store minimal data needed instead of cloning ConfiguredLint
    // We'll store just what we need from the rules
    module_rules: Vec<ModuleRuleInfo>,
}

// A simplified representation of rules we need to store
#[derive(Clone)]
struct ModuleRuleInfo {
    rule_type: ModuleRuleType,
    severity: Severity,
}

// Types of rules we handle
#[derive(Clone)]
enum ModuleRuleType {
    MustBeNamed(String),
    MustNotBeNamed(String),
    MustNotBeEmpty,
    MustBeEmpty,
    MustHaveEmptyModFile,
    RestrictImports {
        allowed_only: Option<Vec<String>>,
        denied: Option<Vec<String>>,
    },
    NoWildcardImports,
    DeniedItems(Vec<String>),
}

impl ModuleLint {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: &ConfiguredLint) -> Box<dyn ArchitectureLintRule + Send> {
        // TODO - just clone this
        match config {
            ConfiguredLint::Module(m) => {
                // Extract rule information to our simplified structure
                let module_rules = m
                    .rules
                    .iter()
                    .filter_map(|rule| {
                        match rule {
                            ModuleRule::MustBeNamed(name, severity) => Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::MustBeNamed(name.clone()),
                                severity: *severity,
                            }),
                            ModuleRule::MustNotBeNamed(name, severity) => Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::MustNotBeNamed(name.clone()),
                                severity: *severity,
                            }),
                            ModuleRule::MustNotBeEmpty(severity) => Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::MustNotBeEmpty,
                                severity: *severity,
                            }),
                            ModuleRule::MustBeEmpty(severity) => Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::MustBeEmpty,
                                severity: *severity,
                            }),
                            ModuleRule::MustHaveEmptyModFile(severity) => Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::MustHaveEmptyModFile,
                                severity: *severity,
                            }),
                            ModuleRule::RestrictImports {
                                allowed_only,
                                denied,
                                severity,
                            } => {
                                // Clone the string vectors inside allowed_only and denied
                                let allowed_clone = allowed_only.clone();
                                let denied_clone = denied.clone();

                                Some(ModuleRuleInfo {
                                    rule_type: ModuleRuleType::RestrictImports {
                                        allowed_only: allowed_clone,
                                        denied: denied_clone,
                                    },
                                    severity: *severity,
                                })
                            }
                            ModuleRule::NoWildcardImports(severity) => Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::NoWildcardImports,
                                severity: *severity,
                            }),
                            // Handle the DeniedItems variant
                            ModuleRule::DeniedItems { items, severity } => Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::DeniedItems(items.clone()),
                                severity: *severity,
                            }),
                            // Not handling logical combinations for now
                            ModuleRule::And(_, _) => None,
                            ModuleRule::Or(_, _) => None,
                            ModuleRule::Not(_) => None,
                        }
                    })
                    .collect();

                Box::new(Self {
                    name: m.name.clone(),
                    matches: m.matches.clone(),
                    module_rules,
                })
            }
            _ => panic!("Expected a Module lint configuration"),
        }
    }

    // Method to check if a module_lint path matches our configured module_lint patterns
    fn matches_module(&self, module_path: &str) -> bool {
        Self::evaluate_module_match(&self.matches, module_path)
    }

    // Helper method to evaluate a ModuleMatch against a module_lint path
    fn evaluate_module_match(module_match: &ModuleMatch, module_path: &str) -> bool {
        match module_match {
            ModuleMatch::Module(pattern) => {
                // Try to compile the pattern as a regex and match against module_lint path
                match Regex::new(pattern) {
                    Ok(regex) => regex.is_match(module_path),
                    Err(_) => {
                        // If not a valid regex, fall back to direct string comparison
                        pattern == module_path
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

    // Helper method to determine if a string matches a pattern (using regex if possible)
    fn string_matches_pattern(&self, string: &str, pattern: &str) -> bool {
        match Regex::new(pattern) {
            Ok(regex) => regex.is_match(string),
            Err(_) => string == pattern, // Fall back to exact match
        }
    }

    // Helper method to get a user-friendly description of a pattern
    fn describe_pattern(&self, pattern: &str) -> &'static str {
        if pattern.contains(|c: char| {
            c == '*' || c == '.' || c == '+' || c == '[' || c == '(' || c == '|'
        }) {
            "pattern"
        } else {
            "name"
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
                let item = ctx.tcx.hir().expect_item(local_def_id);

                // If this is a wildcard import, report it
                if let ItemKind::Use(_, UseKind::Glob) = &item.kind {
                    span_lint_and_help(
                        ctx,
                        ModuleWildcardImportLint::get_by_severity(severity),
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
            let hir = ctx.tcx.hir();
            let item_name = hir.name(nested_item.hir_id()).to_ident_string();

            // Check if this is in a mod.rs file (pass to callback so it can decide what to do)
            let is_mod_rs = self.is_mod_rs_file(ctx, &nested_item.span);

            // Call the callback to handle the disallowed item, passing only necessary context
            on_disallowed_item(self, ctx, nested_item, &item_name, is_mod_rs);
        }
    }
}

// Define specific lints for different rule types
declare_variable_severity_lint_new!(
    pub,
    ModuleMustBeNamedLint,
    MODULE_MUST_BE_NAMED_LINT_DENY,
    MODULE_MUST_BE_NAMED_LINT_WARN,
    "Module must match a specific naming pattern"
);

declare_variable_severity_lint_new!(
    pub,
    ModuleMustNotBeNamedLint,
    MODULE_MUST_NOT_BE_NAMED_LINT_DENY,
    MODULE_MUST_NOT_BE_NAMED_LINT_WARN,
    "Module must not match a specific naming pattern"
);

declare_variable_severity_lint_new!(
    pub,
    ModuleMustNotBeEmptyLint,
    MODULE_MUST_NOT_BE_EMPTY_LINT_DENY,
    MODULE_MUST_NOT_BE_EMPTY_LINT_WARN,
    "Module must not be empty"
);

declare_variable_severity_lint_new!(
    pub,
    ModuleRestrictImportsLint,
    MODULE_RESTRICT_IMPORTS_LINT_DENY,
    MODULE_RESTRICT_IMPORTS_LINT_WARN,
    "Module has import restrictions"
);

declare_variable_severity_lint_new!(
    pub,
    ModuleWildcardImportLint,
    MODULE_WILDCARD_IMPORT_LINT_DENY,
    MODULE_WILDCARD_IMPORT_LINT_WARN,
    "Wildcard imports are not allowed"
);

// Define specific lints for denied item types
declare_variable_severity_lint_new!(
    pub,
    ModuleDeniedItemsLint,
    MODULE_DENIED_ITEMS_LINT_DENY,
    MODULE_DENIED_ITEMS_LINT_WARN,
    "Module contains denied item types"
);

declare_variable_severity_lint_new!(
    pub,
    ModuleMustBeEmptyLint,
    MODULE_MUST_BE_EMPTY_LINT_DENY,
    MODULE_MUST_BE_EMPTY_LINT_WARN,
    "Module must be empty"
);

declare_variable_severity_lint_new!(
    pub,
    ModuleMustHaveEmptyModFileLint,
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
        self.name.clone()
    }

    fn applies_to_module(&self, namespace: &str) -> bool {
        self.matches_module(namespace)
    }

    fn applies_to_trait(&self, _trait_path: &str) -> bool {
        false // Module lints don't apply to traits by default
    }

    fn register_late_pass(&self, lint_store: &mut LintStore) {
        let name = self.name.clone();
        let matches = self.matches.clone();
        let module_rules = self.module_rules.clone();

        lint_store.register_late_pass(move |_| {
            // Create a new instance of ModuleLint to be used as LateLintPass
            Box::new(ModuleLint {
                name: name.clone(),
                matches: matches.clone(),
                module_rules: module_rules.clone(),
            })
        });
    }
}

impl<'tcx> LateLintPass<'tcx> for ModuleLint {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let parent_item = ctx.tcx.hir_get_parent_item(item.hir_id());
        let parent_module_path = get_full_module_name(&ctx.tcx, &parent_item);

        // Check if the parent module matches our patterns (original behavior)
        if !self.matches_module(&parent_module_path) {
            // Get the full path of the current item for module-specific rules
            if let ItemKind::Mod(_) = item.kind {
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
        for rule_info in &self.module_rules {
            match &rule_info.rule_type {
                ModuleRuleType::MustBeNamed(pattern) => {
                    if let ItemKind::Mod(_) = item.kind {
                        let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id());
                        let item_name_str = item_name.to_string();

                        // Check if module_lint name matches the pattern
                        if !self.string_matches_pattern(&item_name_str, pattern) {
                            let pattern_type = self.describe_pattern(pattern);
                            let message = format!(
                                "Module must match {} '{}', found '{}'",
                                pattern_type, pattern, item_name_str
                            );

                            let help = if pattern_type == "pattern" {
                                format!("Rename this module to match the pattern '{}'", pattern)
                            } else {
                                format!("Rename this module to '{}'", pattern)
                            };

                            span_lint_and_help(
                                ctx,
                                ModuleMustBeNamedLint::get_by_severity(rule_info.severity),
                                self.name().as_str(),
                                item.span,
                                message,
                                None,
                                help,
                            );
                        }
                    }
                }
                ModuleRuleType::MustNotBeNamed(pattern) => {
                    if let ItemKind::Mod(_) = item.kind {
                        let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id());
                        let item_name_str = item_name.to_string();

                        // Check if module_lint name matches the pattern (which it shouldn't)
                        if self.string_matches_pattern(&item_name_str, pattern) {
                            let pattern_type = self.describe_pattern(pattern);
                            let message =
                                format!("Module must not match {} '{}'", pattern_type, pattern);

                            let help = if pattern_type == "pattern" {
                                "Choose a name that doesn't match this pattern"
                            } else {
                                "Choose a different name for this module"
                            };

                            span_lint_and_help(
                                ctx,
                                ModuleMustNotBeNamedLint::get_by_severity(rule_info.severity),
                                self.name().as_str(),
                                item.span,
                                message,
                                None,
                                help,
                            );
                        }
                    }
                }
                ModuleRuleType::MustNotBeEmpty => {
                    if let ItemKind::Mod(module_data) = item.kind {
                        if module_data.item_ids.is_empty() {
                            span_lint_and_help(
                                ctx,
                                ModuleMustNotBeEmptyLint::get_by_severity(rule_info.severity),
                                self.name().as_str(),
                                item.span,
                                "Module must not be empty",
                                None,
                                "Add content to this module or remove it",
                            );
                        }
                    }
                }
                ModuleRuleType::MustBeEmpty => {
                    if let ItemKind::Mod(module_data) = item.kind {
                        let severity = rule_info.severity;
                        self.check_for_disallowed_items(
                            ctx,
                            module_data,
                            |slf, ctx, item, item_name, _is_mod_rs| {
                                // For MustBeEmpty, we don't care if it's a mod.rs file or not
                                span_lint_and_help(
                                    ctx,
                                    ModuleMustBeEmptyLint::get_by_severity(severity),
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
                ModuleRuleType::MustHaveEmptyModFile => {
                    if let ItemKind::Mod(module_data) = item.kind {
                        let severity = rule_info.severity;
                        self.check_for_disallowed_items(
                            ctx,
                            module_data,
                            |slf, ctx, item, item_name, is_mod_rs| {
                                // Only emit the lint if this is in a mod.rs file
                                if is_mod_rs {
                                    span_lint_and_help(
                                        ctx,
                                        ModuleMustHaveEmptyModFileLint::get_by_severity(severity),
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
                ModuleRuleType::RestrictImports {
                    allowed_only,
                    denied,
                } => {
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
                                    Err(_) => import_module.starts_with(pattern),
                                });

                            if !is_allowed {
                                let message = format!(
                                    "Use of module '{}' is not allowed; only {:?} are permitted",
                                    import_module, allowed
                                );

                                span_lint_and_help(
                                    ctx,
                                    ModuleRestrictImportsLint::get_by_severity(rule_info.severity),
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
                                    Err(_) => import_module.starts_with(pattern),
                                });

                            if is_denied {
                                let message =
                                    format!("Use of module '{}' is denied", import_module);

                                span_lint_and_help(
                                    ctx,
                                    ModuleRestrictImportsLint::get_by_severity(rule_info.severity),
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
                ModuleRuleType::NoWildcardImports => {
                    // Check if the current item is a wildcard import
                    if let ItemKind::Use(_, UseKind::Glob) = &item.kind {
                        span_lint_and_help(
                            ctx,
                            ModuleWildcardImportLint::get_by_severity(rule_info.severity),
                            self.name().as_str(),
                            item.span,
                            "Wildcard imports are not allowed",
                            None,
                            "Import specific items instead of using a wildcard",
                        );
                    }

                    // Also check nested modules for wildcard imports
                    if let ItemKind::Mod(module) = &item.kind {
                        self.check_for_wildcard_imports(ctx, module, rule_info.severity);
                    }
                }
                ModuleRuleType::DeniedItems(denied_items) => {
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
                    if !item_type.is_empty() && denied_items.contains(&item_type.to_string()) {
                        let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id());
                        span_lint_and_help(
                            ctx,
                            ModuleDeniedItemsLint::get_by_severity(rule_info.severity),
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
            }
        }
    }
}
