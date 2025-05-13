use rustc_lint::{LateContext, LateLintPass, Lint, LintStore};
use rustc_hir::{Item, ItemKind, ImplItem, ImplItemKind, def_id::LOCAL_CRATE};
use rustc_session::impl_lint_pass;
use rustc_span::BytePos;
use regex::Regex;
use cargo_pup_lint_config::{ConfiguredLint, Severity, FunctionMatch, FunctionRule};
use crate::ArchitectureLintRule;
use crate::helpers::clippy_utils::span_lint_and_help;
use crate::helpers::queries::get_full_module_name;
use crate::declare_variable_severity_lint;

pub struct FunctionLint {
    name: String,
    matches: FunctionMatch,
    function_rules: Vec<FunctionRule>,
}

impl FunctionLint {
    pub fn new(config: &ConfiguredLint) -> Box<dyn ArchitectureLintRule + Send> {
        if let ConfiguredLint::Function(f) = config {
            Box::new(Self {
                name: f.name.clone(),
                matches: f.matches.clone(),
                function_rules: f.rules.clone(),
            })
        } else {
            panic!("Expected a Function lint configuration")
        }
    }
    
    // Helper method to check if a function in a given module with a given name should be linted
    fn matches_function(&self, module_path: &str, function_name: &str) -> bool {
        self.evaluate_function_match(&self.matches, module_path, function_name)
    }
    
    // Evaluates the complex matcher structure to determine if a function matches
    fn evaluate_function_match(&self, matcher: &FunctionMatch, module_path: &str, function_name: &str) -> bool {
        match matcher {
            FunctionMatch::NameEquals(name) => {
                function_name == name
            },
            FunctionMatch::NameRegex(pattern) => {
                match Regex::new(pattern) {
                    Ok(regex) => regex.is_match(function_name),
                    Err(_) => false,
                }
            },
            FunctionMatch::InModule(pattern) => {
                match Regex::new(pattern) {
                    Ok(regex) => regex.is_match(module_path),
                    Err(_) => module_path == pattern,
                }
            },
            FunctionMatch::AndMatches(left, right) => {
                self.evaluate_function_match(left, module_path, function_name) && 
                self.evaluate_function_match(right, module_path, function_name)
            },
            FunctionMatch::OrMatches(left, right) => {
                self.evaluate_function_match(left, module_path, function_name) || 
                self.evaluate_function_match(right, module_path, function_name)
            },
            FunctionMatch::NotMatch(inner) => {
                !self.evaluate_function_match(inner, module_path, function_name)
            }
        }
    }

    // Evaluate function rules
    fn evaluate_function_rule<'tcx>(&self, rule: &FunctionRule, ctx: &LateContext<'tcx>, 
                                  item_name: &str, body_id: rustc_hir::BodyId, 
                                  span: rustc_span::Span) -> bool {
        match rule {
            FunctionRule::MaxLength(max_lines, _) => {
                let body = ctx.tcx.hir_body(body_id);
                let source_map = ctx.tcx.sess.source_map();
                
                if let Ok(file_lines) = source_map.span_to_lines(body.value.span) {
                    file_lines.lines.len() > *max_lines
                } else {
                    false
                }
            }
            // Removed logical operator cases
        }
    }
}

// Declare the function_lint lint with variable severity
declare_variable_severity_lint!(
    pub,
    FUNCTION_LINT,
    FUNCTION_LINT_DENY, 
    FUNCTION_LINT_WARN,
    "Function properties and constraints"
);

impl_lint_pass!(FunctionLint => [FUNCTION_LINT_DENY, FUNCTION_LINT_WARN]);

impl ArchitectureLintRule for FunctionLint {
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
        let function_rules = self.function_rules.clone();
        
        lint_store.register_late_pass(move |_| {
            Box::new(FunctionLint {
                name: name.clone(),
                matches: matches.clone(),
                function_rules: function_rules.clone(),
            })
        });
    }
}

impl<'tcx> LateLintPass<'tcx> for FunctionLint {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // Only care about functions
        if let ItemKind::Fn { body, .. } = item.kind {
            let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id()).to_string();
            let _crate_name = ctx.tcx.crate_name(LOCAL_CRATE).to_string();
            let parent_item = ctx.tcx.hir_get_parent_item(item.hir_id());
            let module_path = get_full_module_name(&ctx.tcx, &parent_item);
            
            // Check if this function matches our patterns
            if !self.matches_function(&module_path, &item_name) {
                return;
            }
            
            // Apply rules
            for rule in &self.function_rules {
                match rule {
                    FunctionRule::MaxLength(max_lines, severity) => {
                        let body = ctx.tcx.hir_body(body);
                        let source_map = ctx.tcx.sess.source_map();
                        
                        if let Ok(file_lines) = source_map.span_to_lines(body.value.span) {
                            if file_lines.lines.len() > *max_lines {
                                // Create a span that only covers the function signature
                                let sig_span = item.span.with_hi(
                                    item.span.lo() + BytePos((item_name.len() + 5) as u32) // "fn name"
                                );
                                
                                span_lint_and_help(
                                    ctx,
                                    get_lint(*severity),
                                    self.name().as_str(),
                                    sig_span,
                                    format!(
                                        "Function exceeds maximum length of {} lines with {} lines",
                                        max_lines,
                                        file_lines.lines.len()
                                    ),
                                    None,
                                    "Consider breaking this function into smaller parts",
                                );
                            }
                        }
                    }
                    // Removed logical operators cases
                }
            }
        }
    }
    
    fn check_impl_item(&mut self, ctx: &LateContext<'tcx>, impl_item: &'tcx ImplItem<'tcx>) {
        if let ImplItemKind::Fn(_, body_id) = &impl_item.kind {
            let item_name = impl_item.ident.to_string();
            
            // Get the module path using the original code's approach
            let impl_block = ctx.tcx.hir_get_parent_item(impl_item.owner_id.into());
            let module = ctx.tcx.hir_get_parent_item(impl_block.into());
            let module_path = get_full_module_name(&ctx.tcx, &module);
            
            // Check if this method matches our patterns
            if !self.matches_function(&module_path, &item_name) {
                return;
            }
            
            // Apply rules
            for rule in &self.function_rules {
                match rule {
                    FunctionRule::MaxLength(max_lines, severity) => {
                        let body = ctx.tcx.hir_body(*body_id);
                        let source_map = ctx.tcx.sess.source_map();
                        
                        if let Ok(file_lines) = source_map.span_to_lines(body.value.span) {
                            if file_lines.lines.len() > *max_lines {
                                // Create a span that only covers the method signature
                                let sig_span = impl_item.span.with_hi(
                                    impl_item.span.lo() + BytePos((item_name.len() + 5) as u32) // "fn name"
                                );
                                
                                span_lint_and_help(
                                    ctx,
                                    get_lint(*severity),
                                    self.name().as_str(),
                                    sig_span,
                                    format!(
                                        "Function exceeds maximum length of {} lines with {} lines",
                                        max_lines,
                                        file_lines.lines.len()
                                    ),
                                    None,
                                    "Consider breaking this function into smaller parts",
                                );
                            }
                        }
                    }
                    // Removed logical operators cases
                }
            }
        }
    }
}

// Add method to determine the severity of a rule
impl FunctionLint {
    fn get_rule_severity(&self, rule: &FunctionRule) -> Severity {
        match rule {
            FunctionRule::MaxLength(_, severity) => *severity
            // Removed logical operator cases
        }
    }
} 