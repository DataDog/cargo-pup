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
        to_writer_pretty(file, &self, PrettyConfig::default())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }

    // Method to read the LintBuilder from a file
    pub fn read_from_file<P: AsRef<std::path::Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Map any io::Error

        let lint_builder: LintBuilder = from_reader(file)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Map ron::de::SpannedError to io::Error

        Ok(lint_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use crate::{ConfiguredLint, ModuleMatch, ModuleRule, module_matcher, struct_matcher, function_matcher, Severity};
    use crate::module_lint::{ModuleLint, ModuleLintExt};
    use crate::struct_lint::{StructMatch, StructRule, StructLintExt};
    use crate::function_lint::{FunctionMatch, FunctionRule, FunctionLintExt};

    #[test]
    fn test_write_to_file() {
        let mut builder = LintBuilder::new();

        // Use a more complex matcher to demonstrate the DSL capabilities with regex
        builder.module()
            .matching(|m| 
                m.module("^core::(models|entities)$")
                    .or(
                        m.module("api::controllers")
                    )
            )
            .with_severity(Severity::Error)
            .must_be_named("domain_entity".into())
            .with_severity(Severity::Warn)
            .no_wildcard_imports()
            .build();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        builder.write_to_file(temp_path).unwrap();

        assert!(temp_file.path().exists());
    }

    #[test]
    fn test_read_from_file() {
        let mut builder = LintBuilder::new();

        // Use the same complex matcher as in test_write_to_file
        builder.module()
            .matching(|m| 
                m.module("^core::(models|entities)$")
                    .or(
                        m.module("api::controllers")
                    )
            )
            .with_severity(Severity::Error)
            .must_be_named("domain_entity".into())
            .with_severity(Severity::Warn)
            .no_wildcard_imports()
            .build();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        builder.write_to_file(temp_path).unwrap();

        let loaded_builder = LintBuilder::read_from_file(temp_path).unwrap();

        // Verify that the loaded builder contains the correct data
        assert_eq!(loaded_builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &loaded_builder.lints[0] {
            assert_eq!(module_lint.name, "module_lint");
            
            // Check that it's a complex matcher with OR at the top level
            if let ModuleMatch::OrMatches(left, right) = &module_lint.matches {
                // We don't check the entire structure, just that the serialization/deserialization worked
                
                // Check that the left side uses a regex module matcher
                if let ModuleMatch::Module(pattern) = &**left {
                    assert_eq!(pattern, "^core::(models|entities)$");
                } else {
                    panic!("Expected Module");
                }
            } else {
                panic!("Expected OrMatches at top level");
            }
            
            // Check rules and their severity levels
            assert_eq!(module_lint.rules.len(), 2);
            
            // First rule should be MustBeNamed with Deny severity
            if let ModuleRule::MustBeNamed(name, severity) = &module_lint.rules[0] {
                assert_eq!(name, "domain_entity");
                assert_eq!(severity, &Severity::Error);
            } else {
                panic!("Expected MustBeNamed with Deny severity");
            }
            
            // Second rule should be NoWildcardImports with Warn severity
            if let ModuleRule::NoWildcardImports(severity) = &module_lint.rules[1] {
                assert_eq!(severity, &Severity::Warn);
            } else {
                panic!("Expected NoWildcardImports with Warn severity");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }

    #[test]
    fn test_regex_struct_matcher() {
        let mut builder = LintBuilder::new();
        
        // Test regex capabilities for struct matching
        builder.struct_lint()
            .matching(|m| 
                m.name("^[A-Z][a-z]+Model$")
                    .and(m.has_attribute("derive\\(.*Debug.*\\)"))
            )
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
                    assert_eq!(pattern, "derive\\(.*Debug.*\\)");
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
    fn test_function_lint_max_length() {
        let mut builder = LintBuilder::new();
        
        // Test function lint with name matching and max length
        builder.function()
            .matching(|m| m.name("process_data"))
            .with_severity(Severity::Error)
            .max_length(50)
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Function(function_lint) = &builder.lints[0] {
            assert_eq!(function_lint.name, "function_lint");
            
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
        builder.function()
            .matching(|m| 
                m.name_regex("^(get|set)_[a-z_]+$")
                    .and(m.in_module("^core::models::[a-zA-Z]+$"))
            )
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
                assert_eq!(severity, &Severity::Warn); // Default severity
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
        builder.function()
            .matching(|m| m.name("test_and").and(m.name_regex(".*")))
            .with_severity(Severity::Error)
            .max_length(10)
            .build();
            
        // Test OR match
        builder.function()
            .matching(|m| m.name("test_or_1").or(m.name("test_or_2")))
            .with_severity(Severity::Error)
            .max_length(20)
            .build();
            
        // Test NOT match
        builder.function()
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
    fn test_must_not_be_empty_rule() {
        let mut builder = LintBuilder::new();
        
        // Test the builder extension method with new matcher DSL
        builder.module()
            .matching(|m| m.module("core::utils"))
            .must_not_be_empty()
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::MustNotBeEmpty(severity) = &module_lint.rules[0] {
                assert_eq!(severity, &Severity::Warn); // Default severity is Warn
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
        builder.module()
            .matching(|m| m.module("src/ui"))
            .no_wildcard_imports()
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::NoWildcardImports(severity) = &module_lint.rules[0] {
                assert_eq!(severity, &Severity::Warn); // Default severity is Warn
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
        builder.module()
            .matching(|m| m.module("app::core"))
            .restrict_imports(Some(allowed.clone()), Some(denied.clone()))
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &builder.lints[0] {
            assert_eq!(module_lint.rules.len(), 1);
            if let ModuleRule::RestrictImports { allowed_only, denied: denied_mods, severity } = &module_lint.rules[0] {
                assert_eq!(allowed_only.as_ref().unwrap(), &allowed);
                assert_eq!(denied_mods.as_ref().unwrap(), &denied);
                assert_eq!(severity, &Severity::Warn); // Default severity is Warn
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
                assert_eq!(severity, &Severity::Warn); // Default severity
            } else {
                panic!("Expected MustNotBeEmpty as first rule");
            }
            
            // Check second rule - NoWildcardImports
            if let ModuleRule::NoWildcardImports(severity) = &module_lint.rules[1] {
                assert_eq!(severity, &Severity::Warn); // Default severity
            } else {
                panic!("Expected NoWildcardImports as second rule");
            }
            
            // Check third rule - MustBeNamed
            if let ModuleRule::MustBeNamed(name, severity) = &module_lint.rules[2] {
                assert_eq!(name, "core");
                assert_eq!(severity, &Severity::Warn); // Default severity
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
        
        // Use the builder interface for struct lints with new matcher DSL
        builder.struct_lint()
            .matching(|m| m.name("User"))
            .must_be_named("User".into())
            .must_not_be_named("UserStruct".into())
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        if let ConfiguredLint::Struct(struct_lint) = &builder.lints[0] {
            assert_eq!(struct_lint.name, "struct_lint");
            
            if let StructMatch::Name(name) = &struct_lint.matches {
                assert_eq!(name, "User");
            } else {
                panic!("Expected Name match");
            }
            
            assert_eq!(struct_lint.rules.len(), 2);
            
            // Check first rule - MustBeNamed
            if let StructRule::MustBeNamed(name, severity) = &struct_lint.rules[0] {
                assert_eq!(name, "User");
                assert_eq!(severity, &Severity::Warn); // Default severity
            } else {
                panic!("Expected MustBeNamed as first rule");
            }
            
            // Check second rule - MustNotBeNamed
            if let StructRule::MustNotBeNamed(name, severity) = &struct_lint.rules[1] {
                assert_eq!(name, "UserStruct");
                assert_eq!(severity, &Severity::Warn); // Default severity
            } else {
                panic!("Expected MustNotBeNamed as second rule");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }

    #[test]
    fn test_complex_module_matcher() {
        let mut builder = LintBuilder::new();
        
        // Test a complex matching expression
        builder.module()
            .matching(|m| 
                m.module("app::core")
                    .or(m.module("lib::utils").not())
            )
            .must_not_be_empty()
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        // We're not checking the complex match structure in depth,
        // just that it builds successfully
    }
    
    #[test]
    fn test_complex_struct_matcher() {
        let mut builder = LintBuilder::new();
        
        // Test a complex matching expression for structs
        builder.struct_lint()
            .matching(|m| 
                m.name("User")
                    .or(m.name("Account"))
                    .and(m.has_attribute("derive(Debug)").not())
            )
            .must_be_named("Entity".into())
            .build();
            
        assert_eq!(builder.lints.len(), 1);
        // We're not checking the complex match structure in depth,
        // just that it builds successfully
    }

    /// Test that ensures the full LintBuilder structure is correctly serialized and deserialized,
    /// preserving the exact structure and format expected by cargo-pup.
    #[test]
    fn test_lint_builder_serialization_roundtrip() {
        // Create a builder with multiple different types of lint configurations
        let mut original_builder = LintBuilder::new();
        
        // Add a module lint
        original_builder.module()
            .matching(|m| m.module("^test::module$"))
            .with_severity(Severity::Warn)
            .must_not_be_empty()
            .build();
            
        // Add a struct lint
        original_builder.struct_lint()
            .matching(|m| m.name("TestStruct"))
            .with_severity(Severity::Error)
            .must_be_named("TestStruct".into())
            .build();
            
        // Add a function lint
        original_builder.function()
            .matching(|m| m.name_regex("^test_.*$"))
            .with_severity(Severity::Warn)
            .max_length(50)
            .build();
        
        // Serialize to RON
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        
        // Write to file
        original_builder.write_to_file(temp_path).unwrap();
        
        // Read it back
        let deserialized_builder = LintBuilder::read_from_file(temp_path).unwrap();
        
        // Verify the count of lints is the same
        assert_eq!(
            original_builder.lints.len(), 
            deserialized_builder.lints.len(),
            "Deserialized builder should have the same number of lints"
        );
        
        // Also directly test serialization and deserialization in memory (without file I/O)
        let serialized = ron::to_string(&original_builder).expect("Failed to serialize to RON string");
        let from_ron: LintBuilder = ron::from_str(&serialized).expect("Failed to deserialize from RON string");
        
        assert_eq!(
            original_builder.lints.len(), 
            from_ron.lints.len(),
            "In-memory deserialized builder should have the same number of lints"
        );
        
        // Print the serialized RON for debugging
        println!("Serialized RON format:\n{}", serialized);
        
        // Verify the types match for each lint after deserialization
        for (i, original_lint) in original_builder.lints.iter().enumerate() {
            let deserialized_lint = &deserialized_builder.lints[i];
            
            match (original_lint, deserialized_lint) {
                (ConfiguredLint::Module(_), ConfiguredLint::Module(_)) => {
                    // Both are module lints - correct
                },
                (ConfiguredLint::Struct(_), ConfiguredLint::Struct(_)) => {
                    // Both are struct lints - correct
                },
                (ConfiguredLint::Function(_), ConfiguredLint::Function(_)) => {
                    // Both are function lints - correct
                },
                _ => {
                    panic!("Lint type mismatch after deserialization at index {}", i);
                }
            }
        }
        
        // For additional verification, re-serialize the deserialized builder and compare the strings
        let reserialize1 = ron::to_string(&original_builder).expect("Failed to reserialize original");
        let reserialize2 = ron::to_string(&deserialized_builder).expect("Failed to serialize deserialized");
        
        assert_eq!(
            reserialize1, 
            reserialize2,
            "Reserialized RON strings should be identical"
        );
    }
}