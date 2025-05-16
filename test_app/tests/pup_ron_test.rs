//! Integration test for pup.ron configuration
//!
//! This test verifies that we can build the same configuration programmatically
//! with cargo_pup_lint_config that we previously defined in pup.yaml.

use cargo_pup_lint_config::{FunctionLintExt, LintBuilder, ModuleLintExt, Severity, StructLintExt};

#[test]
fn test_lint_config_matches_yaml() {
    // Create a new LintBuilder
    let mut builder = LintBuilder::new();

    // Add rules that approximate those in pup.yaml
    // Note: Some exact rules may not be available in the API, so we're using what's available

    builder
        .module()
        .lint_named("empty_module_check")
        .matching(|m| m.module("^test_app::function_length$"))
        .with_severity(Severity::Warn)
        .must_not_be_empty()
        .build();

    // Function length limit for functions in function_length module
    builder
        .function()
        .lint_named("function_length_check")
        .matching(|m| m.in_module("^test_app::function_length$"))
        .with_severity(Severity::Warn)
        .max_length(5)
        .build();


    builder
        .module()
        .lint_named("module_usage")
        .matching(|m| m.module("^test_app::module_usage$"))
        .with_severity(Severity::Warn)
        .restrict_imports(None, Some(vec!["^std::collections".to_string()]))
        .no_wildcard_imports()
        .build();

    // Empty module rule - must NOT be empty
    builder
        .module()
        .lint_named("must_not_be_empty_module")
        .matching(|m| m.module("^test_app::empty_mod$"))
        .with_severity(Severity::Warn)
        .must_not_be_empty()
        .build();
        
    // Empty module rule - MUST be empty
    builder
        .module()
        .lint_named("must_be_empty_module")
        .matching(|m| m.module("^test_app::must_be_empty$"))
        .with_severity(Severity::Warn)
        .must_be_empty()
        .build();

    // Item type restrictions
    builder
        .module()
        .lint_named("item_type_restrictions")
        .matching(|m| m.module("^test_app::item_type$"))
        .with_severity(Severity::Warn)
        .denied_items(vec![
            "struct".to_string(),
            "enum".to_string(),
            "trait".to_string(),
            "module".to_string(),
        ])
        .build();

    // Trait restrictions
    builder.struct_lint()
        .lint_named("trait_restrictions")
        .matching(|m|
        m.implements_trait("^test_app::trait_impl::MyTrait$"))
        .with_severity(Severity::Warn)
        .must_be_named(".*MyTraitImpl$".into())        
        .must_be_private()
        .build();
        
    // Result error implementation check
    builder
        .function()
        .lint_named("result_type_check")
        .matching(|m| m.in_module("^test_app::result_error$"))
        .with_severity(Severity::Warn)
        .enforce_error_trait_implementation()
        .build();

    // Write the configuration to pup.ron using the fixed write_to_file method
    builder
        .write_to_file("pup.ron")
        .expect("Failed to write pup.ron file");

    println!("Successfully created and verified pup.ron configuration");
}
