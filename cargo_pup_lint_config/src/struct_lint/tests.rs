#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_builder::LintBuilder;
    use crate::{Severity, StructLintExt, StructRule};
    use crate::ConfiguredLint;
    
    #[test]
    fn test_struct_visibility_rules() {
        let mut builder = LintBuilder::new();
        
        // Test both visibility rules
        builder.struct_lint()
            .lint_named("struct_lint")
            .matching(|m| m.name("UserModel"))
            .with_severity(Severity::Error)
            .must_be_private() // First rule
            .build();
            
        builder.struct_lint()
            .lint_named("struct_lint_2")
            .matching(|m| m.name("PublicAPI"))
            .with_severity(Severity::Warn)
            .must_be_public() // Second rule
            .build();
        
        assert_eq!(builder.lints.len(), 2);
        
        // Check private rule
        if let ConfiguredLint::Struct(struct_lint) = &builder.lints[0] {
            assert_eq!(struct_lint.rules.len(), 1);
            if let StructRule::MustBePrivate(severity) = &struct_lint.rules[0] {
                assert_eq!(severity, &Severity::Error);
            } else {
                panic!("Expected MustBePrivate rule");
            }
        } else {
            panic!("Expected Struct lint type");
        }
        
        // Check public rule
        if let ConfiguredLint::Struct(struct_lint) = &builder.lints[1] {
            assert_eq!(struct_lint.rules.len(), 1);
            if let StructRule::MustBePublic(severity) = &struct_lint.rules[0] {
                assert_eq!(severity, &Severity::Warn);
            } else {
                panic!("Expected MustBePublic rule");
            }
        } else {
            panic!("Expected Struct lint type");
        }
    }
}

#[cfg(test)]
mod context_generation_tests {
    use cargo_pup_common::project_context::{ProjectContext, TraitInfo, ModuleInfo};
    use crate::GenerateFromContext;
    use crate::lint_builder::LintBuilder;
    use crate::ConfiguredLint;
    use crate::struct_lint::StructLint;
    
    #[test]
    fn test_struct_lint_generation_from_contexts() {
        // Create test ProjectContexts
        let mut context1 = ProjectContext::new();
        context1.module_root = "crate1".to_string();
        
        // Add some traits with implementors to first context
        context1.traits = vec![
            TraitInfo {
                name: "crate1::MyTrait".to_string(),
                implementors: vec!["crate1::MyImpl".to_string()],
                applicable_lints: vec![],
            },
        ];
        
        // Create a second context with different traits
        let mut context2 = ProjectContext::new();
        context2.module_root = "crate2".to_string();
        context2.traits = vec![
            TraitInfo {
                name: "crate2::interfaces::Handler".to_string(),
                implementors: vec![
                    "crate2::handlers::DefaultHandler".to_string(),
                    "crate2::handlers::CustomHandler".to_string(),
                ],
                applicable_lints: vec![],
            },
        ];
        
        // Create a context array with both contexts
        let contexts = vec![context1, context2];
        
        // Create a builder to collect generated lints
        let mut builder = LintBuilder::new();
        
        // Generate struct lints from multiple contexts
        StructLint::generate_from_contexts(&contexts, &mut builder);
        
        // Check that we generated the expected number of lints
        // Should be 3: one for each trait plus the generic naming convention
        assert_eq!(builder.lints.len(), 3, "Should generate 3 struct lints");
        
        // Verify the generated lints
        let mut trait_impl_lint_count = 0;
        let mut naming_convention_lint_count = 0;
        
        for lint in builder.lints {
            if let ConfiguredLint::Struct(struct_lint) = lint {
                match struct_lint.name.as_str() {
                    "mytrait_implementors" => {
                        trait_impl_lint_count += 1;
                        
                        // Verify it has the right matcher
                        if let crate::struct_lint::StructMatch::ImplementsTrait(trait_name) = &struct_lint.matches {
                            assert_eq!(trait_name, "crate1::MyTrait");
                        } else {
                            panic!("Expected ImplementsTrait matcher");
                        }
                        
                        // Check the rules
                        assert_eq!(struct_lint.rules.len(), 2);
                    },
                    "handler_implementors" => {
                        trait_impl_lint_count += 1;
                        
                        // Verify it has the right matcher
                        if let crate::struct_lint::StructMatch::ImplementsTrait(trait_name) = &struct_lint.matches {
                            assert_eq!(trait_name, "crate2::interfaces::Handler");
                        } else {
                            panic!("Expected ImplementsTrait matcher");
                        }
                        
                        // Check the rules
                        assert_eq!(struct_lint.rules.len(), 2);
                    },
                    "struct_naming_convention" => {
                        naming_convention_lint_count += 1;
                        
                        // Verify it has the right matcher
                        if let crate::struct_lint::StructMatch::Name(pattern) = &struct_lint.matches {
                            assert_eq!(pattern, ".*");
                        } else {
                            panic!("Expected Name matcher");
                        }
                        
                        // Check the rules
                        assert_eq!(struct_lint.rules.len(), 1);
                    },
                    _ => {
                        panic!("Unexpected lint name: {}", struct_lint.name);
                    }
                }
            } else {
                panic!("Expected StructLint");
            }
        }
        
        // Verify counts
        assert_eq!(trait_impl_lint_count, 2, "Should generate 2 trait implementation lints");
        assert_eq!(naming_convention_lint_count, 1, "Should generate 1 naming convention lint");
    }
    
    #[test]
    fn test_duplicate_trait_handling() {
        // Create two contexts with the same trait
        let mut context1 = ProjectContext::new();
        context1.module_root = "crate1".to_string();
        context1.traits = vec![
            TraitInfo {
                name: "common::MyTrait".to_string(),
                implementors: vec!["crate1::MyImpl".to_string()],
                applicable_lints: vec![],
            },
        ];
        
        let mut context2 = ProjectContext::new();
        context2.module_root = "crate2".to_string();
        context2.traits = vec![
            TraitInfo {
                name: "common::MyTrait".to_string(), // Same trait in both contexts
                implementors: vec!["crate2::MyImpl".to_string()],
                applicable_lints: vec![],
            },
        ];
        
        let contexts = vec![context1, context2];
        
        // Create a builder to collect generated lints
        let mut builder = LintBuilder::new();
        
        // Generate struct lints
        StructLint::generate_from_contexts(&contexts, &mut builder);
        
        // Check that we generated only 2 lints (1 for the trait, 1 for naming convention)
        // We shouldn't have duplicate lints for the same trait
        assert_eq!(builder.lints.len(), 2, "Should generate 2 struct lints without duplicates");
        
        // Verify the trait lint has the right trait name
        if let ConfiguredLint::Struct(struct_lint) = &builder.lints[0] {
            assert_eq!(struct_lint.name, "mytrait_implementors");
            
            if let crate::struct_lint::StructMatch::ImplementsTrait(trait_name) = &struct_lint.matches {
                assert_eq!(trait_name, "common::MyTrait");
            } else {
                panic!("Expected ImplementsTrait matcher");
            }
        }
    }
    
    #[test]
    fn test_empty_contexts_minimal_generation() {
        // Create an empty contexts array
        let contexts: Vec<ProjectContext> = Vec::new();
        
        // Create a builder to collect generated lints
        let mut builder = LintBuilder::new();
        
        // Generate struct lints
        StructLint::generate_from_contexts(&contexts, &mut builder);
        
        // Check that we didn't generate any lints since the contexts array is empty
        assert_eq!(builder.lints.len(), 0, "Should not generate lints for empty contexts");
    }
}