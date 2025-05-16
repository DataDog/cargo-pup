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