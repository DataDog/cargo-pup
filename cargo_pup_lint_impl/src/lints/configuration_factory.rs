use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::path::{Path, PathBuf};
use std::fs;
use crate::{ArchitectureLintRule, LintFactory};
use anyhow::{Result, anyhow};
use cargo_pup_common::project_context::ProjectContext;
use cargo_pup_lint_config::ConfiguredLint;
use cargo_pup_lint_config::lint_builder::LintBuilder;
use crate::lints::module_lint::ModuleLint;
use crate::lints::struct_lint::StructLint;
use crate::lints::function_lint::FunctionLint;
use ron;
use serde_yaml;

// Supercedes the old LintConfigurationFactory
pub struct LintConfigurationFactory {
}

static INSTANCE: LazyLock<Mutex<crate::LintConfigurationFactory>> =
    LazyLock::new(|| Mutex::new(crate::LintConfigurationFactory::new()));

impl LintConfigurationFactory {
    /// Get a mutable reference to the global instance of the factory
    fn get_instance() -> std::sync::MutexGuard<'static, crate::LintConfigurationFactory> {
        INSTANCE
            .lock()
            .expect("Failed to lock the global LintConfigurationFactory")
    }

    /// Create a new factory
    pub fn new() -> Self {
        Self {}
    }

    pub fn from_file(file: String) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
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
                    _ => panic!("Unsupported lint type")
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
                        _ => panic!("Lint type not yet implemented in new system")
                    }
                }).collect())
            },
            Err(e) => {
                // If RON parsing fails, try as YAML
                match serde_yaml::from_str::<LintBuilder>(content) {
                    Ok(lint_builder) => {
                        Ok(lint_builder.lints.iter().map(|l| {
                            match l {
                                ConfiguredLint::Module(_) => ModuleLint::new(l),
                                ConfiguredLint::Struct(_) => StructLint::new(l),
                                ConfiguredLint::Function(_) => FunctionLint::new(l),
                                _ => panic!("Unsupported lint type")
                            }
                        }).collect())
                    },
                    Err(yaml_err) => {
                        // Neither format worked
                        Err(anyhow!("Failed to parse as RON: {}; Failed to parse as YAML: {}", e, yaml_err))
                    }
                }
            }
        }
    }

    pub fn generate_file(context: &ProjectContext) -> Result<String> {
        panic!("Not implemented!");
    }
}