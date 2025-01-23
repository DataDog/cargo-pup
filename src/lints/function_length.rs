use super::{ArchitectureLintRule, LintResult, Severity};
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use rustc_hir::{ImplItem, ImplItemKind, Item, ItemKind};
use rustc_middle::hir::map::{self};
use rustc_middle::ty::TyCtxt;
use rustc_span::source_map::SourceMap;
use rustc_span::Span;
use serde::Deserialize;

/// Represents a set of function length lint rules for a module
#[derive(Debug, Deserialize)]
pub struct FunctionLengthConfiguration {
    pub namespace: String,
    pub max_lines: usize,
    pub severity: Severity,
}

/// Function length lint processor that applies rules and collects results
pub struct FunctionLengthLintProcessor {
    name: String,
    rule: FunctionLengthConfiguration,
}

impl FunctionLengthLintProcessor {
    pub fn new(name: String, rule: FunctionLengthConfiguration) -> Self {
        Self { name, rule }
    }

    /// Process a module and its functions to apply function length lint rules
    pub fn process_module<'tcx>(
        &self,
        hir: map::Map<'tcx>,
        module: &Item<'tcx>,
        source_map: &SourceMap,
    ) -> Vec<LintResult> {
        if let ItemKind::Mod(module_data) = module.kind {
            let module_name = module.ident.as_str();

            if self.rule.namespace.eq(module_name) {
                module_data
                    .item_ids
                    .iter()
                    .flat_map(|&item_id| {
                        let item = hir.item(item_id);
                        match &item.kind {
                            ItemKind::Fn { sig, body, .. } => {
                                let body = hir.body(*body);
                                self.check_function_length(body.value.span, sig.span, source_map)
                            }
                            ItemKind::Impl(impl_) => impl_
                                .items
                                .iter()
                                .flat_map(|impl_item_ref| {
                                    let impl_item = hir.impl_item(impl_item_ref.id);
                                    self.process_impl_item(impl_item, hir, source_map)
                                })
                                .collect::<Vec<_>>(),
                            _ => vec![],
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

    /// Process an individual impl item
    fn process_impl_item<'tcx>(
        &self,
        impl_item: &ImplItem<'tcx>,
        hir: map::Map<'tcx>,
        source_map: &SourceMap,
    ) -> Vec<LintResult> {
        match &impl_item.kind {
            ImplItemKind::Fn(sig, body_id) => {
                let body = hir.body(*body_id);
                self.check_function_length(body.value.span, sig.span, source_map)
            }
            _ => vec![],
        }
    }

    /// Check if a function exceeds the allowed maximum length
    fn check_function_length(
        &self,
        body_span: Span,
        header_span: Span,
        source_map: &SourceMap,
    ) -> Vec<LintResult> {
        let lines = match source_map.span_to_lines(body_span) {
            Ok(file_lines) => file_lines.lines.len(),
            Err(_) => return vec![], // Skip if we can't determine line count
        };

        if lines > self.rule.max_lines {
            vec![LintResult {
                lint: "function_length".into(),
                lint_name: self.name.clone(),
                span: header_span, // Use the header span instead of the body span
                message: format!(
                    "Function exceeds maximum length of {} lines (found {}) for namespace '{}'.",
                    self.rule.max_lines, lines, self.rule.namespace
                ),
                severity: self.rule.severity,
            }]
        } else {
            vec![]
        }
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
}

impl ArchitectureLintRule for FunctionLengthLintProcessor {
    fn lint(&self, ctx: TyCtxt<'_>) -> Vec<LintResult> {
        ctx.hir()
            .krate()
            .owners
            .iter()
            .filter_map(|owner| owner.as_owner())
            .flat_map(|owner| {
                if let rustc_hir::OwnerNode::Item(item) = owner.node() {
                    self.process_module(ctx.hir(), item, ctx.sess.source_map())
                } else {
                    vec![]
                }
            })
            .collect()
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
 test_me_function_length_rule:
    type: function_length
    namespace: \"test_me\"
    max_lines: 10
    severity: Error
";

    #[test]
    pub fn can_load_configuration_via_lint_factory() -> anyhow::Result<()> {
        // Register ourselves with the configuration factory

        FunctionLengthLintFactory::register();

        // Try load it
        let results = LintConfigurationFactory::from_yaml(CONFIGURATION_YAML)?;

        assert_eq!(results.len(), 1);

        Ok(())
    }
}
