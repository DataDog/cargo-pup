// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

#[cfg(test)]
mod tests {
    use crate::ConfiguredLint;
    use crate::GenerateFromContext;
    use crate::LintBuilder;
    use crate::Severity;
    use crate::function_lint::{FunctionLint, FunctionLintExt, FunctionMatch, FunctionRule};
    use cargo_pup_common::project_context::{ModuleInfo, ProjectContext};

    // Helper function to verify default severity
    fn assert_default_severity(severity: &Severity) {
        assert_eq!(severity, &Severity::Warn, "Default severity should be Warn");
    }

    #[test]
    fn test_function_lint_max_length() {
        let mut builder = LintBuilder::new();

        // Test function lint with name matching and max length
        builder
            .function_lint()
            .lint_named("process_data_lint")
            .matching(|m| m.name("process_data"))
            .with_severity(Severity::Error)
            .max_length(50)
            .build();

        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Function(function_lint) = &builder.lints[0] {
            assert_eq!(function_lint.name, "process_data_lint");

            if let FunctionMatch::NameEquals(name) = &function_lint.matches {
                assert_eq!(name, "process_data");
            } else {
                panic!("Expected NameEquals match");
            }

            assert_eq!(function_lint.rules.len(), 1);
            if let FunctionRule::MaxLength(length, severity) = &function_lint.rules[0] {
                assert_eq!(*length, 50);
                assert_eq!(severity, &Severity::Error);
            } else {
                panic!("Expected MaxLength rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }

    #[test]
    fn test_function_lint_regex_matching() {
        let mut builder = LintBuilder::new();

        // Test function lint with regex matching
        builder
            .function_lint()
            .lint_named("regexp_lint")
            .matching(|m| {
                m.name_regex("^(get|set)_[a-z_]+$")
                    .and(m.in_module("^core::models::[a-zA-Z]+$"))
            })
            .max_length(30)
            .build();

        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Function(function_lint) = &builder.lints[0] {
            // Check that the matcher is an AND with regex patterns
            if let FunctionMatch::AndMatches(left, right) = &function_lint.matches {
                if let FunctionMatch::NameRegex(pattern) = &**left {
                    assert_eq!(pattern, "^(get|set)_[a-z_]+$");
                } else {
                    panic!("Expected NameRegex");
                }

                if let FunctionMatch::InModule(pattern) = &**right {
                    assert_eq!(pattern, "^core::models::[a-zA-Z]+$");
                } else {
                    panic!("Expected InModule");
                }
            } else {
                panic!("Expected AndMatches");
            }

            // Check rule
            assert_eq!(function_lint.rules.len(), 1);
            if let FunctionRule::MaxLength(length, severity) = &function_lint.rules[0] {
                assert_eq!(*length, 30);
                assert_default_severity(severity);
            } else {
                panic!("Expected MaxLength rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }

    #[test]
    fn test_function_lint_combined_rules() {
        let mut builder = LintBuilder::new();

        // Test AND match
        builder
            .function_lint()
            .lint_named("test_and")
            .matching(|m| m.name("test_and").and(m.name_regex(".*")))
            .with_severity(Severity::Error)
            .max_length(10)
            .build();

        // Test OR match
        builder
            .function_lint()
            .lint_named("test_or")
            .matching(|m| m.name("test_or_1").or(m.name("test_or_2")))
            .with_severity(Severity::Error)
            .max_length(20)
            .build();

        // Test NOT match
        builder
            .function_lint()
            .lint_named("test_not")
            .matching(|m| m.name_regex("test_.*").not())
            .with_severity(Severity::Error)
            .max_length(100)
            .build();

        assert_eq!(builder.lints.len(), 3);

        // Verify AND match
        if let ConfiguredLint::Function(function_lint) = &builder.lints[0] {
            if let FunctionMatch::AndMatches(left, right) = &function_lint.matches {
                if let FunctionMatch::NameEquals(name) = &**left {
                    assert_eq!(name, "test_and");
                } else {
                    panic!("Expected NameEquals on left side");
                }

                if let FunctionMatch::NameRegex(pattern) = &**right {
                    assert_eq!(pattern, ".*");
                } else {
                    panic!("Expected NameRegex on right side");
                }
            } else {
                panic!("Expected AndMatches");
            }

            // Verify rule is a simple MaxLength
            assert_eq!(function_lint.rules.len(), 1);
            if let FunctionRule::MaxLength(length, severity) = &function_lint.rules[0] {
                assert_eq!(*length, 10);
                assert_eq!(severity, &Severity::Error);
            } else {
                panic!("Expected MaxLength rule");
            }
        }

        // Verify OR match
        if let ConfiguredLint::Function(function_lint) = &builder.lints[1] {
            if let FunctionMatch::OrMatches(left, right) = &function_lint.matches {
                if let FunctionMatch::NameEquals(name) = &**left {
                    assert_eq!(name, "test_or_1");
                } else {
                    panic!("Expected NameEquals on left side");
                }

                if let FunctionMatch::NameEquals(name) = &**right {
                    assert_eq!(name, "test_or_2");
                } else {
                    panic!("Expected NameEquals on right side");
                }
            } else {
                panic!("Expected OrMatches");
            }

            // Verify rule is a simple MaxLength
            assert_eq!(function_lint.rules.len(), 1);
            if let FunctionRule::MaxLength(length, severity) = &function_lint.rules[0] {
                assert_eq!(*length, 20);
                assert_eq!(severity, &Severity::Error);
            } else {
                panic!("Expected MaxLength rule");
            }
        }

        // Verify NOT match
        if let ConfiguredLint::Function(function_lint) = &builder.lints[2] {
            if let FunctionMatch::NotMatch(inner) = &function_lint.matches {
                if let FunctionMatch::NameRegex(pattern) = &**inner {
                    assert_eq!(pattern, "test_.*");
                } else {
                    panic!("Expected NameRegex inside NOT");
                }
            } else {
                panic!("Expected NotMatch");
            }

            // Verify rule is a simple MaxLength
            assert_eq!(function_lint.rules.len(), 1);
            if let FunctionRule::MaxLength(length, severity) = &function_lint.rules[0] {
                assert_eq!(*length, 100);
                assert_eq!(severity, &Severity::Error);
            } else {
                panic!("Expected MaxLength rule");
            }
        }
    }

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

        // Should have 3 lints: 2 defaults + 1 for the project (test_crate)
        assert_eq!(function_lints.len(), 3);

        // Verify we have a lint for the project
        let has_project_lint = function_lints
            .iter()
            .any(|lint| lint.name.contains("test_crate"));
        assert!(
            has_project_lint,
            "Should have a lint for the test_crate project"
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

        // Should have 4 lints: 2 defaults + 1 for each project (crate1, crate2)
        assert_eq!(function_lints.len(), 4);

        // Verify we have a lint for each project
        let has_crate1_lint = function_lints
            .iter()
            .any(|lint| lint.name.contains("crate1"));
        let has_crate2_lint = function_lints
            .iter()
            .any(|lint| lint.name.contains("crate2"));

        assert!(has_crate1_lint, "Should have a lint for crate1");
        assert!(has_crate2_lint, "Should have a lint for crate2");

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
