use std::fs::File;
use std::io;
use ron::de::from_reader;
use ron::ser::{to_writer_pretty, PrettyConfig};
// lint_builder.rs
use serde::{Deserialize, Serialize};
use crate::ConfiguredLint;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LintBuilder {
    pub lints: Vec<ConfiguredLint>,
}

impl LintBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, lint: ConfiguredLint) {
        self.lints.push(lint);
    }

    // Method to write the LintBuilder to a file
    pub fn write_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        let file = File::create(path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        to_writer_pretty(file, &self.lints, PrettyConfig::default())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }

    // Method to read the LintBuilder from a file
    pub fn read_from_file<P: AsRef<std::path::Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Map any io::Error

        let lints: Vec<ConfiguredLint> = from_reader(file)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Map ron::de::SpannedError to io::Error

        Ok(LintBuilder { lints })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use crate::{ConfiguredLint, ModuleMatch, ModuleRule};
    use crate::module_lint::{ModuleLint, ModuleLintExt};
    use crate::struct_lint::{StructMatch, StructRule, StructLintExt};

    #[test]
    fn test_write_to_file() {
        let mut builder = LintBuilder::new();

        builder.module()
            .matches(ModuleMatch::NamespaceEquals("bob".into()))
            .must_be_named("bob".into())
            .build();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        builder.write_to_file(temp_path).unwrap();

        assert!(temp_file.path().exists());
    }

    #[test]
    fn test_read_from_file() {
        let mut builder = LintBuilder::new();

        builder.module()
            .matches(ModuleMatch::NamespaceEquals("bob".into()))
            .must_be_named("bob".into())
            .build();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        builder.write_to_file(temp_path).unwrap();

        let loaded_builder = LintBuilder::read_from_file(temp_path).unwrap();

        // Verify that the loaded builder contains the correct data
        assert_eq!(loaded_builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &loaded_builder.lints[0] {
            assert_eq!(module_lint.name, "module_lint");
            if let ModuleMatch::NamespaceEquals(namespace) = &module_lint.matches {
                assert_eq!(namespace, "bob");
            }
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::MustBeNamed(name) = &module_lint.rules[0] {
                assert_eq!(name, "bob");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }

    #[test]
    fn test_must_not_be_empty_rule() {
        let mut builder = LintBuilder::new();
        
        // Test the builder extension method
        builder.module()
            .matches(ModuleMatch::NamespaceEquals("core::utils".into()))
            .must_not_be_empty()
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::MustNotBeEmpty = &module_lint.rules[0] {
                // Test passes if we can match the pattern
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
        
        // Test the builder extension method
        builder.module()
            .matches(ModuleMatch::PathContains("src/ui".into()))
            .no_wildcard_imports()
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::NoWildcardImports = &module_lint.rules[0] {
                // Test passes if we can match the pattern
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
        
        // Test the builder extension method
        builder.module()
            .matches(ModuleMatch::NamespaceEquals("app::core".into()))
            .restrict_imports(Some(allowed.clone()), Some(denied.clone()))
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::RestrictImports { allowed_only, denied: denied_mods } = &module_lint.rules[0] {
                assert_eq!(allowed_only.as_ref().unwrap(), &allowed);
                assert_eq!(denied_mods.as_ref().unwrap(), &denied);
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
        builder.module()
            .matches(ModuleMatch::NamespaceEquals("app::core".into()))
            .must_not_be_empty()
            .no_wildcard_imports()
            .must_be_named("core".into())
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 3);
            
            // Check first rule - MustNotBeEmpty
            if let ModuleRule::MustNotBeEmpty = &module_lint.rules[0] {
                // First rule is correct
            } else {
                panic!("Expected MustNotBeEmpty as first rule");
            }
            
            // Check second rule - NoWildcardImports
            if let ModuleRule::NoWildcardImports = &module_lint.rules[1] {
                // Second rule is correct
            } else {
                panic!("Expected NoWildcardImports as second rule");
            }
            
            // Check third rule - MustBeNamed
            if let ModuleRule::MustBeNamed(name) = &module_lint.rules[2] {
                assert_eq!(name, "core");
            } else {
                panic!("Expected MustBeNamed as third rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }

    #[test]
    fn test_struct_lint_builder() {
        let mut builder = LintBuilder::new();
        
        // Use the builder interface for struct lints
        builder.struct_lint()
            .matches(StructMatch::NameEquals("User".into()))
            .must_be_named("User".into())
            .must_not_be_named("UserStruct".into())
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Struct(struct_lint) = &builder.lints[0] {
            assert_eq!(struct_lint.name, "struct_lint");
            
            if let StructMatch::NameEquals(name) = &struct_lint.matches {
                assert_eq!(name, "User");
            } else {
                panic!("Expected NameEquals match");
            }
            
            assert_eq!(struct_lint.rules.len(), 2);
            
            // Check first rule - MustBeNamed
            if let StructRule::MustBeNamed(name) = &struct_lint.rules[0] {
                assert_eq!(name, "User");
            } else {
                panic!("Expected MustBeNamed as first rule");
            }
            
            // Check second rule - MustNotBeNamed
            if let StructRule::MustNotBeNamed(name) = &struct_lint.rules[1] {
                assert_eq!(name, "UserStruct");
            } else {
                panic!("Expected MustNotBeNamed as second rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }
}