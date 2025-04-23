use super::{ArchitectureLintRule, Severity};
use crate::lints::helpers::clippy_utils::span_lint_and_help;
use crate::lints::helpers::queries::implements_error_trait;
use crate::{
    declare_variable_severity_lint,
    utils::configuration_factory::{LintConfigurationFactory, LintFactory},
};
use regex::Regex;
use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LateLintPass, Lint};
use rustc_middle::ty::TyKind;
use rustc_session::impl_lint_pass;
use serde::Deserialize;

/// Configuration for Result error type lint rule
#[derive(Debug, Deserialize, Clone)]
pub struct ResultErrorConfiguration {
    pub modules: Vec<String>,
    pub severity: Severity,
}

/// Result error type lint processor
struct ResultErrorLintProcessor {
    name: String,
    rule: ResultErrorConfiguration,
    module_regexps: Vec<Regex>,
}

// Declare lint
declare_variable_severity_lint!(
    pub,
    RESULT_ERROR,
    RESULT_ERROR_DENY,
    RESULT_ERROR_WARN,
    "Result error types must implement Error trait"
);
impl_lint_pass!(ResultErrorLintProcessor => [RESULT_ERROR_DENY, RESULT_ERROR_WARN]);

impl ResultErrorLintProcessor {
    pub fn new(name: String, rule: ResultErrorConfiguration) -> Self {
        let module_regexps = rule
            .modules
            .iter()
            .map(|m| Regex::new(m).unwrap_or_else(|_| panic!("Can construct a regexp from {}", m)))
            .collect();

        Self {
            name,
            rule,
            module_regexps,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for ResultErrorLintProcessor {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // Add debug output to see if the module name matches any regex
        // let matches = self.applies_to_module(&ctx.tcx, &item.owner_id);
        let module_id = ctx.tcx.parent_module(item.hir_id());
        let module_def_id = module_id.to_def_id();
        let crate_name = ctx.tcx.crate_name(module_def_id.krate);
        let module_path = ctx.tcx.def_path_str(module_def_id);
        let full_name = format!("{}::{}", crate_name, module_path);

        let matches = self.module_regexps.iter().any(|r| r.is_match(&full_name));

        if !matches {
            return;
        }

        // Check if this is a function, static, or const item
        match &item.kind {
            ItemKind::Fn { sig, .. } => {
                if let rustc_hir::FnRetTy::Return(_) = sig.decl.output {
                    let fn_def_id = item.owner_id.to_def_id();
                    let fn_ty = ctx.tcx.type_of(fn_def_id).skip_binder();

                    // Get the actual return type from the function type
                    if let TyKind::FnDef(def_id, _) = fn_ty.kind() {
                        let fn_sig = ctx.tcx.fn_sig(*def_id);
                        let return_ty = fn_sig.skip_binder().output().skip_binder();

                        if let TyKind::Adt(adt_def, substs) = return_ty.kind() {
                            let path = ctx.tcx.def_path_str(adt_def.did());
                            if path == "std::result::Result" || path == "core::result::Result" {
                                if let Some(error_ty) = substs.types().nth(1) {
                                    // Check if the error type implements Error trait
                                    let param_env = ctx.param_env;
                                    let implements_error =
                                        implements_error_trait(ctx.tcx, param_env, error_ty);

                                    if !implements_error {
                                        let error_type_name = error_ty.to_string();
                                        span_lint_and_help(
                                            ctx,
                                            get_lint(self.rule.severity),
                                            self.name().as_str(),
                                            item.span,
                                            format!("Type '{}' is used as an error type in Result but does not implement Error trait", error_type_name).to_string(),
                                            None,
                                            "Implement the Error trait for this type or use a type that implements Error.",
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ItemKind::Static(_, _, _) | ItemKind::Const(_, _, _) => {
                let ty = ctx.tcx.type_of(item.owner_id.to_def_id()).skip_binder();
                if let TyKind::Adt(adt_def, substs) = ty.kind() {
                    let path = ctx.tcx.def_path_str(adt_def.did());
                    if path == "core::result::Result" {
                        if let Some(error_ty) = substs.types().nth(1) {
                            // Check if the error type implements Error trait
                            let param_env = ctx.param_env;
                            let implements_error =
                                implements_error_trait(ctx.tcx, param_env, error_ty);

                            if !implements_error {
                                let error_type_name = error_ty.to_string();
                                span_lint_and_help(
                                    ctx,
                                    get_lint(self.rule.severity),
                                    self.name().as_str(),
                                    item.span,
                                    format!("Type '{}' is used as an error type in Result but does not implement Error trait", error_type_name).to_string(),
                                    None,
                                    "Implement the Error trait for this type or use a type that implements Error.",
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl ArchitectureLintRule for ResultErrorLintProcessor {
    fn register_late_pass(&self, lint_store: &mut rustc_lint::LintStore) {
        let name = self.name.clone();
        let config = self.rule.clone();

        lint_store.register_late_pass(move |_| {
            Box::new(ResultErrorLintProcessor::new(name.clone(), config.clone()))
        });
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn applies_to_module(&self, module: &str) -> bool {
        self.module_regexps.iter().any(|r| r.is_match(module))
    }
}

/// Factory for creating Result error type lint processors
pub(crate) struct ResultErrorLintFactory {}

impl ResultErrorLintFactory {
    pub fn new() -> Self {
        ResultErrorLintFactory {}
    }
}

impl LintFactory for ResultErrorLintFactory {
    fn register() {
        LintConfigurationFactory::register_lint_factory("result_error", Self::new());
    }

    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        let raw_config: ResultErrorConfiguration = serde_yaml::from_value(yaml.clone())?;
        Ok(vec![Box::new(ResultErrorLintProcessor::new(
            rule_name.into(),
            raw_config,
        ))])
    }
    
    fn generate_config(&self, context: &crate::utils::project_context::ProjectContext) -> anyhow::Result<std::collections::HashMap<String, String>> {
        use std::collections::HashMap;
        
        let mut configs = HashMap::new();
        
        // Generate a crate-wide config based on the current crate
        let rule_name = format!("enforce_result_error_{}", context.module_root);
        
        // Build a regex pattern for the current crate
        let module_pattern = format!("^{}", context.module_root);
        
        // Load template from file and format it
        let template = include_str!("templates/result_error.tmpl");
        let config = template.replace("{0}", &context.module_root)
                             .replace("{1}", &module_pattern);
        
        configs.insert(rule_name.to_string(), config);
        
        Ok(configs)
    }
}

#[cfg(test)]
pub mod test {
    use crate::lints::result_error::ResultErrorLintFactory;
    use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
    use crate::utils::project_context::ProjectContext;

    const CONFIGURATION_YAML: &str = "
enforce_result_error:
  type: result_error
  modules:
    - \"test\"
  severity: Warn
";


    #[test]
    pub fn can_load_configuration_via_lint_factory() -> anyhow::Result<()> {
        ResultErrorLintFactory::register();
        let results = LintConfigurationFactory::from_yaml(CONFIGURATION_YAML.to_string())?;
        assert_eq!(results.len(), 1);
        Ok(())
    }
    
    #[test]
    pub fn test_generate_config_template() -> anyhow::Result<()> {
        // Create a factory instance
        let factory = ResultErrorLintFactory::new();
        
        // Create a test context
        let context = ProjectContext {
            modules: vec![
                "test_crate".to_string(),
                "test_crate::module1".to_string(),
            ],
            module_root: "test_crate".to_string(),
            traits: Vec::new(),
        };
        
        // Generate config
        let configs = factory.generate_config(&context)?;
        
        // Verify the configs map
        assert_eq!(configs.len(), 1, "Should generate exactly 1 config");
        
        // Check if the generated config has the expected name
        let expected_key = "enforce_result_error_test_crate";
        assert!(configs.contains_key(expected_key), 
                "Should contain key with format 'enforce_result_error_{}'", context.module_root);
        
        // Get the config
        let config = configs.get(expected_key).unwrap();
        
        // Verify content contains expected elements
        assert!(config.contains("type: result_error"), "Config should specify result_error type");
        assert!(config.contains("- \"^test_crate\""), "Config should have correct module pattern");
        
        // Ensure the template was correctly loaded
        assert!(config.contains("Result error type enforcement for the crate"), 
                "Config should contain text from template");
        
        Ok(())
    }
}
