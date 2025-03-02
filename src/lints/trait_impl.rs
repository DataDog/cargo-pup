use super::{ArchitectureLintRule, Severity};
use crate::declare_variable_severity_lint;
use crate::lints::helpers::clippy_utils::span_lint_and_help;
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use rustc_hir::{Item, ItemKind, Node};
use rustc_lint::{LateContext, LateLintPass, Lint};
use rustc_middle::ty::Visibility;
use rustc_session::impl_lint_pass;
use serde::Deserialize;

/// Configuration for trait implementation lint rule
#[derive(Debug, Deserialize, Clone)]
pub struct TraitImplConfiguration {
    pub source_name: String,
    pub name_must_match: Option<String>,
    pub enforce_visibility: Option<RequiredVisibility>,
    pub severity: Severity,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum RequiredVisibility {
    Private,
    Public,
}

/// Trait implementation lint processor
struct TraitImplLintProcessor {
    name: String,
    name_regex: Regex,
    rule: TraitImplConfiguration,
}

// Declare lint
declare_variable_severity_lint!(
    pub,
    TRAIT_IMPL,
    TRAIT_IMPL_DENY,
    TRAIT_IMPL_WARN,
    "Trait implementation constraints violated"
);
impl_lint_pass!(TraitImplLintProcessor => [TRAIT_IMPL_DENY, TRAIT_IMPL_WARN]);

impl TraitImplLintProcessor {
    pub fn new(name: String, rule: TraitImplConfiguration) -> Self {
        let name_regex = Regex::new(rule.source_name.as_str())
            .expect("Failed constructing regexp for trait_impl trait name match");
        Self {
            name,
            rule,
            name_regex,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for TraitImplLintProcessor {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &Item<'tcx>) {
        if let ItemKind::Impl(impl_item) = &item.kind {
            if let Some(trait_ref) = &impl_item.of_trait {
                // Construct the full, crate-qualified name of the trait
                let module = cx
                    .tcx
                    .crate_name(item.owner_id.to_def_id().krate)
                    .to_ident_string();
                let trait_name = cx.tcx.def_path_str(trait_ref.trait_def_id().unwrap());
                let full_trait_name = format!("{}::{}", module, trait_name);

                // Do we match?
                if self.name_regex.is_match(&full_trait_name) {
                    if let rustc_hir::TyKind::Path(rustc_hir::QPath::Resolved(_, path)) =
                        &impl_item.self_ty.kind
                    {
                        if let Some(struct_def_id) = path.res.opt_def_id()
                            && let Some(struct_node) = cx.tcx.hir_get_if_local(struct_def_id)
                        {
                            if let Node::Item(struct_item) = struct_node {
                                if let ItemKind::Struct(_, _) = struct_item.kind {
                                    let struct_span = struct_item.span; // Span of the struct definition
                                    let struct_name =
                                        path.segments.last().map(|s| s.ident.to_string());

                                    if let Some(struct_name) = struct_name {
                                        // Check name pattern rule
                                        if let Some(name_pattern) = &self.rule.name_must_match {
                                            let regex =
                                                Regex::new(name_pattern).expect("Invalid regex");
                                            if !regex.is_match(&struct_name) {
                                                span_lint_and_help(
                                                    cx,
                                                    get_lint(self.rule.severity),
                                                    self.name().as_str(),
                                                    struct_span,
                                                    format!(
                                                        "Struct '{}' does not match the required pattern '{}'.",
                                                        struct_name, name_pattern
                                                    ),
                                                    None,
                                                    "Consider renaming the struct.",
                                                );
                                            }
                                        }

                                        // Check visibility rule
                                        if let Some(expected_visibility) =
                                            &self.rule.enforce_visibility
                                        {
                                            let struct_visibility =
                                                cx.tcx.visibility(struct_def_id);
                                            match expected_visibility {
                                                RequiredVisibility::Private => {
                                                    if struct_visibility == Visibility::Public {
                                                        span_lint_and_help(
                                                            cx,
                                                            get_lint(self.rule.severity),
                                                            self.name().as_str(),
                                                            struct_span,
                                                            format!(
                                                                "Struct '{}' is public, but should be private.",
                                                                struct_name
                                                            ),
                                                            None,
                                                            "Change the visibility to private.",
                                                        );
                                                    }
                                                }
                                                RequiredVisibility::Public
                                                    if struct_visibility != Visibility::Public =>
                                                {
                                                    span_lint_and_help(
                                                        cx,
                                                        get_lint(self.rule.severity),
                                                        self.name().as_str(),
                                                        struct_span,
                                                        format!(
                                                            "Struct '{}' is private, but should be public.",
                                                            struct_name
                                                        ),
                                                        None,
                                                        "Change the visibility to public.",
                                                    );
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl ArchitectureLintRule for TraitImplLintProcessor {
    fn register_late_pass(&self, lint_store: &mut rustc_lint::LintStore) {
        let name = self.name.clone();
        let config = self.rule.clone();
        lint_store.register_late_pass(move |_| {
            Box::new(TraitImplLintProcessor::new(name.clone(), config.clone()))
        });
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn applies_to_module(&self, _namespace: &str) -> bool {
        false
    }
}

/// Factory for creating trait implementation lint processors
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
        let raw_config: TraitImplConfiguration = serde_yaml::from_value(yaml.clone())?;
        Ok(vec![Box::new(TraitImplLintProcessor::new(
            rule_name.into(),
            raw_config,
        ))])
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
    #[ignore = "fix in-process testing framework"]
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
    }

    #[test]
    #[ignore = "fix in-process testing framework"]
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
    #[ignore = "fix in-process testing framework"]
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
        // assert!(
        //     lints
        //         .lint_results_text()
        //         .contains("Struct 'test::MyStruct' is public, but should be private")
        // );
    }

    #[test]
    #[ignore = "fix in-process testing framework"]
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

    const CONFIGURATION_YAML: &str = "
test_trait_constraint:
  type: trait_impl
  source_name: \"path_to::SomeTrait\"
  # ... must be named consistently
  name_must_match: \".*SomeTraitImpl\"
  # ... and must be private
  enforce_visibility: \"Private\"
  severity: Warn
";

    #[test]
    pub fn can_load_configuration_via_lint_factory() -> anyhow::Result<()> {
        // Register ourselves with the configuration factory

        TraitImplLintFactory::register();

        // Try load it
        let results = LintConfigurationFactory::from_yaml(CONFIGURATION_YAML.to_string())?;

        assert_eq!(results.len(), 1);

        Ok(())
    }
}
