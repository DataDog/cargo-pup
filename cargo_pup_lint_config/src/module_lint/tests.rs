#[cfg(test)]
mod builder_tests {
    use crate::lint_builder::LintBuilder;
    use crate::ConfiguredLint;
    use crate::module_lint::{ModuleRule, ModuleMatch, ModuleLintExt};
    use crate::Severity;
    use tempfile::NamedTempFile;
    
    // Helper function to verify default severity
    fn assert_default_severity(severity: &Severity) {
        assert_eq!(severity, &Severity::Warn, "Default severity should be Warn");
    }
    
    #[test]
    fn test_must_not_be_empty_rule() {
        let mut builder = LintBuilder::new();

        // Test the builder extension method with new matcher DSL
        builder
            .module_lint()
            .lint_named("module_matcher")
            .matching(|m| m.module("core::utils"))
            .must_not_be_empty()
            .build();

        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::MustNotBeEmpty(severity) = &module_lint.rules[0] {
                assert_default_severity(severity);
            } else {
                panic!("Expected MustNotBeEmpty rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }
    
    #[test]
    fn test_no_wildcard_imports_rule() {
        let mut builder = LintBuilder::new();

        // Test the builder extension method with new matcher DSL
        builder
            .module_lint()
            .lint_named("wildcard_rule")
            .matching(|m| m.module("ui"))
            .no_wildcard_imports()
            .build();

        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::NoWildcardImports(severity) = &module_lint.rules[0] {
                assert_default_severity(severity);
            } else {
                panic!("Expected NoWildcardImports rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }
    
    #[test]
    fn test_restrict_imports_rule() {
        let mut builder = LintBuilder::new();
        let allowed = vec!["std::collections".into(), "crate::utils".into()];
        let denied = vec!["std::sync".into()];

        // Test the builder extension method with new matcher DSL
        builder
            .module_lint()
            .lint_named("test_restrict_imports")
            .matching(|m| m.module("app::core"))
            .restrict_imports(Some(allowed.clone()), Some(denied.clone()))
            .build();

        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::RestrictImports {
                allowed_only,
                denied: denied_mods,
                severity,
            } = &module_lint.rules[0]
            {
                assert_eq!(allowed_only.as_ref().unwrap(), &allowed);
                assert_eq!(denied_mods.as_ref().unwrap(), &denied);
                assert_default_severity(severity);
            } else {
                panic!("Expected RestrictImports rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }
    
    #[test]
    fn test_multiple_rules() {
        let mut builder = LintBuilder::new();

        // Apply multiple rules to the same module match
        builder
            .module_lint()
            .lint_named("multiple_matches")
            .matching(|m| m.module("app::core"))
            .must_not_be_empty()
            .no_wildcard_imports()
            .must_be_named("core".into())
            .build();

        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 3);

            // Check first rule - MustNotBeEmpty
            if let ModuleRule::MustNotBeEmpty(severity) = &module_lint.rules[0] {
                assert_default_severity(severity);
            } else {
                panic!("Expected MustNotBeEmpty as first rule");
            }

            // Check second rule - NoWildcardImports
            if let ModuleRule::NoWildcardImports(severity) = &module_lint.rules[1] {
                assert_default_severity(severity);
            } else {
                panic!("Expected NoWildcardImports as second rule");
            }

            // Check third rule - MustBeNamed
            if let ModuleRule::MustBeNamed(name, severity) = &module_lint.rules[2] {
                assert_eq!(name, "core");
                assert_default_severity(severity);
            } else {
                panic!("Expected MustBeNamed as third rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }
    
    #[test]
    fn test_complex_module_matcher() {
        let mut builder = LintBuilder::new();

        // Test a complex matching expression
        builder
            .module_lint()
            .lint_named("complex_module_matcher")
            .matching(|m| m.module("app::core").or(m.module("lib::utils").not()))
            .must_not_be_empty()
            .build();

        assert_eq!(builder.lints.len(), 1);
        assert!(matches!(&builder.lints[0], ConfiguredLint::Module(_)));
    }
}

#[cfg(test)]
mod context_generation_tests {
    use cargo_pup_common::project_context::{ProjectContext, ModuleInfo};
    use crate::GenerateFromContext;
    use crate::lint_builder::LintBuilder;
    use crate::ConfiguredLint;
    use crate::module_lint::{ModuleLint, ModuleRule, ModuleMatch};
    
    #[test]
    fn test_module_lint_generation_from_contexts() {
        // Create test ProjectContexts
        let mut context1 = ProjectContext::new();
        context1.module_root = "crate1".to_string();
        
        // Add some modules to first context
        context1.modules = vec![
            ModuleInfo {
                name: "crate1::module1".to_string(),
                applicable_lints: vec![],
            },
            ModuleInfo {
                name: "crate1::module2".to_string(),
                applicable_lints: vec![],
            },
        ];
        
        // Create a second context with different modules
        let mut context2 = ProjectContext::new();
        context2.module_root = "crate2".to_string();
        context2.modules = vec![
            ModuleInfo {
                name: "crate2::other_module".to_string(),
                applicable_lints: vec![],
            },
        ];
        
        // Create a context array with both contexts
        let contexts = vec![context1, context2];
        
        // Create a builder to collect generated lints
        let mut builder = LintBuilder::new();
        
        // Generate module lints from multiple contexts
        ModuleLint::generate_from_contexts(&contexts, &mut builder);
        
        // Verify we have the expected number of lints
        // Should be 3: empty_mod_rule, result_error_trait, and module_naming_convention
        assert_eq!(builder.lints.len(), 3, "Should generate 3 module lints");
        
        // Verify each lint
        let mut empty_mod_lint_found = false;
        let mut error_trait_lint_found = false;
        let mut naming_convention_lint_found = false;
        
        for lint in builder.lints {
            if let ConfiguredLint::Module(module_lint) = lint {
                match module_lint.name.as_str() {
                    "empty_mod_rule" => {
                        empty_mod_lint_found = true;
                        
                        // Verify it has the right matcher
                        if let ModuleMatch::Module(pattern) = &module_lint.matches {
                            assert!(pattern.contains("mod\\.rs"), "Should match mod.rs files");
                        } else {
                            panic!("Expected Module matcher");
                        }
                        
                        // Check the rule
                        assert_eq!(module_lint.rules.len(), 1);
                        match &module_lint.rules[0] {
                            ModuleRule::MustBeEmpty(severity) => {
                                assert_eq!(*severity, crate::Severity::Error);
                            },
                            _ => panic!("Expected MustBeEmpty rule"),
                        }
                    },
                    "result_error_trait" => {
                        error_trait_lint_found = true;
                        
                        // Verify it has the right matcher (wildcard)
                        if let ModuleMatch::Module(pattern) = &module_lint.matches {
                            assert_eq!(pattern, ".*", "Should match all modules");
                        } else {
                            panic!("Expected Module matcher");
                        }
                        
                        // Check the rule
                        assert_eq!(module_lint.rules.len(), 1);
                        match &module_lint.rules[0] {
                            ModuleRule::DeniedItems { items, severity } => {
                                assert_eq!(*severity, crate::Severity::Error);
                                assert!(!items.is_empty(), "Should have denied items");
                                assert!(items.iter().any(|item| item.contains("Result<*, i32>")), 
                                       "Should include primitive types like i32");
                            },
                            _ => panic!("Expected DeniedItems rule"),
                        }
                    },
                    "module_naming_convention" => {
                        naming_convention_lint_found = true;
                        
                        // Verify it has the right matcher (wildcard)
                        if let ModuleMatch::Module(pattern) = &module_lint.matches {
                            assert_eq!(pattern, ".*", "Should match all modules");
                        } else {
                            panic!("Expected Module matcher");
                        }
                        
                        // Check the rules
                        assert_eq!(module_lint.rules.len(), 2, "Should have two rules");
                        
                        // Check naming rule
                        let has_naming_rule = module_lint.rules.iter().any(|rule| {
                            matches!(rule, ModuleRule::MustBeNamed(pattern, _) if pattern.contains("^[a-z]"))
                        });
                        assert!(has_naming_rule, "Should include naming convention rule");
                        
                        // Check wildcard import rule
                        let has_wildcard_rule = module_lint.rules.iter().any(|rule| {
                            matches!(rule, ModuleRule::NoWildcardImports(_))
                        });
                        assert!(has_wildcard_rule, "Should include no wildcard imports rule");
                    },
                    _ => {
                        panic!("Unexpected lint name: {}", module_lint.name);
                    }
                }
            } else {
                panic!("Expected ModuleLint");
            }
        }
        
        // Verify all expected lints were found
        assert!(empty_mod_lint_found, "Should include empty mod.rs lint");
        assert!(error_trait_lint_found, "Should include result error trait lint");
        assert!(naming_convention_lint_found, "Should include module naming convention lint");
    }
    
    #[test]
    fn test_module_lint_generation_with_empty_contexts() {
        // Create an empty context array
        let contexts: Vec<ProjectContext> = Vec::new();
        
        // Create a builder to collect generated lints
        let mut builder = LintBuilder::new();
        
        // Generate module lints
        ModuleLint::generate_from_contexts(&contexts, &mut builder);
        
        // We should still get our two core lints that don't depend on context
        assert_eq!(builder.lints.len(), 2, "Should generate 2 lints even with empty contexts");
        
        // Verify the lints
        let lint_names: Vec<String> = builder.lints.iter().map(|lint| {
            if let ConfiguredLint::Module(module_lint) = lint {
                module_lint.name.clone()
            } else {
                panic!("Expected ModuleLint");
            }
        }).collect();
        
        assert!(lint_names.contains(&"empty_mod_rule".to_string()), "Should include empty mod.rs lint");
        assert!(lint_names.contains(&"result_error_trait".to_string()), "Should include result error trait lint");
    }
}