use super::{ArchitectureLintRule, Severity};
use crate::declare_variable_severity_lint;
use crate::lints::helpers::clippy_utils::span_lint_and_help;
use crate::lints::helpers::queries::get_full_module_name;
use crate::lints::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use rustc_hir::{Item, ItemKind, OwnerId};
use rustc_lint::{LateContext, LateLintPass, Lint, LintContext};
use rustc_middle::ty::TyCtxt;
use rustc_session::impl_lint_pass;
use rustc_span::FileName;
use serde::Deserialize;

/// Configuration for empty module lint rule
#[derive(Debug, Deserialize, Clone)]
pub struct EmptyModConfiguration {
    pub modules: Vec<String>,
    pub severity: Severity,
}

/// Empty module lint processor
struct EmptyModLintProcessor {
    name: String,
    rule: EmptyModConfiguration,
    module_regexps: Vec<Regex>,
}

// Declare lint
declare_variable_severity_lint!(
    pub,
    EMPTY_MOD,
    EMPTY_MOD_DENY,
    EMPTY_MOD_WARN,
    "Modules must be empty"
);
impl_lint_pass!(EmptyModLintProcessor => [EMPTY_MOD_DENY, EMPTY_MOD_WARN]);

impl EmptyModLintProcessor {
    pub fn new(name: String, rule: EmptyModConfiguration) -> Self {
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

    fn applies_to_module(&self, tcx: &TyCtxt<'_>, module_def_id: &OwnerId) -> bool {
        let full_name = get_full_module_name(tcx, module_def_id);
        self.module_regexps
            .iter()
            .any(|r| r.is_match(full_name.as_str()))
    }
}

impl<'tcx> LateLintPass<'tcx> for EmptyModLintProcessor {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let hir = ctx.tcx.hir();

        if let ItemKind::Mod(module_data) = item.kind {
            if !self.applies_to_module(&ctx.tcx, &item.owner_id) {
                return;
            }

            for item_id in module_data.item_ids {
                let item = ctx.tcx.hir_item(*item_id);
                let span = item.span;
                let item_name = hir.name(item.hir_id()).to_ident_string();

                let filename = ctx.sess().source_map().span_to_filename(span);
                if let FileName::Real(filename) = filename
                    && filename
                        .to_string_lossy(rustc_span::FileNameDisplayPreference::Local)
                        .ends_with("/mod.rs")
                {
                    match &item.kind {
                        ItemKind::Static(..)
                        | ItemKind::Struct(..)
                        | ItemKind::Union(..)
                        | ItemKind::Trait(..)
                        | ItemKind::Enum(..) => {
                            span_lint_and_help(
                                ctx,
                                get_lint(self.rule.severity),
                                self.name().as_str(),
                                span.shrink_to_lo(),
                                format!(
                                    "Item {} disallowed in mod.rs due to empty-module policy",
                                    item_name
                                ),
                                None,
                                "Remove this definition from the module.",
                            );
                        }
                        ItemKind::Impl(impl_data) if impl_data.of_trait.is_none() => {
                            span_lint_and_help(
                                ctx,
                                get_lint(self.rule.severity),
                                self.name().as_str(),
                                span,
                                format!(
                                    "Item {} disallowed in mod.rs due to empty-module policy",
                                    item_name
                                ),
                                None,
                                "Remove this implementation from the module.",
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

impl ArchitectureLintRule for EmptyModLintProcessor {
    fn register_late_pass(&self, lint_store: &mut rustc_lint::LintStore) {
        let name = self.name.clone();
        let config = self.rule.clone();

        lint_store.register_late_pass(move |_| {
            Box::new(EmptyModLintProcessor::new(name.clone(), config.clone()))
        });
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn applies_to_module(&self, module: &str) -> bool {
        self.module_regexps.iter().any(|r| r.is_match(module))
    }
}

/// Factory for creating empty module lint processors
pub(crate) struct EmptyModLintFactory {}

impl EmptyModLintFactory {
    pub fn new() -> Self {
        EmptyModLintFactory {}
    }
}

impl LintFactory for EmptyModLintFactory {
    fn register() {
        LintConfigurationFactory::register_lint_factory("empty_mod", Self::new());
    }

    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        let raw_config: EmptyModConfiguration = serde_yaml::from_value(yaml.clone())?;
        Ok(vec![Box::new(EmptyModLintProcessor::new(
            rule_name.into(),
            raw_config,
        ))])
    }

    fn generate_config(
        &self,
        context: &crate::utils::project_context::ProjectContext,
    ) -> anyhow::Result<std::collections::HashMap<String, String>> {
        use std::collections::HashMap;

        let mut configs = HashMap::new();

        // Generate a single rule for the entire crate
        let rule_name = format!("enforce_empty_mod_{}", context.module_root);

        // Load template from file and format it
        let template = include_str!("templates/empty_mod.tmpl");
        let config = template
            .replace("{0}", &context.module_root)
            .replace("{1}", &context.module_root);

        configs.insert(rule_name, config);

        Ok(configs)
    }
}
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
    use crate::utils::project_context::ProjectContext;

    const CONFIGURATION_YAML: &str = "
enforce_empty_mod:
  type: empty_mod
  modules:
    - \"some_module\"
  severity: Warn
";

    #[test]
    pub fn can_load_configuration_via_lint_factory() -> anyhow::Result<()> {
        // Register ourselves with the configuration factory
        EmptyModLintFactory::register();

        // Try load it
        let results = LintConfigurationFactory::from_yaml(CONFIGURATION_YAML.to_string())?;

        assert_eq!(results.len(), 1);

        Ok(())
    }

    #[test]
    fn test_generate_config_template() -> anyhow::Result<()> {
        // Create a factory instance
        let factory = EmptyModLintFactory::new();

        // Create a test context
        let context = ProjectContext::with_data(
            vec![
                "test_crate".to_string(),
                "test_crate::submodule".to_string(),
            ],
            "test_crate".to_string(),
            Vec::new()
        );

        // Generate config
        let configs = factory.generate_config(&context)?;

        // Verify the configs map
        assert_eq!(configs.len(), 1, "Should generate 1 config");

        // Check if the key exists
        let expected_key = "enforce_empty_mod_test_crate";
        assert!(
            configs.contains_key(expected_key),
            "Should contain expected key"
        );

        // Get the config
        let config = configs.get(expected_key).unwrap();

        // Verify content contains expected elements
        assert!(
            config.contains("type: empty_mod"),
            "Config should specify empty_mod type"
        );
        assert!(
            config.contains("modules:"),
            "Config should have modules section"
        );

        // Ensure the template was correctly loaded
        assert!(
            config.contains("Empty module enforcer for the entire crate"),
            "Config should contain text from template"
        );

        Ok(())
    }
}
