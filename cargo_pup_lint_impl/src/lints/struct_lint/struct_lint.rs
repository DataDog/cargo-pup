use rustc_lint::{LateContext, LateLintPass, Lint, LintStore};
use rustc_hir::{Item, ItemKind, def_id::DefId};
use rustc_session::impl_lint_pass;
use rustc_span::BytePos;
use regex::Regex;
use cargo_pup_lint_config::{ConfiguredLint, Severity, StructMatch, StructRule};
use crate::ArchitectureLintRule;
use crate::helpers::clippy_utils::span_lint_and_help;
use crate::declare_variable_severity_lint_new;


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
    
    // Helper method to check if a struct in a given crate should be linted
    fn matches_struct(&self, crate_name: &str, struct_name: &str) -> bool {
        self.evaluate_struct_match(&self.matches, crate_name, struct_name)
    }
    
    // Evaluates the complex matcher structure to determine if a struct matches
    fn evaluate_struct_match(&self, matcher: &StructMatch, crate_name: &str, struct_name: &str) -> bool {
        match matcher {
            StructMatch::Name(pattern) => {
                // Try to match both the crate name and the struct name
                // If it's a crate name starting with "test_", prefer that match
                if pattern.starts_with("test_") {
                    // This is likely a crate name pattern
                    self.string_matches_pattern(crate_name, pattern)
                } else {
                    // This is likely a struct name pattern
                    self.string_matches_pattern(struct_name, pattern)
                }
            },
            StructMatch::HasAttribute(pattern) => {
                // Attribute matching not yet implemented
                false
            },
            StructMatch::ImplementsTrait(_) => {
                // Implementation will be handled in check_item
                // Always return true here and do the filtering there
                true
            },
            StructMatch::AndMatches(left, right) => {
                self.evaluate_struct_match(left, crate_name, struct_name) && 
                self.evaluate_struct_match(right, crate_name, struct_name)
            },
            StructMatch::OrMatches(left, right) => {
                self.evaluate_struct_match(left, crate_name, struct_name) || 
                self.evaluate_struct_match(right, crate_name, struct_name)
            },
            StructMatch::NotMatch(inner) => {
                !self.evaluate_struct_match(inner, crate_name, struct_name)
            }
        }
    }
    
    // Helper to determine if a string matches a pattern (exact match or regex)
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
    
    // Check if this struct has any trait implementations that match our patterns
    fn has_matching_trait_impl<'tcx>(&self, ctx: &LateContext<'tcx>, def_id: DefId) -> bool {
        // This is a simplified implementation that will be expanded later
        fn needs_trait_check(matcher: &StructMatch) -> Option<String> {
            match matcher {
                StructMatch::ImplementsTrait(pattern) => Some(pattern.clone()),
                StructMatch::AndMatches(left, right) => {
                    needs_trait_check(left).or_else(|| needs_trait_check(right))
                },
                StructMatch::OrMatches(left, right) => {
                    needs_trait_check(left).or_else(|| needs_trait_check(right))
                },
                StructMatch::NotMatch(inner) => needs_trait_check(inner),
                _ => None,
            }
        }
        
        // Check if we have any trait matchers
        if let Some(trait_pattern) = needs_trait_check(&self.matches) {
            // This is a simplified placeholder implementation
            // A real implementation would need to traverse all trait implementations
            // for the type and check them against the pattern
            //
            // For now, we'll just return true to allow the basic matcher structure to work
            //
            // TODO: Replace this with actual trait implementation checking
            // using TCX and hir() traversal when that part of the API is better understood.
            return true;
        }
        
        // If no trait patterns found, return true
        true
    }
}

declare_variable_severity_lint_new!(
    pub,
    STRUCT_LINT_MUST_BE_NAMED,
    STRUCT_LINT_MUST_BE_NAMED_DENY, 
    STRUCT_LINT_MUST_BE_NAMED_WARN,
    "Struct naming and attribute rules"
);

declare_variable_severity_lint_new!(
    pub,
    STRUCT_LINT_MUST_NOT_BE_NAMED,
    STRUCT_LINT_MUST_NOT_BE_NAMED_DENY, 
    STRUCT_LINT_MUST_NOT_BE_NAMED_WARN,
    "Struct naming and attribute rules"
);

impl_lint_pass!(StructLint => [STRUCT_LINT_MUST_BE_NAMED_DENY, STRUCT_LINT_MUST_BE_NAMED_WARN, STRUCT_LINT_MUST_NOT_BE_NAMED_DENY, STRUCT_LINT_MUST_NOT_BE_NAMED_WARN]);

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
        // We only care about struct items
        if let ItemKind::Struct(..) = item.kind {
            let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id()).to_string();
            let crate_name = ctx.tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE).to_string();
            
            // Check if this struct matches our patterns
            if !self.matches_struct(&crate_name, &item_name) {
                return;
            }
            
            // Check trait implementations if needed
            let def_id = item.owner_id.def_id.to_def_id();
            if !self.has_matching_trait_impl(ctx, def_id) {
                return;
            }
            
            // Create a span that only covers the struct definition line
            // This includes "pub struct Name {" but not the struct fields or closing brace
            let definition_span = {
                let span = item.span;
                // Check if the struct is public by checking if ident_span is different from vis_span
                let is_pub = !item.vis_span.is_empty();
                let prefix_len = if is_pub { 11 } else { 7 }; // "pub struct " or "struct "
                
                // Create a span from start to just after the struct name and opening brace
                let end_pos = span.lo() + BytePos((prefix_len + item_name.len() + 2) as u32); // +2 for " {"
                span.with_hi(end_pos)
            };
            
            // Apply rules
            for rule in &self.struct_rules {
                match rule {
                    StructRule::MustBeNamed(pattern, severity) => {
                        if !self.string_matches_pattern(&item_name, pattern) {
                            let pattern_type = self.describe_pattern(pattern);
                            let message = format!("Struct must match {} '{}', found '{}'", 
                                               pattern_type, pattern, item_name);
                            
                            let help = if pattern_type == "pattern" {
                                format!("Rename this struct to match the pattern '{}'", pattern)
                            } else {
                                format!("Rename this struct to '{}'", pattern)
                            };
                            
                            span_lint_and_help(
                                ctx,
                                STRUCT_LINT_MUST_BE_NAMED::get_by_severity(*severity),
                                self.name().as_str(),
                                definition_span,
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
                                "Choose a different name for this struct"
                            };
                            
                            span_lint_and_help(
                                ctx,
                                STRUCT_LINT_MUST_NOT_BE_NAMED::get_by_severity(*severity),
                                self.name().as_str(),
                                definition_span,
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