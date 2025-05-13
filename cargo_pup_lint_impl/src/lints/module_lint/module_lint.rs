use rustc_lint::{LateContext, LateLintPass, Lint, LintStore};
use rustc_hir::{Item, ItemKind, OwnerId, UseKind};
use rustc_middle::ty::TyCtxt;
use rustc_session::impl_lint_pass;
use regex::Regex;
use cargo_pup_lint_config::{ConfiguredLint, Severity, ModuleMatch, ModuleRule};
use crate::ArchitectureLintRule;
use crate::helpers::queries::get_full_module_name;
use crate::helpers::clippy_utils::span_lint_and_help;
use crate::declare_variable_severity_lint;

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
    RestrictImports {
        allowed_only: Option<Vec<String>>,
        denied: Option<Vec<String>>,
    },
    NoWildcardImports,
}

impl ModuleLint {
    pub fn new(config: &ConfiguredLint) -> Box<dyn ArchitectureLintRule + Send> {
        match config {
            ConfiguredLint::Module(m) => {
                // Extract rule information to our simplified structure
                let module_rules = m.rules.iter().filter_map(|rule| {
                    match rule {
                        ModuleRule::MustBeNamed(name, severity) => 
                            Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::MustBeNamed(name.clone()),
                                severity: *severity
                            }),
                        ModuleRule::MustNotBeNamed(name, severity) => 
                            Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::MustNotBeNamed(name.clone()),
                                severity: *severity
                            }),
                        ModuleRule::MustNotBeEmpty(severity) => 
                            Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::MustNotBeEmpty,
                                severity: *severity
                            }),
                        ModuleRule::RestrictImports { allowed_only, denied, severity } => {
                            // Clone the string vectors inside allowed_only and denied
                            let allowed_clone = allowed_only.as_ref().map(|v| v.clone());
                            let denied_clone = denied.as_ref().map(|v| v.clone());
                            
                            Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::RestrictImports { 
                                    allowed_only: allowed_clone, 
                                    denied: denied_clone 
                                },
                                severity: *severity
                            })
                        },
                        ModuleRule::NoWildcardImports(severity) => 
                            Some(ModuleRuleInfo {
                                rule_type: ModuleRuleType::NoWildcardImports,
                                severity: *severity
                            }),
                        // Not handling logical combinations for now
                        ModuleRule::And(_, _) => None,
                        ModuleRule::Or(_, _) => None,
                        ModuleRule::Not(_) => None,
                    }
                }).collect();
                
                Box::new(Self {
                    name: m.name.clone(),
                    matches: m.matches.clone(),
                    module_rules,
                })
            },
            _ => panic!("Expected a Module lint configuration")
        }
    }
    
    // Method to check if a module_lint path matches our configured module_lint patterns
    fn matches_module(&self, module_path: &str) -> bool {
        self.evaluate_module_match(&self.matches, module_path)
    }
    
    // Helper method to evaluate a ModuleMatch against a module_lint path
    fn evaluate_module_match(&self, module_match: &ModuleMatch, module_path: &str) -> bool {
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
            },
            ModuleMatch::AndMatches(left, right) => {
                self.evaluate_module_match(left, module_path) && 
                self.evaluate_module_match(right, module_path)
            },
            ModuleMatch::OrMatches(left, right) => {
                self.evaluate_module_match(left, module_path) || 
                self.evaluate_module_match(right, module_path)
            },
            ModuleMatch::NotMatch(inner) => {
                !self.evaluate_module_match(inner, module_path)
            }
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
        if pattern.contains(|c: char| c == '*' || c == '.' || c == '+' || c == '[' || c == '(' || c == '|') {
            "pattern"
        } else {
            "name"
        }
    }
}

// Declare the module_lint lint with variable severity
declare_variable_severity_lint!(
    pub,
    MODULE_LINT,
    MODULE_LINT_DENY, 
    MODULE_LINT_WARN,
    "Module structure and organization rules"
);

impl_lint_pass!(ModuleLint => [MODULE_LINT_DENY, MODULE_LINT_WARN]);

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
        // Get the full module_lint path
        let parent_item = ctx.tcx.hir_get_parent_item(item.hir_id());
        let module_path = get_full_module_name(&ctx.tcx, &parent_item);
        
        // Check if this module_lint matches our patterns
        if !self.matches_module(&module_path) {
            return;
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
                            let message = format!("Module must match {} '{}', found '{}'", 
                                                pattern_type, pattern, item_name_str);
                            
                            let help = if pattern_type == "pattern" {
                                format!("Rename this module_lint to match the pattern '{}'", pattern)
                            } else {
                                format!("Rename this module_lint to '{}'", pattern)
                            };
                            
                            span_lint_and_help(
                                ctx,
                                get_lint(rule_info.severity),
                                self.name().as_str(),
                                item.span,
                                message,
                                None,
                                help,
                            );
                        }
                    }
                },
                ModuleRuleType::MustNotBeNamed(pattern) => {
                    if let ItemKind::Mod(_) = item.kind {
                        let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id());
                        let item_name_str = item_name.to_string();
                        
                        // Check if module_lint name matches the pattern (which it shouldn't)
                        if self.string_matches_pattern(&item_name_str, pattern) {
                            let pattern_type = self.describe_pattern(pattern);
                            let message = format!("Module must not match {} '{}'", 
                                                pattern_type, pattern);
                            
                            let help = if pattern_type == "pattern" {
                                "Choose a name that doesn't match this pattern"
                            } else {
                                "Choose a different name for this module_lint"
                            };
                            
                            span_lint_and_help(
                                ctx,
                                get_lint(rule_info.severity),
                                self.name().as_str(),
                                item.span,
                                message,
                                None,
                                help,
                            );
                        }
                    }
                },
                ModuleRuleType::MustNotBeEmpty => {
                    if let ItemKind::Mod(module_data) = item.kind {
                        if module_data.item_ids.is_empty() {
                            span_lint_and_help(
                                ctx,
                                get_lint(rule_info.severity),
                                self.name().as_str(),
                                item.span,
                                "Module must not be empty",
                                None,
                                "Add content to this module_lint or remove it",
                            );
                        }
                    }
                },
                ModuleRuleType::RestrictImports { allowed_only, denied } => {
                    if let ItemKind::Use(path, _) = &item.kind {
                        let import_path: Vec<_> = path
                            .segments
                            .iter()
                            .map(|s| s.ident.as_str().to_string())
                            .collect();
                        let import_module = import_path.join("::");
                        
                        // Check allowed imports if specified
                        if let Some(allowed) = allowed_only {
                            let is_allowed = allowed.iter().any(|pattern| {
                                match Regex::new(pattern) {
                                    Ok(re) => re.is_match(&import_module),
                                    Err(_) => import_module.starts_with(pattern),
                                }
                            });
                            
                            if !is_allowed {
                                let message = format!("Use of module '{}' is not allowed; only {:?} are permitted",
                                        import_module, allowed);
                                
                                span_lint_and_help(
                                    ctx,
                                    get_lint(rule_info.severity),
                                    self.name().as_str(),
                                    item.span,
                                    message,
                                    None,
                                    "Use only allowed module_lint imports",
                                );
                            }
                        }
                        
                        // Check denied imports if specified
                        if let Some(denied_list) = denied {
                            let is_denied = denied_list.iter().any(|pattern| {
                                match Regex::new(pattern) {
                                    Ok(re) => re.is_match(&import_module),
                                    Err(_) => import_module.starts_with(pattern),
                                }
                            });
                            
                            if is_denied {
                                let message = format!("Use of module_lint '{}' is denied", import_module);
                                
                                span_lint_and_help(
                                    ctx,
                                    get_lint(rule_info.severity),
                                    self.name().as_str(),
                                    item.span,
                                    message,
                                    None,
                                    "Remove this import",
                                );
                            }
                        }
                    }
                },
                ModuleRuleType::NoWildcardImports => {
                    if let ItemKind::Use(_, UseKind::Glob) = &item.kind {
                        span_lint_and_help(
                            ctx,
                            get_lint(rule_info.severity),
                            self.name().as_str(),
                            item.span,
                            "Wildcard imports are not allowed",
                            None,
                            "Import specific items instead of using a wildcard",
                        );
                    }
                },
            }
        }
    }
}