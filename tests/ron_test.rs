// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use cargo_pup_lint_config::{FunctionLintExt, LintBuilder, LintBuilderExt, ModuleLintExt, Severity, StructLintExt};



///
/// Uses cargo-pup to validate cargo-pup.
/// This both:
///
/// * emits the pup.ron file, so we can check it in
///   and run `cargo pup` as part of the CI job if we like
/// * uses the unit testing harness to run the tests inline
#[test]
fn validate_cargo_pup_structure() {
    // Create a new LintBuilder
    let mut builder = LintBuilder::new();

    // Add TraitImpl rule for architecture_lint_rules
    builder
        .struct_lint() 
        .lint_named("architecture_lint_rule_checker")
        .matching(|m| m.implements_trait("^pup_driver::lints::architecture_lint_rule::ArchitectureLintRule"))
        .with_severity(Severity::Error)
        .must_be_named(".*LintProcessor$".into())
        .must_be_private()
        .build();

    // Add TraitImpl rule for configuration_factories
    builder
        .struct_lint()
        .lint_named("lint_factory_checker")
        .matching(|m| m.implements_trait("^pup_driver::lints::configuration_factory.rs::LintFactory"))
        .with_severity(Severity::Error)
        .must_be_named(".*LintFactory$".into())
        .must_be_private()
        .build();

    // Add EmptyMod rule for modules following the mod.rs structure
    builder
        .module_lint()
        .lint_named("empty_mod_rule")
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Warn)
        .must_have_empty_mod_file()
        .build();

    // Add ItemType rule for helpers_no_structs_or_traits
    builder
        .module_lint()
        .lint_named("helpers_no_structs_or_traits")
        .matching(|m| m.module("^pup_driver::lints::helpers$"))
        .with_severity(Severity::Error)
        .denied_items(vec![
            "struct".to_string(),
            "trait".to_string(),
        ])
        .build();

    // Utils shouldn't contain structs or traits
    builder
        .module_lint()
        .lint_named("utils_no_structs_or_traits")
        .matching(|m| m.module("^pup_driver::utils$"))
        .with_severity(Severity::Error)
        .denied_items(vec![
            "struct".to_string(),
            "trait".to_string(),
        ])
        .build();

    // All Result<_,T> must return something implementing the error trait
    builder
        .function_lint()
        .lint_named("result_error_impl_rule")
        .matching(|m| m.in_module(".*"))
        .with_severity(Severity::Error)
        .enforce_error_trait_implementation()
        .build();

    // cargo_pup shouldn't use the lints subsystem
    builder
        .module_lint()
        .lint_named("cargo_pup_no_lints_usage")
        .matching(|m| m.module("^cargo_pup::"))
        .with_severity(Severity::Error)
        .restrict_imports(None, Some(vec!["::lints".to_string()]))
        .build();

    // Write the configuration to pup.ron using the write_to_file method
    builder
        .write_to_file("pup.ron")
        .expect("Failed to write pup.ron file");

    // Run it!
    builder.assert_lints(None).unwrap();

    println!("Successfully created and verified pup.ron configuration");
}

/// Test that validates our lint harness correctly detects rule violations.
/// 
/// This test creates a rule that we know will fail when run against cargo-pup itself,
/// then verifies that assert_lints properly panics when violations are detected.
#[test]
fn test_lint_harness_detects_violations() {
    let mut builder = LintBuilder::new();

    // Add a rule that will definitely fail - require all modules to be empty
    // This will fail because cargo-pup has many non-empty modules
    builder
        .module_lint()
        .lint_named("all_modules_must_be_empty")
        .matching(|m| m.module(".*"))
        .with_severity(Severity::Error)
        .must_be_empty()
        .build();

    // Use std::panic::catch_unwind to catch the expected panic
    let panic_result = std::panic::catch_unwind(|| {
        // This should panic because the rule will be violated
        builder.assert_lints(None).unwrap();
    });

    // Verify that the panic occurred (meaning our lint harness correctly detected violations)
    assert!(
        panic_result.is_err(),
        "Expected assert_lints to panic due to lint violations, but it didn't panic"
    );

    // Extract the panic message and verify it contains expected content
    let panic_payload = panic_result.unwrap_err();
    if let Some(panic_msg) = panic_payload.downcast_ref::<String>() {
        assert!(
            panic_msg.contains("cargo pup checks failed"),
            "Panic message should indicate cargo pup checks failed, got: {}",
            panic_msg
        );
    } else if let Some(panic_msg) = panic_payload.downcast_ref::<&str>() {
        assert!(
            panic_msg.contains("cargo pup checks failed"),
            "Panic message should indicate cargo pup checks failed, got: {}",
            panic_msg
        );
    } else {
        // If we can't extract the message, just verify that a panic occurred
        // which we already did above
    }

    println!("Successfully verified that lint harness detects violations and panics appropriately");
} 
