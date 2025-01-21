//! Namespace usage lints
//! 
//! The module provides tools to lint namespace usage. 
//! 
//! ## Features
//! 
//! - Allow or deny specific namespaces or namespace paths
//! - Deny usage of wildcard imports (e.g., `use std::io::*`)
//!   Inspired by Canonical's [import discpline best-practice](https://canonical.github.io/rust-best-practices/import-discipline.html)
//! 

use ctor::ctor;
use rustc_hir::{Item, ItemKind, UseKind};
use rustc_middle::hir::map::{self};
use rustc_middle::ty::TyCtxt;
use rustc_span::{FileName, RealFileName, Span};
use serde::Deserialize;

use super::{ArchitectureLintRule, LintResult, Severity};
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use anyhow::Result;

/// Represents a single namespace usage lint rule
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum NamespaceUsageLintRule {
    ///
    /// Only allows the specified namespaces to be used.
    /// 
    /// If a namespace is not listed in `allowed_namespaces`, it will be denied.
    /// Wildcards are supported.
    /// 
    AllowOnly {
        allowed_namespaces: Vec<String>,
        severity: Severity,
    },

    ///
    /// Denies the use of specific namespaces.
    /// 
    /// If a namespace matches any in `denied_namespaces`, it will be denied.
    /// Wildcards are supported.
    /// 
    Deny {
        denied_namespaces: Vec<String>,
        severity: Severity,
    },

    ///
    /// Denies usage of wildcard imports, e.g., `std::collections::*`.
    /// 
    DenyWildcard {
        severity: Severity
    },

    ///
    /// Requires that mod.rs does not contain any definitions, only
    /// uses and re-exports.
    /// https://canonical.github.io/rust-best-practices/structural-discipline.html
    ///
    RequireEmptyMod {
        severity: Severity
    }
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
    /// Create a new processor for a single rule configuration
    pub fn new(name: String, config: NamespaceUsageRuleConfiguration) -> Self {
        NamespaceUsageLintProcessor { name, config }
    }

    /// Process a module and its imports to apply namespace usage lint rules
    pub fn process_module<'tcx>(
        &self,
        ctx: TyCtxt<'tcx>,
        module: &Item<'tcx>,
    ) -> Vec<LintResult> {
        let hir = ctx.hir();
        if let ItemKind::Mod(module_data) = module.kind {
            let module_name = module.ident.as_str();
            if self.config.namespaces.contains(&module_name.to_string()) {
                let mut namespace_results = module_data
                    .item_ids
                    .iter()
                    .flat_map(|&item_id| {
                        let item = hir.item(item_id);
                        if let ItemKind::Use(path, use_kind) = &item.kind {
                            let import_path: Vec<_> = path
                                .segments
                                .iter()
                                .map(|segment| segment.ident.as_str().to_string())
                                .collect();
                            let import_namespace = import_path.join("::");
                            self.check_namespace_import_rules(
                                &self.config.rules,
                                &import_namespace,
                                use_kind,
                                item.span,
                            )
                        } else {
                            vec![]
                        }
                    })
                    .collect();

                    // Do we have the "require empty module" rule, and, are we in a
                    // mod.rs?
                    let filename = ctx.sess.source_map().span_to_filename(module.span);
                    if let FileName::Real(filename) = filename &&
                        filename.to_string_lossy(rustc_span::FileNameDisplayPreference::Local).ends_with("mod.rs") {
                            // Do we have the empty mod rule ?
                            if let Some(NamespaceUsageLintRule::RequireEmptyMod { severity }) = 
                                self.config.rules.iter().find(|rule| matches!(rule, NamespaceUsageLintRule::RequireEmptyMod { .. }))
                            {
                                let empty_mod_results = self.check_empty_module(severity, module_data.item_ids.iter());
                                eprintln!("{:?}", filename);
                            }
                    }
                    namespace_results
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Check the empty module rule
    fn check_empty_module(
        &self,
        severity: &Severity,
        item_ids: std::slice::Iter<'_, rustc_hir::ItemId>) -> Vec<LintResult> {
        vec![]
    }
        
    
    /// Check namespace usage rules against a specific import
    fn check_namespace_import_rules(
        &self,
        rules: &[NamespaceUsageLintRule],
        import_namespace: &str,
        use_kind: &UseKind,
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
                NamespaceUsageLintRule::DenyWildcard { 
                    severity
                } => {
                    if use_kind == &UseKind::Glob {
                        Some(LintResult {
                            lint: "namespace".into(),
                            lint_name: self.name.clone(),
                            span,
                            message: format!(
                                "Use of wildcard imports in '{}' is denied.",
                                import_namespace),                            
                            severity: *severity
                            })
                    } else {
                        None
                    }
                }
                // Empty module rule is applied elsewhere
                NamespaceUsageLintRule::RequireEmptyMod { .. } => None 
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
                    self.process_module(ctx, item)
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
            use std::io::*;
            
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
        assert_eq!(lints.lint_results().len(), 3);
    }

    #[test]
    pub fn denied_wildcard_error() {
        let namespace_rules = NamespaceUsageLintProcessor::new(
            "Deny wildcards".into(),
            NamespaceUsageRuleConfiguration { 
                namespaces: vec!["test".to_string()], 
                rules:  vec![NamespaceUsageLintRule::DenyWildcard {
                    severity: Severity::Warn
                }]
            });

        let lints = lints_for_code(TEST_FN, namespace_rules);
        eprintln!("{}", lints.to_string());
        assert_eq!(lints.lint_results().len(), 1);
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
