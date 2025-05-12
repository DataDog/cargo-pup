use cargo_pup_lint_config::ConfiguredLint;
use crate::ArchitectureLintRule;

use rustc_lint::{LateContext, LateLintPass, Lint, LintStore};

pub mod configuration_factory;
mod module;

pub struct ModuleLint {
    config: ConfiguredLint::Module
}

impl ModuleLint {
    pub fn new(config: ConfiguredLint::Module) -> Self {
        Self {
            config
        }
    }
}

impl ArchitectureLintRule for ModuleLint {
    fn name(&self) -> String {
        todo!()
    }

    fn applies_to_module(&self, namespace: &str) -> bool {
        todo!()
    }

    fn applies_to_trait(&self, _trait_path: &str) -> bool {
        todo!()
    }

    fn register_late_pass(&self, _lint_store: &mut LintStore) {
        todo!()
    }
}