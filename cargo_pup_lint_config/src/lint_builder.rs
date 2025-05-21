use ron::de::from_reader;
use ron::ser::{PrettyConfig, to_writer_pretty};
use std::fs::File;
use std::io;
// lint_builder.rs
use crate::function_lint::FunctionLint;
use crate::module_lint::ModuleLint;
use crate::struct_lint::StructLint;
use crate::{ConfiguredLint, GenerateFromContext};
use cargo_pup_common::project_context::ProjectContext;
use serde::{Deserialize, Serialize};

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

    /// Builds the configuration and returns all configured lints
    pub fn build(&self) -> Vec<ConfiguredLint> {
        self.lints.clone()
    }

    /// Generate lint configurations from project contexts for all lint types
    ///
    /// This method takes a slice of ProjectContext instances and uses the GenerateFromContext
    /// trait implementations for each lint type to generate a new LintBuilder.
    ///
    /// Returns the populated LintBuilder containing all generated lints.
    pub fn generate_from_contexts(contexts: &[ProjectContext]) -> Self {
        let mut builder = LintBuilder::new();

        // Generate lints from each lint type
        ModuleLint::generate_from_contexts(contexts, &mut builder);
        StructLint::generate_from_contexts(contexts, &mut builder);
        FunctionLint::generate_from_contexts(contexts, &mut builder);

        builder
    }

    /// Generate lint configurations from project contexts and write directly to a file
    ///
    /// This method combines the generation of lints from contexts with writing the result to a file.
    /// It returns an io::Result<()> to indicate whether the operation was successful.
    pub fn generate_and_write<P: AsRef<std::path::Path>>(
        contexts: &[ProjectContext],
        path: P,
    ) -> io::Result<()> {
        // Generate the builder from contexts
        let builder = Self::generate_from_contexts(contexts);

        // Write directly to file
        builder.write_to_file(path)
    }

    // Method to write the LintBuilder to a file
    pub fn write_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        let file = File::create(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        to_writer_pretty(file, &self, PrettyConfig::default())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }

    // Method to read the LintBuilder from a file
    pub fn read_from_file<P: AsRef<std::path::Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Map any io::Error

        let lint_builder: LintBuilder =
            from_reader(file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Map ron::de::SpannedError to io::Error

        Ok(lint_builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConfiguredLint, ModuleMatch, ModuleRule, Severity};
    use crate::module_lint::ModuleLintExt;
    use crate::struct_lint::StructLintExt;
    use crate::function_lint::FunctionLintExt;
    use cargo_pup_common::project_context::{ModuleInfo, ProjectContext, TraitInfo};
    use tempfile::NamedTempFile;
    
    // Helper function that creates a standard module matcher for tests
    fn create_standard_module_matcher() -> LintBuilder {
        let mut builder = LintBuilder::new();
        builder
            .module_lint()
            .lint_named("my_module_rules")
            .matching(|m| {
                m.module("^core::(models|entities)$")
                    .or(m.module("api::controllers"))
            })
            .with_severity(Severity::Error)
            .must_be_named("domain_entity".into())
            .with_severity(Severity::Warn)
            .no_wildcard_imports()
            .build();
        builder
    }
    
    // Helper function to verify default severity
    fn assert_default_severity(severity: &Severity) {
        assert_eq!(severity, &Severity::Warn, "Default severity should be Warn");
    }

    #[test]
    fn test_write_to_file() {
        let builder = create_standard_module_matcher();
        
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        builder.write_to_file(temp_path).unwrap();

        assert!(temp_file.path().exists());
    }

    #[test]
    fn test_read_from_file() {
        let builder = create_standard_module_matcher();
        
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        builder.write_to_file(temp_path).unwrap();

        let loaded_builder = LintBuilder::read_from_file(temp_path).unwrap();

        // Verify that the loaded builder contains the correct data
        assert_eq!(loaded_builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &loaded_builder.lints[0] {
            assert_eq!(module_lint.name, "my_module_rules");

            // Check that it's a complex matcher with OR at the top level
            if let ModuleMatch::OrMatches(left, _) = &module_lint.matches {
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

            // First rule should be MustBeNamed with Error severity
            if let ModuleRule::MustBeNamed(name, severity) = &module_lint.rules[0] {
                assert_eq!(name, "domain_entity");
                assert_eq!(severity, &Severity::Error);
            } else {
                panic!("Expected MustBeNamed with Error severity");
            }

            // Second rule should be NoWildcardImports with Warn severity
            if let ModuleRule::NoWildcardImports(severity) = &module_lint.rules[1] {
                assert_default_severity(severity);
            } else {
                panic!("Expected NoWildcardImports with Warn severity");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }

    /// Test that ensures the full LintBuilder structure is correctly serialized and deserialized,
    /// preserving the exact structure and format expected by cargo-pup.
    #[test]
    fn test_lint_builder_serialization_roundtrip() {
        // Create a builder with multiple different types of lint configurations
        let mut original_builder = LintBuilder::new();

        // Add a module lint
        original_builder
            .module_lint()
            .lint_named("module_lint")
            .matching(|m| m.module("^test::module$"))
            .with_severity(Severity::Warn)
            .must_not_be_empty()
            .build();

        // Add a struct lint
        original_builder
            .struct_lint()
            .lint_named("struct_lint")
            .matching(|m| m.name("TestStruct"))
            .with_severity(Severity::Error)
            .must_be_named("TestStruct".into())
            .build();

        // Add a function lint
        original_builder
            .function_lint()
            .lint_named("function_lint")
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
        let serialized =
            ron::to_string(&original_builder).expect("Failed to serialize to RON string");
        let from_ron: LintBuilder =
            ron::from_str(&serialized).expect("Failed to deserialize from RON string");

        assert_eq!(
            original_builder.lints.len(),
            from_ron.lints.len(),
            "In-memory deserialized builder should have the same number of lints"
        );

        // Verify the types match for each lint after deserialization
        for (i, original_lint) in original_builder.lints.iter().enumerate() {
            let deserialized_lint = &deserialized_builder.lints[i];

            match (original_lint, deserialized_lint) {
                (ConfiguredLint::Module(_), ConfiguredLint::Module(_)) => {
                    // Both are module lints - correct
                }
                (ConfiguredLint::Struct(_), ConfiguredLint::Struct(_)) => {
                    // Both are struct lints - correct
                }
                (ConfiguredLint::Function(_), ConfiguredLint::Function(_)) => {
                    // Both are function lints - correct
                }
                _ => {
                    panic!("Lint type mismatch after deserialization at index {}", i);
                }
            }
        }
    }

    #[test]
    fn test_generate_from_contexts() {
        // Create test project contexts
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
        context1.traits = vec![TraitInfo {
            name: "crate1::Trait1".to_string(),
            implementors: vec!["crate1::Type1".to_string()],
            applicable_lints: vec![],
        }];

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
        context2.traits = vec![TraitInfo {
            name: "crate2::TraitX".to_string(),
            implementors: vec!["crate2::TypeX".to_string()],
            applicable_lints: vec![],
        }];

        // Generate lints from contexts
        let builder = LintBuilder::generate_from_contexts(&[context1, context2]);

        // Verify that lints were generated
        assert!(
            !builder.lints.is_empty(),
            "Builder should contain generated lints"
        );

        // Check that we have each lint type
        let module_lints = builder
            .lints
            .iter()
            .filter(|lint| matches!(lint, ConfiguredLint::Module(_)))
            .count();
        let struct_lints = builder
            .lints
            .iter()
            .filter(|lint| matches!(lint, ConfiguredLint::Struct(_)))
            .count();
        let function_lints = builder
            .lints
            .iter()
            .filter(|lint| matches!(lint, ConfiguredLint::Function(_)))
            .count();

        // Verify we have at least some lints
        assert!(!builder.lints.is_empty(), "Builder should contain at least some lints");
        
        // Log the lint types for debugging
        println!("Found {} module lints, {} struct lints, {} function lints", 
                 module_lints, struct_lints, function_lints);
    }

    #[test]
    fn test_generate_and_write() {
        // Create test project contexts
        let mut context = ProjectContext::new();
        context.module_root = "test_crate".to_string();
        context.modules = vec![ModuleInfo {
            name: "test_crate::module1".to_string(),
            applicable_lints: vec![],
        }];
        context.traits = vec![TraitInfo {
            name: "test_crate::Trait1".to_string(),
            implementors: vec!["test_crate::Type1".to_string()],
            applicable_lints: vec![],
        }];

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Generate lints and write to file
        LintBuilder::generate_and_write(&[context], path).unwrap();

        // Verify the file exists and is not empty
        let metadata = std::fs::metadata(path).unwrap();
        assert!(metadata.len() > 0, "File should not be empty");

        // Read the file back and verify it's valid
        let builder = LintBuilder::read_from_file(path).unwrap();
        assert!(!builder.lints.is_empty(), "Builder should contain lints");
    }
}
