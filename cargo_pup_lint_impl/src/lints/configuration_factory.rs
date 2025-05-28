// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use std::path::Path;
use crate::ArchitectureLintRule;
use anyhow::Result;
use cargo_pup_common::project_context::ProjectContext;
use cargo_pup_lint_config::ConfiguredLint;
use cargo_pup_lint_config::lint_builder::LintBuilder;
use crate::lints::module_lint::ModuleLint;
use crate::lints::struct_lint::StructLint;
use crate::lints::function_lint::FunctionLint;
use ron;

pub struct LintConfigurationFactory {
}

impl LintConfigurationFactory {

    pub fn from_file(file: String) -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        // Check if this is a file path or actual content
        let path = Path::new(&file);
        if path.exists() {
            // Use LintBuilder for file deserialization
            let lint_builder = LintBuilder::read_from_file(file)
                .map_err(|e| anyhow::anyhow!("Failed to read/parse lint file: {}", e))?;
            
            // Convert to architecture lint rules
            Self::from_lint_builder(lint_builder)
        } else {
            // Try parsing as direct content
            match ron::from_str::<LintBuilder>(&file) {
                Ok(lint_builder) => Self::from_lint_builder(lint_builder),
                Err(e) => {
                    // Extract an error line preview
                    let error_preview = match file.lines().enumerate().take(10).map(|(i, line)| format!("{}: {}", i+1, line)).collect::<Vec<_>>() {
                        lines if !lines.is_empty() => format!("\nFirst few lines of the content:\n{}", lines.join("\n")),
                        _ => String::new()
                    };
                    Err(anyhow::anyhow!("Failed to parse RON content: {}{}", e, error_preview))
                }
            }
        }
    }
    
    // Converts a LintBuilder to a collection of ArchitectureLintRules
    fn from_lint_builder(lint_builder: LintBuilder) -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        Ok(lint_builder.lints.iter().map(|l| {
            match l {
                ConfiguredLint::Module(_) => ModuleLint::new(l),
                ConfiguredLint::Struct(_) => StructLint::new(l),
                ConfiguredLint::Function(_) => FunctionLint::new(l),
            }
        }).collect())
    }

    pub fn generate_file(_context: &ProjectContext) -> Result<String> {
        panic!("Not implemented!");
    }
}
