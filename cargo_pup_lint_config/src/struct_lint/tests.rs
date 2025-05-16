#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint_builder::LintBuilder;
    use crate::Severity;
    
    // ... existing tests ...
    
    #[test]
    fn test_struct_visibility_rules() {
        let mut builder = LintBuilder::new();
        
        // Test both visibility rules
        builder.struct_lint()
            .matching(|m| m.name("UserModel"))
            .with_severity(Severity::Error)
            .must_be_private() // First rule
            .build();
            
        builder.struct_lint()
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