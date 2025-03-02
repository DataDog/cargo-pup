use crate::lints::empty_mod::EmptyModLintFactory;
use crate::lints::module_usage::ModuleUsageLintFactory;
use crate::utils::configuration_factory::LintFactory;

use crate::lints::{
    ArchitectureLintRule, function_length::FunctionLengthLintFactory,
    trait_impl::TraitImplLintFactory,
};

///
/// Collects a set of architecture lints configured
/// and ready to run.
///
///
pub struct ArchitectureLintCollection {
    lints: Vec<Box<dyn ArchitectureLintRule + Send>>,
}

impl ArchitectureLintCollection {
    pub fn new(lints: Vec<Box<dyn ArchitectureLintRule + Send>>) -> ArchitectureLintCollection {
        ArchitectureLintCollection { lints }
    }

    pub fn lints(&self) -> &Vec<Box<dyn ArchitectureLintRule + Send>> {
        &self.lints
    }
}

///
/// Should be called once at startup to register
/// all the lints with the configuration factory.
pub fn register_all_lints() {
    FunctionLengthLintFactory::register();
    TraitImplLintFactory::register();
    EmptyModLintFactory::register();
    ModuleUsageLintFactory::register();
}
