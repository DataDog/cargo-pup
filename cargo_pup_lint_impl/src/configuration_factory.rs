use std::{
    collections::HashMap,
    fs,
    io::Write,
    sync::{Arc, LazyLock, Mutex},
};

use crate::ArchitectureLintRule;
use cargo_pup_common::project_context::ProjectContext;
use anyhow::Result;

pub trait LintFactory: Send + Sync {
    ///
    /// Registers itself with the ConfigurationFactory
    ///
    fn register()
    where
        Self: Sized;
}

pub struct LintConfigurationFactory {
    factories: HashMap<String, Arc<dyn LintFactory>>,
}

// Define the static singleton instance
static INSTANCE: LazyLock<Mutex<LintConfigurationFactory>> =
    LazyLock::new(|| Mutex::new(LintConfigurationFactory::new()));

impl LintConfigurationFactory {

    /// Create a new, empty factory
    pub(crate) fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
}

// This function is kept for backward compatibility but is deprecated
pub fn setup_lints_yaml() -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
    // Call the setup_lints function which is now RON-based
    setup_lints()
}

pub fn setup_lints() -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
    use std::fs;
    
    // First try the current directory
    let ron_content = match fs::read_to_string("pup.ron") {
        Ok(content) => content,
        Err(_e) => {
            // If that fails, try the parent directory (in case we're in a subdirectory)
            match fs::read_to_string("../pup.ron") {
                Ok(content) => content,
                Err(_) => {
                    // If that fails too, try the original relative path
                    match fs::read_to_string("../../pup.ron") {
                        Ok(content) => content,
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                return Ok(Vec::new());
                            }
                            return Err(anyhow::Error::from(e));
                        }
                    }
                }
            }
        }
    };

    // Load the LintBuilder from RON
    let lint_builder = ron::from_str::<cargo_pup_lint_config::LintBuilder>(&ron_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse RON configuration: {}", e))?;
    
    // Convert to ArchitectureLintRules
    let lint_rules = Vec::new(); // TODO: Implement conversion from LintBuilder to ArchitectureLintRules

    Ok(lint_rules)
}
