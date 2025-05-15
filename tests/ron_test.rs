//! Integration test for pup.ron configuration
//!
//! This test verifies that we can build the same configuration programmatically
//! with cargo_pup_lint_config that we previously defined in pup.yaml.

use cargo_pup_lint_config::{FunctionLintExt, LintBuilder, ModuleLintExt, Severity, StructLintExt};

#[test]
fn test_lint_config_matches_yaml() {
    // Create a new LintBuilder
    let mut builder = LintBuilder::new();

    // Add TraitImpl rule for architecture_lint_rules
    builder
        .struct_lint() 
        .matching(|m| m.implements_trait("^pup_driver::lints::architecture_lint_rule::ArchitectureLintRule"))
        .with_severity(Severity::Error)
        .must_be_named(".*LintProcessor$".into())
        .must_be_private()
        .build();

    // Add TraitImpl rule for configuration_factories
    builder
        .struct_lint()
        .matching(|m| m.implements_trait("^pup_driver::lints::configuration_factory.rs::LintFactory"))
        .with_severity(Severity::Error)
        .must_be_named(".*LintFactory$".into())
        .must_be_private()
        .build();

    // Add EmptyMod rule for modules following the mod.rs structure
    builder
        .module()
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Warn)
        .must_be_empty()
        .build();

    // Add ItemType rule for helpers_no_structs_or_traits
    builder
        .module()
        .matching(|m| m.module("^pup_driver::lints::helpers$"))
        .with_severity(Severity::Error)
        .denied_items(vec![
            "struct".to_string(),
            "trait".to_string(),
        ])
        .build();

    /// Utils shouldn't contain structs or traits
    builder
        .module()
        .matching(|m| m.module("^pup_driver::utils$"))
        .with_severity(Severity::Error)
        .denied_items(vec![
            "struct".to_string(),
            "trait".to_string(),
        ])
        .build();

    // All Result<_,T> must return something implementing the error trait
    builder
        .function()
        .matching(|m| m.in_module(".*"))
        .with_severity(Severity::Error)
        .enforce_error_trait_implementation()
        .build();

    // cargo_pup shouldn't use the lints subsystem
    builder
        .module()
        .matching(|m| m.module("^cargo_pup::"))
        .with_severity(Severity::Error)
        .restrict_imports(None, Some(vec!["::lints".to_string()]))
        .build();

    // Write the configuration to pup.ron using the write_to_file method
    builder
        .write_to_file("pup.ron")
        .expect("Failed to write pup.ron file");

    println!("Successfully created and verified pup.ron configuration");
} 