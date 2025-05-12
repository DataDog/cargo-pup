use rustc_lint::{LateContext, LateLintPass, Lint, LintStore};
use rustc_hir::{Item, ItemKind};
use rustc_session::impl_lint_pass;
use regex::Regex;
use cargo_pup_lint_config::{ConfiguredLint, Severity, StructMatch, StructRule};
use crate::ArchitectureLintRule;
use crate::helpers::clippy_utils::span_lint_and_help;
use crate::declare_variable_severity_lint;

pub struct StructLint {
    name: String,
    matches: StructMatch,
    struct_rules: Vec<StructRule>,
}

impl StructLint {
    pub fn new(config: &ConfiguredLint) -> Box<dyn ArchitectureLintRule + Send> {
        if let ConfiguredLint::Struct(s) = config {
            Box::new(Self {
                name: s.name.clone(),
                matches: s.matches.clone(),
                struct_rules: s.rules.iter().map(|r| r.clone()).collect(),
            })
        } else {
            panic!("Expected a Struct lint configuration")
        }
    }
    
    fn string_matches_pattern(&self, string: &str, pattern: &str) -> bool {
        match Regex::new(pattern) {
            Ok(regex) => regex.is_match(string),
            Err(_) => string == pattern,
        }
    }
    
    fn describe_pattern(&self, pattern: &str) -> &'static str {
        if pattern.contains(|c: char| c == '*' || c == '.' || c == '+' || c == '[' || c == '(' || c == '|') {
            "pattern"
        } else {
            "name"
        }
    }
}

// Declare the struct_lint lint with variable severity
declare_variable_severity_lint!(
    pub,
    STRUCT_LINT,
    STRUCT_LINT_DENY, 
    STRUCT_LINT_WARN,
    "Struct naming and attribute rules"
);

impl_lint_pass!(StructLint => [STRUCT_LINT_DENY, STRUCT_LINT_WARN]);

impl ArchitectureLintRule for StructLint {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn applies_to_module(&self, _namespace: &str) -> bool {
        false
    }

    fn applies_to_trait(&self, _trait_path: &str) -> bool {
        false
    }

    fn register_late_pass(&self, lint_store: &mut LintStore) {
        let name = self.name.clone();
        let matches = self.matches.clone();
        let struct_rules = self.struct_rules.clone();
        
        lint_store.register_late_pass(move |_| {
            Box::new(StructLint {
                name: name.clone(),
                matches: matches.clone(),
                struct_rules: struct_rules.clone(),
            })
        });
    }
}

impl<'tcx> LateLintPass<'tcx> for StructLint {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // We only care about struct_lint items
        if let ItemKind::Struct(..) = item.kind {
            let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id()).to_string();
            
            // Simple pattern matching for now
            if let StructMatch::Name(pattern) = &self.matches {
                if !self.string_matches_pattern(&item_name, pattern) {
                    return;
                }
            }
            
            // Apply rules
            for rule in &self.struct_rules {
                match rule {
                    StructRule::MustBeNamed(pattern, severity) => {
                        if !self.string_matches_pattern(&item_name, pattern) {
                            let pattern_type = self.describe_pattern(pattern);
                            let message = format!("Struct must match {} '{}', found '{}'", 
                                               pattern_type, pattern, item_name);
                            
                            let help = if pattern_type == "pattern" {
                                format!("Rename this struct_lint to match the pattern '{}'", pattern)
                            } else {
                                format!("Rename this struct_lint to '{}'", pattern)
                            };
                            
                            span_lint_and_help(
                                ctx,
                                get_lint(*severity),
                                self.name().as_str(),
                                item.span,
                                message,
                                None,
                                help,
                            );
                        }
                    },
                    StructRule::MustNotBeNamed(pattern, severity) => {
                        if self.string_matches_pattern(&item_name, pattern) {
                            let pattern_type = self.describe_pattern(pattern);
                            let message = format!("Struct must not match {} '{}'", 
                                                pattern_type, pattern);
                            
                            let help = if pattern_type == "pattern" {
                                "Choose a name that doesn't match this pattern"
                            } else {
                                "Choose a different name for this struct_lint"
                            };
                            
                            span_lint_and_help(
                                ctx,
                                get_lint(*severity),
                                self.name().as_str(),
                                item.span,
                                message,
                                None,
                                help,
                            );
                        }
                    },
                    _ => {} // Ignore other rule types for now
                }
            }
        }
    }
} 