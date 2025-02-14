use crate::lints::{ArchitectureLintRule, LintResult, Severity};
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{Impl, Item, ItemKind, OwnerNode, QPath, TyKind};
use rustc_middle::ty::TyCtxt;
use rustc_middle::ty::Visibility;
use rustc_middle::ty::layout::HasTyCtxt;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TraitImplConfiguration {
    /// The trait for whom we should enforce properties
    /// of the implementations
    pub source_name: String,

    // A regular expression that impls of the trait must match
    pub name_must_match: Option<String>,

    // Visibility that impls of the trait must have
    pub enforce_visibility: Option<RequiredVisibility>,

    // Severity of lint failure
    pub severity: Severity,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum RequiredVisibility {
    Private,
    Public,
}

pub(crate) struct TraitImplLintFactory {}
impl TraitImplLintFactory {
    pub fn new() -> Self {
        TraitImplLintFactory {}
    }
}

impl LintFactory for TraitImplLintFactory {
    fn register() {
        LintConfigurationFactory::register_lint_factory("trait_impl", Self::new());
    }

    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        // Parse the YAML into a structured configuration.
        let raw_config: TraitImplConfiguration = serde_yaml::from_value(yaml.clone())?;
        Ok(vec![
            Box::new(TraitImplLintProcessor::new(rule_name.into(), raw_config))
                as Box<dyn ArchitectureLintRule + Send>,
        ])
    }
}

pub struct TraitImplLintProcessor {
    rule_name: String,
    rule: TraitImplConfiguration,
}

impl TraitImplLintProcessor {
    pub fn new(rule_name: String, rule: TraitImplConfiguration) -> Self {
        Self { rule_name, rule }
    }
}

fn struct_name_from_hir(item: &Item<'_>) -> Option<String> {
    if let ItemKind::Impl(Impl { self_ty, .. }) = &item.kind {
        if let TyKind::Path(rustc_hir::QPath::Resolved(_, path)) = &self_ty.kind {
            if let Some(segment) = path.segments.last() {
                return Some(segment.ident.to_string());
            }
        }
    }
    None
}

impl ArchitectureLintRule for TraitImplLintProcessor {
    fn lint(&self, ctx: TyCtxt<'_>) -> Vec<LintResult> {
        let mut results = Vec::new();

        // Iterate over all HIR owners
        for owner in ctx
            .hir()
            .krate()
            .owners
            .iter()
            .filter_map(|owner| owner.as_owner())
        {
            if let OwnerNode::Item(item) = owner.node() {
                if let ItemKind::Impl(impl_item) = &item.kind {
                    // Check if the implementation matches the specified trait
                    if let Some(trait_ref) = &impl_item.of_trait {
                        let trait_name = ctx.def_path_str(trait_ref.trait_def_id().unwrap());
                        if trait_name == self.rule.source_name {
                            // We're on the impl of the trait. Let's get the struct that
                            // is implementing it so we point at the right thing for naming / visibility errors.
                            if let TyKind::Path(QPath::Resolved(_, path)) = &impl_item.self_ty.kind
                                && let Res::Def(DefKind::Struct, struct_def_id) = path.res
                            {
                                let struct_def_span = ctx.tcx().def_span(struct_def_id);
                                // Visibility check
                                if let Some(expected_visibility) = &self.rule.enforce_visibility {
                                    let struct_visibility = ctx.tcx().visibility(struct_def_id);
                                    let struct_name = ctx.def_path_str(struct_def_id);

                                    if *expected_visibility == RequiredVisibility::Private
                                        && struct_visibility == Visibility::Public
                                    {
                                        results.push(LintResult {
                                            lint_name: "trait_impl".into(),
                                            lint: self.rule_name.clone(),
                                            message: format!(
                                                "Struct '{}' is public, but should be private",
                                                struct_name,
                                            ),
                                            span: struct_def_span,
                                            severity: self.rule.severity,
                                        })
                                    }

                                    if *expected_visibility == RequiredVisibility::Public
                                        && struct_visibility != Visibility::Public
                                    {
                                        results.push(LintResult {
                                            lint_name: "trait_impl".into(),
                                            lint: self.rule_name.clone(),
                                            message: format!(
                                                "Struct '{}' is private, but should be public",
                                                struct_name,
                                            ),
                                            span: struct_def_span,
                                            severity: self.rule.severity,
                                        })
                                    }
                                }

                                // Name pattern check
                                if let Some(name_pattern) = &self.rule.name_must_match
                                    && let Some(struct_name) = struct_name_from_hir(item)
                                {
                                    let regex = Regex::new(name_pattern).expect("Invalid regex");
                                    if !regex.is_match(struct_name.as_str()) {
                                        results.push(LintResult {
                                            lint_name: "trait_impl".into(),
                                            lint: self.rule_name.clone(),
                                            message: format!(
                                                "Implementation name '{}' does not match the required pattern '{}'",
                                                struct_name,
                                                name_pattern
                                            ),
                                            span: struct_def_span,
                                            severity: self.rule.severity,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        results
    }

    fn name(&self) -> String {
        self.rule_name.clone()
    }

    fn applies_to_namespace(&self, _namespace: &str) -> bool {
        false
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::{
        lints::Severity,
        utils::test_helper::{assert_lint_results, lints_for_code},
    };

    const TEST_FN: &str = "
            mod test {
              trait MyTrait {
                 fn test_fn() -> i32;
              }

              pub struct MyStruct {}

              impl MyTrait for MyStruct {
                  fn test_fn() -> i32 {
                      let a = 1+1;
                      let b = 1+1;
                      let c = 1+1;
                      a + b + c
                  }
                }
            }
        ";

    #[test]
    pub fn impl_name_error() {
        let function_length_rules = TraitImplLintProcessor::new(
            "trait_name".into(),
            TraitImplConfiguration {
                source_name: "test::MyTrait".to_string(),
                name_must_match: Some(".*MyTraitImpl".into()),
                severity: Severity::Error,
                enforce_visibility: None,
            },
        );

        let lints = lints_for_code(TEST_FN, function_length_rules);
        assert_lint_results(1, &lints);
        assert!(lints.lint_results_text().contains(
            "Implementation name 'MyStruct' does not match the required pattern '.*MyTraitImpl'"
        ));
    }

    #[test]
    pub fn impl_name_no_error() {
        let function_length_rules = TraitImplLintProcessor::new(
            "trait_name".into(),
            TraitImplConfiguration {
                source_name: "test::MyTrait".to_string(),
                name_must_match: Some(".*Struct".into()),
                severity: Severity::Error,
                enforce_visibility: None,
            },
        );

        let lints = lints_for_code(TEST_FN, function_length_rules);
        assert_lint_results(0, &lints);
    }

    #[test]
    pub fn enforce_visibility_private_only() {
        let function_length_rules = TraitImplLintProcessor::new(
            "trait_name".into(),
            TraitImplConfiguration {
                source_name: "test::MyTrait".to_string(),
                name_must_match: None,
                severity: Severity::Error,
                enforce_visibility: Some(RequiredVisibility::Private),
            },
        );

        let lints = lints_for_code(TEST_FN, function_length_rules);
        assert_lint_results(1, &lints);
        assert!(
            lints
                .lint_results_text()
                .contains("Struct 'test::MyStruct' is public, but should be private")
        );
    }

    #[test]
    pub fn enforce_visibility_public_only() {
        let function_length_rules = TraitImplLintProcessor::new(
            "trait_name".into(),
            TraitImplConfiguration {
                source_name: "test::MyTrait".to_string(),
                name_must_match: None,
                severity: Severity::Error,
                enforce_visibility: Some(RequiredVisibility::Public),
            },
        );

        let lints = lints_for_code(TEST_FN, function_length_rules);
        assert_lint_results(0, &lints);
    }
}
