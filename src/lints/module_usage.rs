use super::{ArchitectureLintRule, Severity};
use crate::declare_variable_severity_lint;
use crate::lints::helpers::clippy_utils::span_lint_and_help;
use crate::lints::helpers::queries::get_full_module_name;
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use rustc_hir::{Item, ItemKind, OwnerId, UseKind};
use rustc_lint::{LateContext, LateLintPass, Lint};
use rustc_middle::ty::TyCtxt;
use rustc_session::impl_lint_pass;
use serde::Deserialize;

/// Configuration for module usage lint rule
#[derive(Debug, Deserialize, Clone)]
pub struct ModuleUsageConfiguration {
    pub modules: Vec<String>,
    pub rules: Vec<ModuleUsageLintRule>,
}

/// Represents a single module usage lint rule
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ModuleUsageLintRule {
    AllowOnly {
        allowed_modules: Vec<String>,
        severity: Severity,
    },
    Deny {
        denied_modules: Vec<String>,
        severity: Severity,
    },
    DenyWildcard {
        severity: Severity,
    },
}

/// Module usage lint processor
struct ModuleUsageLintProcessor {
    name: String,
    config: ModuleUsageConfiguration,
    module_regexps: Vec<Regex>,
}

// Declare lint
declare_variable_severity_lint!(
    pub,
    MODULE_USAGE,
    MODULE_USAGE_DENY,
    MODULE_USAGE_WARN,
    "Module usage restrictions violated"
);
impl_lint_pass!(ModuleUsageLintProcessor => [MODULE_USAGE_DENY, MODULE_USAGE_WARN]);

impl ModuleUsageLintProcessor {
    pub fn new(name: String, config: ModuleUsageConfiguration) -> Self {
        let module_regexps = config
            .modules
            .iter()
            .map(|n| Regex::new(n).expect("Failed to create regex"))
            .collect();

        Self {
            name,
            config,
            module_regexps,
        }
    }

    fn applies_to_module(&self, tcx: &TyCtxt<'_>, module_def_id: &OwnerId) -> bool {
        let full_name = get_full_module_name(tcx, module_def_id);
        self.module_regexps
            .iter()
            .any(|r| r.is_match(full_name.as_str()))
    }
    
    /// Helper function to compile a module pattern string to regex
    fn compile_module_regex(pattern: &str) -> anyhow::Result<Regex> {
        Ok(Regex::new(pattern)?)
    }
}

impl<'tcx> LateLintPass<'tcx> for ModuleUsageLintProcessor {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let module_def_id = cx.tcx.hir_get_parent_item(item.hir_id());
        
        // Ensure we apply the lint to the right module
        if !self.applies_to_module(&cx.tcx, &module_def_id) {
            return;
        }

        // Check if the item is a `use` statement
        if let ItemKind::Use(path, use_kind) = &item.kind {
            let import_path: Vec<_> = path
                .segments
                .iter()
                .map(|s| s.ident.as_str().to_string())
                .collect();
            let import_module = import_path.join("::");

            for rule in &self.config.rules {
                match rule {
                    ModuleUsageLintRule::AllowOnly {
                        allowed_modules,
                        severity,
                    } => {
                        // Use regex matching to check allowed modules
                        let allowed = allowed_modules.iter().any(|pattern| {
                            if let Ok(re) = Self::compile_module_regex(pattern) {
                                re.is_match(&import_module)
                            } else {
                                // Fallback to simpler check if regex fails
                                import_module.starts_with(pattern)
                            }
                        });
                        
                        if !allowed {
                            span_lint_and_help(
                                cx,
                                get_lint(*severity),
                                self.name().as_str(),
                                item.span,
                                format!(
                                    "Use of module '{}' is not allowed; only {:?} are permitted.",
                                    import_module, allowed_modules
                                ),
                                None,
                                "Consider removing or changing the import.",
                            );
                        }
                    }
                    ModuleUsageLintRule::Deny {
                        denied_modules,
                        severity,
                    } => {
                        // Use regex matching to check denied modules
                        let denied = denied_modules.iter().any(|pattern| {
                            if let Ok(re) = Self::compile_module_regex(pattern) {
                                re.is_match(&import_module)
                            } else {
                                // Fallback to simpler check if regex fails
                                import_module.starts_with(pattern)
                            }
                        });
                        
                        if denied {
                            span_lint_and_help(
                                cx,
                                get_lint(*severity),
                                self.name().as_str(),
                                item.span,
                                format!(
                                    "Use of module '{}' is denied; {:?} are not permitted.",
                                    import_module, denied_modules
                                ),
                                None,
                                "Remove this import.",
                            );
                        }
                    }
                    ModuleUsageLintRule::DenyWildcard { severity } => {
                        if *use_kind == UseKind::Glob {
                            span_lint_and_help(
                                cx,
                                get_lint(*severity),
                                self.name().as_str(),
                                item.span,
                                format!(
                                    "Use of wildcard imports in '{}' is denied.",
                                    import_module
                                ),
                                None,
                                "Avoid wildcard imports.",
                            );
                        }
                    }
                }
            }
        }
    }
}

impl ArchitectureLintRule for ModuleUsageLintProcessor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn applies_to_module(&self, module: &str) -> bool {
        self.module_regexps.iter().any(|r| r.is_match(module))
    }

    fn register_late_pass(&self, lint_store: &mut rustc_lint::LintStore) {
        let name = self.name.clone();
        let config = self.config.clone();
        lint_store.register_late_pass(move |_| {
            Box::new(ModuleUsageLintProcessor::new(name.clone(), config.clone()))
        });
    }
}

/// Factory for creating module usage lint processors
pub(crate) struct ModuleUsageLintFactory {}

impl ModuleUsageLintFactory {
    pub fn new() -> Self {
        ModuleUsageLintFactory {}
    }
}

impl LintFactory for ModuleUsageLintFactory {
    fn register() {
        LintConfigurationFactory::register_lint_factory("module_usage", Self::new());
    }
    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        let raw_config: ModuleUsageConfiguration = serde_yaml::from_value(yaml.clone())?;
        Ok(vec![Box::new(ModuleUsageLintProcessor::new(
            rule_name.into(),
            raw_config,
        ))])
    }
}

#[cfg(test)]
mod tests {


    use super::*;



    const CONFIGURATION_YAML: &str = "
test_me_namespace_rule:
  type: module_usage
  modules:
    - \"test_me\"
  rules:
    - type: Deny
      severity: Warn
      denied_modules:
        - \"^std::collections\"

test_me_namespace_rule_two:
  type: module_usage
  modules:
    - \"test_me\"
  rules:
    - type: Deny
      severity: Error
      denied_modules:
        - \"^anyhow::.*\"
";

    #[test]
    pub fn can_load_configuration_via_lint_factory() -> anyhow::Result<()> {
        // Register the factory
        LintConfigurationFactory::register_lint_factory(
            "module_usage",
            ModuleUsageLintFactory::new(),
        );

        // Load configuration
        let results = LintConfigurationFactory::from_yaml(CONFIGURATION_YAML.to_string())?;

        // Assert the correct number of rules are loaded
        assert_eq!(results.len(), 2);

        Ok(())
    }
}
