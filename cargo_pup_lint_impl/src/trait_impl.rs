use super::{ArchitectureLintRule, Severity};
use crate::declare_variable_severity_lint;
use crate::helpers::clippy_utils::span_lint_and_help;
use crate::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use rustc_hir::{Item, ItemKind, Node};
use rustc_lint::{LateContext, LateLintPass, Lint};
use rustc_middle::ty::Visibility;
use rustc_session::impl_lint_pass;
use serde::Deserialize;
use cargo_pup_common::project_context::ProjectContext;

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
                // We no longer need to construct the module name here since we use the helper
                
                // Get the canonical trait name using the centralized helper
                let trait_def_id = trait_ref.trait_def_id().unwrap();
                let full_trait_name = crate::helpers::queries::get_full_canonical_trait_name_from_def_id(&cx.tcx, trait_def_id);

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
    fn applies_to_trait(&self, trait_path: &str) -> bool {
        // Check if this lint applies to the given trait based on the source_name pattern
        self.name_regex.is_match(trait_path)
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
    
    fn generate_config(&self, context: &ProjectContext) -> anyhow::Result<std::collections::HashMap<String, String>> {
        use std::collections::HashMap;
    
        let mut configs = HashMap::new();
    
        // Generate a sample config for each trait
        for (i, trait_info) in context.traits.iter().enumerate().take(3) {
            // Only generate for traits that have implementations
            if !trait_info.implementors.is_empty() {
                // Create a rule name based on the trait, prefixed with module root
                let trait_parts: Vec<&str> = trait_info.name.split("::").collect();
                let trait_simple_name = trait_parts.last().unwrap_or(&"unknown");
                let rule_name = format!("{}_enforce_{}_impl", context.module_root, trait_simple_name.to_lowercase());
    
                // Load template from file and format it
                let template = include_str!("templates/trait_impl.tmpl");
                let impl_list = trait_info.implementors.join("\n#   ");
                let config = template.replace("{0}", &trait_info.name)
                                     .replace("{1}", &impl_list)
                                     .replace("{2}", &trait_info.name)
                                     .replace("{3}", trait_simple_name);
    
                configs.insert(rule_name, config);
    
                // Only generate a few examples
                if i >= 2 {
                    break;
                }
            }
        }
    
        // If no traits with impls were found, create a generic example
        if configs.is_empty() {
            let template = include_str!("templates/trait_impl_generic.tmpl");
            configs.insert(format!("{}_enforce_trait_impl", context.module_root), template.to_string());
        }
    
        Ok(configs)
    }
}    

#[cfg(test)]
pub mod tests {

    use super::*;
    use cargo_pup_common::project_context::{ProjectContext, TraitInfo};
    use LintConfigurationFactory;



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
    
    #[test]
    fn test_generate_config_with_traits() -> anyhow::Result<()> {
        // Create a factory instance
        let factory = TraitImplLintFactory::new();
        
        // Create a test context with some traits
        let traits = vec![
            TraitInfo {
                name: "test_crate::Display".to_string(),
                implementors: vec!["test_crate::User".to_string(), "test_crate::Product".to_string()],
                applicable_lints: vec![],
            },
            TraitInfo {
                name: "test_crate::Serialize".to_string(),
                implementors: vec!["test_crate::Config".to_string()],
                applicable_lints: vec![],
            },
        ];
        
        let context = ProjectContext::with_data(
            vec!["test_crate".to_string()],
            "test_crate".to_string(),
            traits
        );
        
        // Generate config
        let configs = factory.generate_config(&context)?;
        
        // We should have 2 configs for the 2 traits
        assert_eq!(configs.len(), 2, "Should generate 1 config per trait with implementations");
        
        // Check if the keys exist with module root prefix
        let display_key = format!("{}_enforce_display_impl", context.module_root);
        let serialize_key = format!("{}_enforce_serialize_impl", context.module_root);
        
        assert!(configs.contains_key(&display_key), 
                "Should contain key for Display trait with module root prefix");
        assert!(configs.contains_key(&serialize_key), 
                "Should contain key for Serialize trait with module root prefix");
        
        // Get the Display config
        let display_config = configs.get(&display_key).unwrap();
        
        // Verify content contains expected elements
        assert!(display_config.contains("type: trait_impl"), 
                "Config should specify trait_impl type");
        assert!(display_config.contains("source_name: \"test_crate::Display\""), 
                "Config should have correct trait name");
        assert!(display_config.contains("name_must_match: \".*DisplayImpl\""), 
                "Config should derive pattern from trait name");
        
        // Ensure the template was correctly loaded
        assert!(display_config.contains("Trait implementation lint for"), 
                "Config should contain text from template");
        
        Ok(())
    }
    
    #[test]
    fn test_generate_config_fallback() -> anyhow::Result<()> {
        // Create a factory instance
        let factory = TraitImplLintFactory::new();
        
        // Create a test context with no traits that have implementations
        let traits = vec![
            TraitInfo {
                name: "test_crate::EmptyTrait".to_string(),
                implementors: vec![],  // No implementations
                applicable_lints: vec![],
            },
        ];
        
        let context = ProjectContext::with_data(
            vec!["test_crate".to_string()],
            "test_crate".to_string(),
            traits
        );
        
        // Generate config
        let configs = factory.generate_config(&context)?;
        
        // Should use the fallback generic template
        assert_eq!(configs.len(), 1, "Should generate 1 fallback config");
        
        // Check if the fallback key exists with module root prefix
        let fallback_key = format!("{}_enforce_trait_impl", context.module_root);
        assert!(configs.contains_key(&fallback_key), 
                "Should contain fallback key with module root prefix");
        
        // Get the config
        let config = configs.get(&fallback_key).unwrap();
        
        // Verify content contains expected elements from the generic template
        assert!(config.contains("Example trait implementation rule"), 
                "Config should contain text from generic template");
        assert!(config.contains("source_name: \"core::example::Trait\""), 
                "Config should use generic example trait");
        
        Ok(())
    }
}
