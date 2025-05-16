#[cfg(test)]
mod tests {
    use crate::ConfiguredLint;
    use crate::GenerateFromContext;
    use crate::LintBuilder;
    use crate::function_lint::FunctionLint;
    use cargo_pup_common::project_context::{ModuleInfo, ProjectContext};

    #[test]
    fn test_function_lint_generate_from_empty_contexts() {
        // Test with empty contexts
        let contexts = Vec::<ProjectContext>::new();
        let mut builder = LintBuilder::new();

        // Generate lints from empty contexts
        FunctionLint::generate_from_contexts(&contexts, &mut builder);

        // We should still have our default lints even with empty contexts
        let lints = builder.build();

        // Filter function lints
        let function_lints: Vec<_> = lints
            .into_iter()
            .filter_map(|lint| match lint {
                ConfiguredLint::Function(f) => Some(f),
                _ => None,
            })
            .collect();

        // Verify we have exactly 2 function lints (the defaults)
        assert_eq!(function_lints.len(), 2);

        // Check that we have the function length lint
        let has_function_length_lint = function_lints
            .iter()
            .any(|lint| lint.name == "function_length_limit");
        assert!(has_function_length_lint, "Should have function length lint");

        // Check that we have the result error lint
        let has_result_error_lint = function_lints
            .iter()
            .any(|lint| lint.name == "result_error_must_implement_error");
        assert!(has_result_error_lint, "Should have result error lint");
    }

    #[test]
    fn test_function_lint_generate_from_contexts() {
        // Create a test context with some modules
        let mut context = ProjectContext::new();
        context.module_root = "test_crate".to_string();

        // Add some modules including nested ones
        let modules = vec![
            ModuleInfo {
                name: "test_crate::module1".to_string(),
                applicable_lints: vec![],
            },
            ModuleInfo {
                name: "test_crate::module2::submodule".to_string(),
                applicable_lints: vec![],
            },
            ModuleInfo {
                name: "test_crate::module3::submodule::subsubmodule".to_string(),
                applicable_lints: vec![],
            },
        ];
        context.modules = modules;

        // Create a builder and generate lints
        let mut builder = LintBuilder::new();
        FunctionLint::generate_from_contexts(&[context], &mut builder);

        // Get all lints
        let lints = builder.build();

        // Filter function lints
        let function_lints: Vec<_> = lints
            .into_iter()
            .filter_map(|lint| match lint {
                ConfiguredLint::Function(f) => Some(f),
                _ => None,
            })
            .collect();

        // We should have 3 lints: 2 defaults + 1 for the deeply nested module
        assert_eq!(function_lints.len(), 3);

        // Verify we have a lint for the nested module (the one with >2 segments)
        let has_nested_module_lint = function_lints.iter().any(|lint| {
            lint.name
                .contains("test_crate::module3::submodule::subsubmodule")
        });
        assert!(
            has_nested_module_lint,
            "Should have a lint for the deeply nested module"
        );
    }

    #[test]
    fn test_function_lint_with_multiple_contexts() {
        // Create two contexts
        let mut context1 = ProjectContext::new();
        context1.module_root = "crate1".to_string();
        context1.modules = vec![
            ModuleInfo {
                name: "crate1::module1".to_string(),
                applicable_lints: vec![],
            },
            ModuleInfo {
                name: "crate1::module2::submodule".to_string(),
                applicable_lints: vec![],
            },
        ];

        let mut context2 = ProjectContext::new();
        context2.module_root = "crate2".to_string();
        context2.modules = vec![
            ModuleInfo {
                name: "crate2::module1".to_string(),
                applicable_lints: vec![],
            },
            ModuleInfo {
                name: "crate2::module2::submodule::nested".to_string(),
                applicable_lints: vec![],
            },
        ];

        // Create a builder and generate lints from both contexts
        let mut builder = LintBuilder::new();
        FunctionLint::generate_from_contexts(&[context1, context2], &mut builder);

        // Get all lints
        let lints = builder.build();

        // Filter function lints
        let function_lints: Vec<_> = lints
            .into_iter()
            .filter_map(|lint| match lint {
                ConfiguredLint::Function(f) => Some(f),
                _ => None,
            })
            .collect();

        // We should have 3 lints: 2 defaults + 1 for the deeply nested module from context2
        assert_eq!(function_lints.len(), 3);

        // Verify we have a lint for the nested module from crate2
        let has_crate2_nested_module_lint = function_lints
            .iter()
            .any(|lint| lint.name.contains("crate2::module2::submodule::nested"));
        assert!(
            has_crate2_nested_module_lint,
            "Should have a lint for the deeply nested module from crate2"
        );

        // Verify we do not have a duplicate lint for any module
        let unique_names: std::collections::HashSet<_> =
            function_lints.iter().map(|lint| &lint.name).collect();
        assert_eq!(
            unique_names.len(),
            function_lints.len(),
            "Should not have duplicate lint names"
        );
    }
}
