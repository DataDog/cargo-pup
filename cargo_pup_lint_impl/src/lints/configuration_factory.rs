use std::path::{Path};
use std::fs;
use crate::{ArchitectureLintRule};
use anyhow::Result;
use cargo_pup_common::project_context::ProjectContext;
use cargo_pup_lint_config::ConfiguredLint;
use cargo_pup_lint_config::lint_builder::LintBuilder;
use crate::lints::module_lint::ModuleLint;
use crate::lints::struct_lint::StructLint;
use crate::lints::function_lint::FunctionLint;
use ron;

// Supercedes the old LintConfigurationFactory
pub struct LintConfigurationFactory {
}

impl LintConfigurationFactory {

    pub fn from_file(file: String) -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        // Check if this is a file path or actual content
        let path = Path::new(&file);
        if path.exists() {
            if let Some(ext) = path.extension() {
                if ext == "ron" {
                    return Self::from_ron_file(file);
                }
            }
            
            // Default to existing implementation for other file types
            let lint_builder = LintBuilder::read_from_file(file)?;
            Ok(lint_builder.lints.iter().map(|l| {
                match l {
                    ConfiguredLint::Module(_) => ModuleLint::new(l),
                    ConfiguredLint::Struct(_) => StructLint::new(l),
                    ConfiguredLint::Function(_) => FunctionLint::new(l),
                }
            }).collect())
        } else {
            // Try parsing as content (assume RON first, then fall back)
            Self::from_content(&file)
        }
    }
    
    // New method for RON files
    fn from_ron_file(file: String) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        // Read file contents
        let content = fs::read_to_string(&file)
            .map_err(|e| anyhow::anyhow!("Failed to read RON file {}: {}", file, e))?;

        Self::from_content(&content)
    }
    
    // Process content regardless of source
    fn from_content(content: &str) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        // Try parsing as RON first
        match ron::from_str::<LintBuilder>(content) {
            Ok(lint_builder) => {
                // Successfully parsed as RON
                Ok(lint_builder.lints.iter().map(|l| {
                    match l {
                        ConfiguredLint::Module(_) => ModuleLint::new(l),
                        ConfiguredLint::Struct(_) => StructLint::new(l),
                        ConfiguredLint::Function(_) => FunctionLint::new(l),
                        // For now, only handle the lints we've already implemented in the new system
                    }
                }).collect())
            },
            Err(_e) => {
                // Extract an error line preview if possible
                let error_preview = match content.lines().enumerate().take(10).map(|(i, line)| format!("{}: {}", i+1, line)).collect::<Vec<_>>() {
                    lines if !lines.is_empty() => format!("\nFirst few lines of the file:\n{}", lines.join("\n")),
                    _ => String::new()
                };
                Result::Err(anyhow::anyhow!("Failed to parse RON file: {}", error_preview))
            }
        }
    }

    pub fn generate_file(_context: &ProjectContext) -> Result<String> {
        panic!("Not implemented!");
    }
}