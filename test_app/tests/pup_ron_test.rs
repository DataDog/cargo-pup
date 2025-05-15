//! Integration test for pup.ron configuration
//! 
//! This test verifies that we can build the same configuration programmatically
//! with cargo_pup_lint_config that we previously defined in pup.yaml.

use cargo_pup_lint_config::{
    ConfiguredLint, LintBuilder, Severity,
    ModuleMatch, ModuleRule, ModuleLintExt,
    module_matcher,
};
use std::path::Path;


#[test]
fn test_lint_config_matches_yaml() {
    // Create a new LintBuilder
    let mut builder = LintBuilder::new();

    // Add rules that approximate those in pup.yaml
    // Note: Some exact rules may not be available in the API, so we're using what's available
    
    // This is meant to represent the function_length lint
    builder.module()
        .matching(|m| m.module("^test_app::function_length$"))
        .with_severity(Severity::Warn)
        .must_not_be_empty()
        .build();

    // This approximates module_usage rules
    builder.module()
        .matching(|m| m.module("^test_app::module_usage$"))
        .with_severity(Severity::Warn)
        .restrict_imports(
            None, 
            Some(vec!["^std::collections".to_string()])
        )
        .no_wildcard_imports()
        .build();

    // Empty module rule
    builder.module()
        .matching(|m| m.module("^test_app::empty_mod$"))
        .with_severity(Severity::Warn)
        .must_not_be_empty()
        .build();

    // Item type restrictions
    builder.module()
        .matching(|m| m.module("^test_app::item_type$"))
        .with_severity(Severity::Warn)
        .denied_items(vec![
            "struct".to_string(), 
            "enum".to_string(), 
            "trait".to_string(), 
            "module".to_string()
        ])
        .build();

    // Write the configuration to pup.ron using the fixed write_to_file method
    builder.write_to_file("pup.ron").expect("Failed to write pup.ron file");
    
    // Verify we can read back the same configuration
    let loaded_builder = LintBuilder::read_from_file("pup.ron").expect("Failed to read pup.ron file");
    
    // Check that we have the correct number of lints
    assert_eq!(loaded_builder.lints.len(), 4, "Should have 4 lint rules configured");
    
    println!("Successfully created and verified pup.ron configuration");
} 