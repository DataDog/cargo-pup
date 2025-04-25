use super::{ArchitectureLintRule, Severity};
use crate::declare_variable_severity_lint;
use crate::lints::helpers::clippy_utils::span_lint_and_help;
use crate::lints::helpers::queries::get_full_module_name;
use crate::lints::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use rustc_hir::{ImplItem, ImplItemKind, Item, ItemKind, OwnerId};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext};
use rustc_middle::ty::TyCtxt;
use rustc_session::impl_lint_pass;
use serde::Deserialize;
use std::collections::HashMap;
use crate::utils::project_context;

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
    
    fn generate_config(&self, context: &project_context::ProjectContext) -> anyhow::Result<HashMap<String, String>> {
        let mut configs = HashMap::new();
        
        // Create a single rule for the entire project, prefixed with module root
        let rule_name = format!("{}_max_function_length", context.module_root);
        
        // Create regex pattern that matches the root module and all submodules
        // The ^ ensures it starts with the module root, no need for $ or ::
        let module_pattern = format!("^{}", context.module_root);
        
        // Load template from file and format it
        let template = include_str!("templates/function_length.tmpl");
        let config = template.replace("{0}", &context.module_root)
                             .replace("{1}", &module_pattern);
        
        configs.insert(rule_name, config);
        
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
    use LintConfigurationFactory;
    use crate::utils::project_context::ProjectContext;
    use super::*;



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
    
    #[test]
    fn test_generate_config_template() -> anyhow::Result<()> {
        // Create a factory instance
        let factory = FunctionLengthLintFactory::new();
        
        // Create a test context
        let context = ProjectContext::with_data(
            vec![
                "test_crate".to_string(),
                "test_crate::module1".to_string(),
                "test_crate::module2".to_string(),
            ],
            "test_crate".to_string(),
            Vec::new()
        );
        
        // Generate config
        let configs = factory.generate_config(&context)?;
        
        // Verify the configs map
        assert_eq!(configs.len(), 1, "Should generate exactly 1 config");
        
        // Check if the key includes the module root prefix
        let expected_key = format!("{}_max_function_length", context.module_root);
        assert!(configs.contains_key(&expected_key), "Should contain '{}' key", expected_key);
        
        // Get the config using the expected key
        let config = configs.get(&expected_key).unwrap();
        
        // Verify content contains expected elements
        assert!(config.contains("type: function_length"), "Config should specify function_length type");
        assert!(config.contains("max_lines: 50"), "Config should set max_lines to 50");
        assert!(config.contains("namespace: \"^test_crate\""), "Config should have correct namespace pattern");
        
        // Ensure the template was correctly loaded
        assert!(config.contains("Function length lint for the entire project"), 
                "Config should contain text from template");
        
        Ok(())
    }
}
