// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

#[cfg(test)]
mod tests {
    use crate::ConfiguredLint;
    use crate::lint_builder::LintBuilder;
    use crate::{Severity, StructLintExt, StructRule, StructMatch};

    // Helper function to verify default severity
    fn assert_default_severity(severity: &Severity) {
        assert_eq!(severity, &Severity::Warn, "Default severity should be Warn");
    }
    
    #[test]
    fn test_regex_struct_matcher() {
        let mut builder = LintBuilder::new();

        // Test regex capabilities for struct matching
        builder
            .struct_lint()
            .lint_named("struct_lint")
            .matching(|m| {
                m.name("^[A-Z][a-z]+Model$")
                    .and(m.has_attribute("derive\\\\(.*Debug.*\\\\)"))
            })
            .with_severity(Severity::Error)
            .must_be_named("EntityModel".into())
            .build();

        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Struct(struct_lint) = &builder.lints[0] {
            // Check that the matcher is an AND with regex patterns
            if let StructMatch::AndMatches(left, right) = &struct_lint.matches {
                if let StructMatch::Name(pattern) = &**left {
                    assert_eq!(pattern, "^[A-Z][a-z]+Model$");
                } else {
                    panic!("Expected Name");
                }

                if let StructMatch::HasAttribute(pattern) = &**right {
                    assert_eq!(pattern, "derive\\\\(.*Debug.*\\\\)");
                } else {
                    panic!("Expected HasAttribute");
                }
            } else {
                panic!("Expected AndMatches");
            }

            // Check rule and severity
            assert_eq!(struct_lint.rules.len(), 1);
            if let StructRule::MustBeNamed(name, severity) = &struct_lint.rules[0] {
                assert_eq!(name, "EntityModel");
                assert_eq!(severity, &Severity::Error);
            } else {
                panic!("Expected MustBeNamed with Deny severity");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }
    
    #[test]
    fn test_struct_lint_builder() {
        let mut builder = LintBuilder::new();

        // Use the builder interface for struct lints with new matcher DSL
        builder
            .struct_lint()
            .lint_named("builder_int")
            .matching(|m| m.name("User"))
            .must_be_named("User".into())
            .must_not_be_named("UserStruct".into())
            .build();

        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Struct(struct_lint) = &builder.lints[0] {
            assert_eq!(struct_lint.name, "builder_int");

            if let StructMatch::Name(name) = &struct_lint.matches {
                assert_eq!(name, "User");
            } else {
                panic!("Expected Name match");
            }

            assert_eq!(struct_lint.rules.len(), 2);

            // Check first rule - MustBeNamed
            if let StructRule::MustBeNamed(name, severity) = &struct_lint.rules[0] {
                assert_eq!(name, "User");
                assert_default_severity(severity);
            } else {
                panic!("Expected MustBeNamed as first rule");
            }

            // Check second rule - MustNotBeNamed
            if let StructRule::MustNotBeNamed(name, severity) = &struct_lint.rules[1] {
                assert_eq!(name, "UserStruct");
                assert_default_severity(severity);
            } else {
                panic!("Expected MustNotBeNamed as second rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }
    
    #[test]
    fn test_complex_struct_matcher() {
        let mut builder = LintBuilder::new();

        // Test a complex matching expression for structs
        builder
            .struct_lint()
            .lint_named("complex_struct_matcher")
            .matching(|m| {
                m.name("User")
                    .or(m.name("Account"))
                    .and(m.has_attribute("derive(Debug)").not())
            })
            .must_be_named("Entity".into())
            .build();

        assert_eq!(builder.lints.len(), 1);
        
        // Simple type check only to verify the matcher was created and stored properly
        if let ConfiguredLint::Struct(struct_lint) = &builder.lints[0] {
            assert_eq!(struct_lint.name, "complex_struct_matcher");
            assert_eq!(struct_lint.rules.len(), 1);
        } else {
            panic!("Expected ConfiguredLint::Struct");
        }
    }

    #[test]
    fn test_struct_visibility_rules() {
        let mut builder = LintBuilder::new();

        // Test both visibility rules
        builder
            .struct_lint()
            .lint_named("struct_lint")
            .matching(|m| m.name("UserModel"))
            .with_severity(Severity::Error)
            .must_be_private() // First rule
            .build();

        builder
            .struct_lint()
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
    use crate::GenerateFromContext;
    use crate::lint_builder::LintBuilder;
    use crate::struct_lint::StructLint;
    use cargo_pup_common::project_context::{ProjectContext, TraitInfo};

    #[test]
    fn test_struct_lint_generation_from_contexts() {
        // Create test ProjectContexts
        let mut context1 = ProjectContext::new();
        context1.module_root = "crate1".to_string();

        // Add some traits with implementors to first context
        context1.traits = vec![TraitInfo {
            name: "crate1::MyTrait".to_string(),
            implementors: vec!["crate1::MyImpl".to_string()],
            applicable_lints: vec![],
        }];

        // Create a second context with different traits
        let mut context2 = ProjectContext::new();
        context2.module_root = "crate2".to_string();
        context2.traits = vec![TraitInfo {
            name: "crate2::interfaces::Handler".to_string(),
            implementors: vec![
                "crate2::handlers::DefaultHandler".to_string(),
                "crate2::handlers::CustomHandler".to_string(),
            ],
            applicable_lints: vec![],
        }];

        // Create a context array with both contexts
        let contexts = vec![context1, context2];

        // Create a builder to collect generated lints
        let mut builder = LintBuilder::new();

        // Generate struct lints from multiple contexts
        StructLint::generate_from_contexts(&contexts, &mut builder);

        // NOTE: The current implementation doesn't add any lints, this will be implemented later
        // For now, we simply verify that the method runs without errors
        
        // TODO: Update once the generate_from_contexts implementation is completed
        assert_eq!(builder.lints.len(), 0, "Current implementation adds no lints");
    }

    #[test]
    fn test_duplicate_trait_handling() {
        // Create two contexts with the same trait
        let mut context1 = ProjectContext::new();
        context1.module_root = "crate1".to_string();
        context1.traits = vec![TraitInfo {
            name: "common::MyTrait".to_string(),
            implementors: vec!["crate1::MyImpl".to_string()],
            applicable_lints: vec![],
        }];

        let mut context2 = ProjectContext::new();
        context2.module_root = "crate2".to_string();
        context2.traits = vec![TraitInfo {
            name: "common::MyTrait".to_string(), // Same trait in both contexts
            implementors: vec!["crate2::MyImpl".to_string()],
            applicable_lints: vec![],
        }];

        let contexts = vec![context1, context2];

        // Create a builder to collect generated lints
        let mut builder = LintBuilder::new();

        // Generate struct lints
        StructLint::generate_from_contexts(&contexts, &mut builder);

        // NOTE: The current implementation doesn't add any lints, this will be implemented later
        // For now, we simply verify that the method runs without errors
        
        // TODO: Update once the generate_from_contexts implementation is completed
        assert_eq!(
            builder.lints.len(),
            0,
            "Current implementation adds no lints"
        );
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
        assert_eq!(
            builder.lints.len(),
            0,
            "Should not generate lints for empty contexts"
        );
    }
}
