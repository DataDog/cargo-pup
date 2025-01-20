use ctor::ctor;
use rustc_hir::{Item, ItemKind};
use rustc_middle::hir::map::{self};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use serde::Deserialize;
use std::collections::HashMap;

use super::{ArchitectureLintRule, LintResult, Severity};
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use anyhow::Result;

/// Represents a single namespace usage lint rule
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum NamespaceUsageLintRule {
    AllowOnly {
        allowed_namespaces: Vec<String>,
        severity: Severity,
    },
    Deny {
        denied_namespaces: Vec<String>,
        severity: Severity,
    },
}

/// Root configuration item from YAML
/// Takes a map of multiple names -> NamespaceUsageRuleConfiguration
/// Each NamespaceUsageRuleConfiguration will result in one instance of the LintProcessor
#[derive(Deserialize)]
pub struct NamespaceUsageConfiguration {
    pub rules: HashMap<String, NamespaceUsageRuleConfiguration>,
}

/// Represents a set of namespace usage lint rules for a module
#[derive(Debug, Deserialize, Clone)]
pub struct NamespaceUsageRuleConfiguration {
    pub namespaces: Vec<String>,
    pub rules: Vec<NamespaceUsageLintRule>,
}

/// Namespace usage lint processor that applies rules and collects results for
/// a particular namespace.
pub struct NamespaceUsageLintProcessor {
    name: String,
    config: NamespaceUsageRuleConfiguration,
}

impl NamespaceUsageLintProcessor {
    /// Configure multiple processors from a set of rules
    pub fn configure(rules: HashMap<String, NamespaceUsageRuleConfiguration>) -> Vec<Self> {
        rules
            .into_iter()
            .map(|(name, rule)| Self::new(name, rule))
            .collect()
    }

    /// Create a new processor for a single rule configuration
    pub fn new(name: String, config: NamespaceUsageRuleConfiguration) -> Self {
        NamespaceUsageLintProcessor { name, config }
    }

    /// Process a module and its imports to apply namespace usage lint rules
    pub fn process_module<'tcx>(
        &self,
        hir: map::Map<'tcx>,
        module: &Item<'tcx>,
    ) -> Vec<LintResult> {
        if let ItemKind::Mod(module_data) = module.kind {
            let module_name = module.ident.as_str();

            if self.config.namespaces.contains(&module_name.to_string()) {
                module_data
                    .item_ids
                    .iter()
                    .flat_map(|&item_id| {
                        let item = hir.item(item_id);
                        if let ItemKind::Use(path, _) = &item.kind {
                            let import_path: Vec<_> = path
                                .segments
                                .iter()
                                .map(|segment| segment.ident.as_str().to_string())
                                .collect();
                            let import_namespace = import_path.join("::");
                            self.check_rules(&self.config.rules, &import_namespace, item.span)
                        } else {
                            vec![]
                        }
                    })
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Check namespace usage rules against a specific import
    fn check_rules(
        &self,
        rules: &[NamespaceUsageLintRule],
        import_namespace: &str,
        span: Span,
    ) -> Vec<LintResult> {
        rules
            .iter()
            .filter_map(|rule| match rule {
                NamespaceUsageLintRule::AllowOnly {
                    allowed_namespaces,
                    severity,
                } => {
                    if !allowed_namespaces
                        .iter()
                        .any(|ns| import_namespace.starts_with(ns))
                    {
                        Some(LintResult {
                            lint: "namespace".into(),
                            lint_name: self.name.clone(),
                            span,
                            message: format!(
                                "Use of namespace '{}' is not allowed; only {:?} are permitted.",
                                import_namespace, allowed_namespaces
                            ),
                            severity: *severity,
                        })
                    } else {
                        None
                    }
                }
                NamespaceUsageLintRule::Deny {
                    denied_namespaces,
                    severity,
                } => {
                    if denied_namespaces
                        .iter()
                        .any(|ns| import_namespace.starts_with(ns))
                    {
                        Some(LintResult {
                            lint: "namespace".into(),
                            lint_name: self.name.clone(),
                            span,
                            message: format!(
                                "Use of namespace '{}' is denied; namespaces {:?} are not permitted.",
                                import_namespace, denied_namespaces
                            ),
                            severity: *severity,
                        })
                    } else {
                        None
                    }
                }
            })
            .collect()
    }
}

impl ArchitectureLintRule for NamespaceUsageLintProcessor {
    fn lint(&self, ctx: TyCtxt<'_>) -> Vec<LintResult> {
        ctx.hir()
            .krate()
            .owners
            .iter()
            .filter_map(|owner| owner.as_owner())
            .flat_map(|owner| {
                if let rustc_hir::OwnerNode::Item(item) = owner.node() {
                    self.process_module(ctx.hir(), item)
                } else {
                    vec![]
                }
            })
            .collect()
    }
}

/// Implement the `LintFactory` trait for dynamic configuration loading
pub(crate) struct NamespaceUsageLintFactory {}

impl NamespaceUsageLintFactory {
    pub fn new() -> Self {
        NamespaceUsageLintFactory {}
    }
}

impl LintFactory for NamespaceUsageLintFactory {
    fn register() {
        LintConfigurationFactory::register_lint_factory("namespace", Self::new());
    }

    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        // Deserialize the entire YAML as a HashMap
        let config: NamespaceUsageRuleConfiguration = serde_yaml::from_value(yaml.clone())?;
        Ok(vec![
            Box::new(NamespaceUsageLintProcessor::new(rule_name.into(), config))
                as Box<dyn ArchitectureLintRule + Send>,
        ])
    }
}

// Register the NamespaceUsageLintProcessor with the LintConfigurationFactory
#[ctor]
fn register_namespace_lint_factory() {
    LintConfigurationFactory::register_lint_factory(
        "namespace_usage",
        NamespaceUsageLintFactory::new(),
    );
}

#[cfg(test)]
mod tests {

    use crate::utils::lints_for_code;

    use super::*;

    const TEST_FN: &str = "
        mod test { 
            use std::collections::HashMap;
            use std::env;

            pub fn _test_fn() -> usize { 
                let mut map = HashMap::new(); // Allowed
                map.insert(\"key\", \"value\");
                let current_dir = env::current_dir().unwrap_or_default(); // Denied
                map.len() + current_dir.as_os_str().len()
            }
        }";

    #[test]
    pub fn allowed_usages_no_errors() {
        let namespace_rules = NamespaceUsageLintProcessor::new(
            "Test Rule".into(),
            NamespaceUsageRuleConfiguration {
                namespaces: vec!["test".to_string()],
                rules: vec![NamespaceUsageLintRule::Deny {
                    denied_namespaces: vec!["std::collections::VecDeque".into()], // Not used in test code
                    severity: Severity::Error,
                }],
            },
        );

        let lints = lints_for_code(TEST_FN, namespace_rules);
        assert_eq!(lints.lint_results().len(), 0);
    }

    #[test]
    pub fn denied_namespace_error() {
        let namespace_rules = NamespaceUsageLintProcessor::new(
            "Test Rule".into(),
            NamespaceUsageRuleConfiguration {
                namespaces: vec!["test".to_string()],
                rules: vec![NamespaceUsageLintRule::Deny {
                    denied_namespaces: vec!["std::env".into()],
                    severity: Severity::Error,
                }],
            },
        );

        let lints = lints_for_code(TEST_FN, namespace_rules);
        assert_eq!(lints.lint_results().len(), 1);
    }

    #[test]
    pub fn denied_parent_namespace_error() {
        let namespace_rules = NamespaceUsageLintProcessor::new(
            "Test Rule".into(),
            NamespaceUsageRuleConfiguration {
                namespaces: vec!["test".to_string()],
                rules: vec![NamespaceUsageLintRule::Deny {
                    denied_namespaces: vec!["std".into()],
                    severity: Severity::Error,
                }],
            },
        );

        let lints = lints_for_code(TEST_FN, namespace_rules);
        eprintln!("{}", lints.to_string());
        assert_eq!(lints.lint_results().len(), 2);
    }

    const CONFIGURATION_YAML: &str = "
test_me_namespace_rule:
  type: namespace_usage
  namespaces:
    - \"test_me\"
  rules:
    - type: Deny
      severity: Warn
      denied_namespaces:
        - \"std::collections\"

test_me_namespace_rule_two:
  type: namespace_usage
  namespaces:
    - \"test_me\"
  rules:
    - type: Deny
      severity: Error
      denied_namespaces:
        - \"anyhow::*\"
";

    #[test]
    pub fn can_load_configuration_via_lint_factory() -> Result<()> {
        // Register the factory
        LintConfigurationFactory::register_lint_factory(
            "namespace_usage",
            NamespaceUsageLintFactory::new(),
        );

        // Load configuration
        let results = LintConfigurationFactory::from_yaml(CONFIGURATION_YAML)?;

        // Assert the correct number of rules are loaded
        assert_eq!(results.len(), 2);

        Ok(())
    }
}
