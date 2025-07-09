// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//! Integration test for pup.ron configuration
//!
//! This test verifies that we can build a configuration with pup.ron

use cargo_pup_lint_config::{FunctionLintExt, LintBuilder, ModuleLintExt, Severity, StructLintExt};

#[test]
fn test_lint_config() {
    // Create a new LintBuilder
    let mut builder = LintBuilder::new();

    builder
        .module_lint()
        .lint_named("empty_module_check")
        .matching(|m| m.module("^test_app::function_length$"))
        .with_severity(Severity::Warn)
        .must_not_be_empty()
        .build();

    // Function length limit for functions in function_length module
    builder
        .function_lint()
        .lint_named("function_length_check")
        .matching(|m| m.in_module("^test_app::function_length$"))
        .with_severity(Severity::Warn)
        .max_length(5)
        .build();


    builder
        .module_lint()
        .lint_named("module_usage")
        .matching(|m| m.module("^test_app::module_usage$"))
        .with_severity(Severity::Warn)
        .restrict_imports(None, Some(vec!["^std::collections".to_string()]))
        .no_wildcard_imports()
        .build();

    // Empty module rule - must NOT be empty
    builder
        .module_lint()
        .lint_named("must_not_be_empty_module")
        .matching(|m| m.module("^test_app::empty_mod$"))
        .with_severity(Severity::Warn)
        .must_not_be_empty()
        .build();
        
    // Empty module rule with empty mod file rule - MUST be empty
    builder
        .module_lint()
        .lint_named("must_be_empty_module")
        .matching(|m| m.module("^test_app::must_be_empty$"))
        .with_severity(Severity::Warn)
        .must_be_empty()
        .build();
        
    // Module must have an empty mod.rs file (only allowed to re-export)
    builder
        .module_lint()
        .lint_named("must_have_empty_mod_file")
        .matching(|m| m.module("^test_app::empty_mod_file$"))
        .with_severity(Severity::Warn)
        .must_have_empty_mod_file()
        .build();

    // Item type restrictions
    builder
        .module_lint()
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
        .function_lint()
        .lint_named("result_type_check")
        .matching(|m| m.in_module("^test_app::result_error$"))
        .with_severity(Severity::Warn)
        .enforce_error_trait_implementation()
        .build();

    // ------------------------------------------------------------------
    // Builder style lint rules (demonstrates consuming vs reference pattern)
    // ------------------------------------------------------------------

    builder
        .function_lint()
        .lint_named("builder_style_with_consuming_forbidden")
        .matching(|m| m.name_regex("^with_.*").and(m.returns_self()))
        .with_severity(Severity::Error)
        .must_not_exist()
        .build();

    builder
        .function_lint()
        .lint_named("builder_style_set_consuming_forbidden")
        .matching(|m| m.name_regex("^set_.*").and(m.returns_self()))
        .with_severity(Severity::Error)
        .must_not_exist()
        .build();

    // Write the configuration to pup.ron using the fixed write_to_file method
    builder
        .write_to_file("pup.ron")
        .expect("Failed to write pup.ron file");

    println!("Successfully created and verified pup.ron configuration");
}
