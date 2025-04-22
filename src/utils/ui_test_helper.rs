use crate::lints::{ArchitectureLintCollection, ArchitectureLintRule};
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Extracts configuration from UI test files.
///
/// This function looks for special comments in the format:
/// //@ pup-config: |
/// //    rule_name:
/// //      type: lint_type
/// //      ...other parameters...
///
/// It then parses these comments and creates a lint configuration.
pub fn extract_config_from_test_file(file_path: &Path) -> anyhow::Result<ArchitectureLintCollection> {
    let content = fs::read_to_string(file_path)?;
    
    // Extract configuration blocks using regex - match both formats
    let config_pattern = Regex::new(r"// PUP-CONFIG:\s*\n((?://.*\n)+)")?;
    
    let mut yaml_config = String::new();
    
    if let Some(captures) = config_pattern.captures(&content) {
        if let Some(config_lines) = captures.get(1) {
            // Convert the comment lines to YAML by removing the comment markers
            let comment_regex = Regex::new(r"^//\s*")?;
            for line in config_lines.as_str().lines() {
                let clean_line = comment_regex.replace(line, "");
                yaml_config.push_str(&clean_line);
                yaml_config.push('\n');
            }
            println!("Extracted YAML config:\n{}", yaml_config);
        }
    } else {
        println!("No config pattern match in file: {:?}", file_path);
    }
    
    if yaml_config.is_empty() {
        // No configuration found, return an empty collection
        println!("Empty YAML config");
        return Ok(ArchitectureLintCollection::new(Vec::new()));
    }
    
    // Parse the YAML configuration
    match LintConfigurationFactory::from_yaml(yaml_config) {
        Ok(lint_rules) => {
            Ok(ArchitectureLintCollection::new(lint_rules))
        },
        Err(e) => {
            println!("Error parsing YAML: {:?}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    
    #[test]
    fn test_extract_config() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.rs");
        
        let test_content = r#"//@ pup-config: |
//    test_rule:
//      type: function_length
//      namespace: "^test$"
//      max_lines: 5
//      severity: Error

fn test_function() {
    // Test code
}
"#;
        
        let mut file = File::create(&file_path)?;
        file.write_all(test_content.as_bytes())?;
        
        // Register the lint factory before testing
        crate::lints::function_length::FunctionLengthLintFactory::register();
        
        let lint_collection = extract_config_from_test_file(&file_path)?;
        
        // We should have one lint rule
        assert_eq!(lint_collection.lints().len(), 1);
        
        Ok(())
    }
} 