use super::{ArchitectureLintRule, Severity};
use crate::declare_variable_severity_lint;
use crate::lints::helpers::clippy_utils::span_lint_and_help;
use crate::lints::helpers::queries::get_full_module_name;
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use rustc_hir::{ImplItem, ImplItemKind, Item, ItemKind, OwnerId};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext};
use rustc_middle::ty::TyCtxt;
use rustc_session::impl_lint_pass;
use serde::Deserialize;
use std::collections::HashMap;
use crate::utils::config_generation;

/// Represents a set of function length lint rules for a module
#[derive(Debug, Deserialize, Clone)]
pub struct FunctionLengthConfiguration {
    pub namespace: String,
    pub max_lines: usize,
    pub severity: Severity,
}

/// Function length lint processor that applies rules and collects results
struct FunctionLengthLintProcessor {
    name: String,
    rule: FunctionLengthConfiguration,
    namespace_match: Regex,
}

// Setup our lint
declare_variable_severity_lint!(
    pub,
    pup_function_length,
    FUNCTION_LENGTH_DENY,
    FUNCTION_LENGTH_WARN,
    "Function length must not exceed given size"
);
impl_lint_pass!(FunctionLengthLintProcessor => [FUNCTION_LENGTH_DENY, FUNCTION_LENGTH_WARN]);

impl FunctionLengthLintProcessor {
    pub fn new(name: String, rule: FunctionLengthConfiguration) -> Self {
        let namespace_match = Regex::new(&rule.namespace).unwrap_or_else(|_| {
            panic!(
                "Couldn't create regexp for namespace match: {:?}",
                rule.namespace.as_str()
            )
        });

        Self {
            name,
            rule,
            namespace_match,
        }
    }

    fn applies_to_module(&self, tcx: &TyCtxt<'_>, module_def_id: &OwnerId) -> bool {
        let full_name = get_full_module_name(tcx, module_def_id);
        let the_match = self.namespace_match.is_match(full_name.as_str());
        the_match
    }
}

pub(crate) struct FunctionLengthLintFactory {}

impl FunctionLengthLintFactory {
    pub fn new() -> Self {
        FunctionLengthLintFactory {}
    }
}

impl LintFactory for FunctionLengthLintFactory {
    fn register() {
        LintConfigurationFactory::register_lint_factory("function_length", Self::new());
    }

    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        // Parse the YAML into a structured configuration.
        let raw_config: FunctionLengthConfiguration = serde_yaml::from_value(yaml.clone())?;

        Ok(vec![Box::new(FunctionLengthLintProcessor::new(
            rule_name.into(),
            raw_config,
        )) as Box<dyn ArchitectureLintRule + Send>])
    }
    
    fn generate_config(&self, context: &config_generation::GenerationContext) -> anyhow::Result<HashMap<String, String>> {
        let mut configs = HashMap::new();
        
        // Add a rule specifically for the root module itself
        let rule_name = format!("max_function_length_{}", context.module_root);
        
        // Create regex pattern that matches the exact module root
        // This will target the root module without including submodules
        let module_pattern = format!("^{}$", context.module_root);
            
        // Create a sample config with comments
        let config = format!(
            r#"  # Function length lint for root module
    #
    # This rule checks that functions in the root module
    # don't exceed the maximum allowed length.
    #
    # Crate: {}
    #
    # Parameters:
    #   namespace: regex pattern for module to check
    #   max_lines: maximum allowed function length in lines
    #   severity: Error or Warn
    #
    type: function_length
    namespace: "{}"
    max_lines: 30
    severity: Warn"#,
            context.module_root,
            module_pattern
        );
        
        configs.insert(rule_name.to_string(), config);
        
        // Add a rule for the entire crate (including submodules)
        let rule_name = format!("max_function_length_{}_all", context.module_root);
        
        // Create regex pattern for the entire crate including submodules
        let module_pattern = format!("^{}::", context.module_root);
            
        // Create a sample config with comments
        let config = format!(
            r#"    # Function length lint for entire crate
    #
    # This rule checks that functions across the entire crate
    # don't exceed the maximum allowed length.
    #
    # Crate: {}
    #
    # Parameters:
    #   namespace: regex pattern for modules to check
    #   max_lines: maximum allowed function length in lines
    #   severity: Error or Warn
    #
    type: function_length
    namespace: "{}"
    max_lines: 30
    severity: Warn"#,
            context.module_root,
            module_pattern
        );
        
        configs.insert(rule_name.to_string(), config);
        
        // Generate a sample config for a specific important submodule if available
        if let Some(module) = context.modules.iter().find(|m| m.contains("::")) {
            // Create a rule name based on the module
            let module_parts: Vec<&str> = module.split("::").collect();
            let rule_name = format!("max_function_length_{}", module_parts.last().unwrap_or(&"submodule"));
            
            // Create a sample config with comments
            let config = format!(
                "    # Function length lint for {} submodule\n\
                 #\n\
                 # This rule checks that functions in a specific submodule\n\
                 # don't exceed the maximum allowed length.\n\
                 #\n\
                 # Parameters:\n\
                 #   namespace: regex pattern for module to check\n\
                 #   max_lines: maximum allowed function length in lines\n\
                 #   severity: Error or Warn\n\
                 #\n\
                 type: function_length\n\
                 namespace: \"{}\"\n\
                 max_lines: 30\n\
                 severity: Warn",
                module_parts.last().unwrap_or(&"specific"),
                // Escape regex special characters
                module.replace(".", "\\.").replace("(", "\\(").replace(")", "\\)")
            );
            
            configs.insert(rule_name, config);
        }
        
        Ok(configs)
    }
}

impl<'tcx> LateLintPass<'tcx> for FunctionLengthLintProcessor {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &Item<'tcx>) {
        if let ItemKind::Fn {
            sig: _,
            generics: _,
            body,
            has_body: _,
        } = &item.kind
        {
            let body = cx.tcx.hir_body(*body);
            let source_map = cx.sess().source_map();
            let line_limit = self.rule.max_lines;
            let module = cx.tcx.hir_get_parent_item(item.owner_id.into());

            if self.applies_to_module(&cx.tcx, &module) {
                if let Ok(file_lines) = source_map.span_to_lines(body.value.span) {
                    if file_lines.lines.len() > line_limit {
                        span_lint_and_help(
                            cx,
                            get_lint(self.rule.severity),
                            self.name().as_str(),
                            item.span,
                            format!(
                                "Function exceeds maximum length of {} lines with {} lines",
                                line_limit,
                                file_lines.lines.len()
                            ),
                            None,
                            "",
                        );
                    }
                }
            }
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &ImplItem<'tcx>) {
        if let ImplItemKind::Fn(_fn_sig, body_id) = &impl_item.kind {
            let body = cx.tcx.hir_body(*body_id);
            let source_map = cx.sess().source_map();
            let line_limit = self.rule.max_lines;

            // This is the containing impl block
            let impl_block = cx.tcx.hir_get_parent_item(impl_item.owner_id.into());
            let module = cx.tcx.hir_get_parent_item(impl_block.into());
            if self.applies_to_module(&cx.tcx, &module) {
                if let Ok(file_lines) = source_map.span_to_lines(body.value.span) {
                    if file_lines.lines.len() > line_limit {
                        span_lint_and_help(
                            cx,
                            get_lint(self.rule.severity),
                            self.name().as_str(),
                            impl_item.span,
                            format!(
                                "Function exceeds maximum length of {} lines with {} lines",
                                line_limit,
                                file_lines.lines.len()
                            ),
                            None,
                            "",
                        );
                    }
                }
            }
        }
    }
}

impl ArchitectureLintRule for FunctionLengthLintProcessor {
    fn register_late_pass(&self, lint_store: &mut rustc_lint::LintStore) {
        let name = self.name.clone();
        let config = self.rule.clone();

        lint_store.register_late_pass(move |_| {
            // let config = config.clone();
            let lint = FunctionLengthLintProcessor::new(name.clone(), config.clone());
            Box::new(lint)
        });
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn applies_to_module(&self, namespace: &str) -> bool {
        self.namespace_match.is_match(namespace)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{
        configuration_factory::LintConfigurationFactory,
        test_helper::{assert_lint_results, lints_for_code},
    };

    use super::*;

    const TEST_FN: &str = "
            mod test { 
              pub fn _test_fn() -> i32 { 
                  let a = 1+1;
                  let b = 1+1;
                  let c = 1+1;
                  a + b + c
              }
            }
        ";

    #[test]
    #[ignore = "fix in-process testing framework"]
    pub fn short_function_no_error() {
        let function_length_rules = FunctionLengthLintProcessor::new(
            "test".into(),
            FunctionLengthConfiguration {
                namespace: "test".into(),
                max_lines: 6,
                severity: Severity::Error,
            },
        );

        let lints = lints_for_code(TEST_FN, function_length_rules);
        assert_lint_results(0, &lints);
    }

    #[test]
    #[ignore = "fix in-process testing framework"]
    pub fn long_function_error() {
        let function_length_rules = FunctionLengthLintProcessor::new(
            "test".into(),
            FunctionLengthConfiguration {
                namespace: "test".into(),
                max_lines: 1,
                severity: Severity::Error,
            },
        );

        let lints = lints_for_code(TEST_FN, function_length_rules);
        assert_lint_results(1, &lints);
    }

    const CONFIGURATION_YAML: &str = "
deny_long_functions:
  type: function_length
  namespace: some_namespace_match
  max_lines: 5
  severity: Warn
";

    #[test]
    pub fn can_load_configuration_via_lint_factory() -> anyhow::Result<()> {
        // Register ourselves with the configuration factory

        FunctionLengthLintFactory::register();

        // Try load it
        let results = LintConfigurationFactory::from_yaml(CONFIGURATION_YAML.to_string())?;

        assert_eq!(results.len(), 1);

        Ok(())
    }
}
