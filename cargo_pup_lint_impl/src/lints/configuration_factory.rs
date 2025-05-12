use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use crate::{ArchitectureLintRule, LintFactory};
use anyhow::Result;
use cargo_pup_common::project_context::ProjectContext;
use cargo_pup_lint_config::ConfiguredLint;
use cargo_pup_lint_config::lint_builder::LintBuilder;
use crate::lints::module::module_lint::ModuleLint;

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

        let lint_builder = LintBuilder::read_from_file(file)?;
        Ok(lint_builder.lints.iter().map(|l| {
            match l {
                ConfiguredLint::Module(config) => ModuleLint::new(l),
                ConfiguredLint::Struct(_) => { panic!("Not implemented"); }
                ConfiguredLint::Function(_) => {panic!("Not implemented"); }
            }
        }).collect())
    }

    pub fn generate_file(context: &ProjectContext) -> Result<String> {
        panic!("Not implemented!");
    }
}