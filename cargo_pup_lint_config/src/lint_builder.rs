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
    use crate::module_lint::ModuleLint;

    #[test]
    fn test_write_to_file() {
        let mut builder = LintBuilder::new();

        builder.push(ConfiguredLint::Module(ModuleLint {
            name: "example_module".into(),
            matches: ModuleMatch::NamespaceEquals("bob".into()),
            rule: ModuleRule::MustBeNamed("bob".into()),
        }));

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        builder.write_to_file(temp_path).unwrap();

        assert!(temp_file.path().exists());
    }

    #[test]
    fn test_read_from_file() {
        let mut builder = LintBuilder::new();

        builder.push(ConfiguredLint::Module(ModuleLint {
            name: "example_module".into(),
            matches: ModuleMatch::NamespaceEquals("bob".into()),
            rule: ModuleRule::MustBeNamed("bob".into()),
        }));

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        builder.write_to_file(temp_path).unwrap();

        let loaded_builder = LintBuilder::read_from_file(temp_path).unwrap();

        // Verify that the loaded builder contains the correct data
        assert_eq!(loaded_builder.lints.len(), 1);
        if let ConfiguredLint::Module(module_lint) = &loaded_builder.lints[0] {
            assert_eq!(module_lint.name, "example_module");
            if let ModuleMatch::NamespaceEquals(namespace) = &module_lint.matches {
                assert_eq!(namespace, "bob");
            }
            if let ModuleRule::MustBeNamed(name) = &module_lint.rule {
                assert_eq!(name, "bob");
            }
        } else {
            panic!("Unexpected lint type");
        }
    }
}