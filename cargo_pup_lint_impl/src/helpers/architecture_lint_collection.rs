use crate::empty_mod::EmptyModLintFactory;
use crate::module_usage::ModuleUsageLintFactory;
use crate::configuration_factory::LintFactory;

use crate::{
    ArchitectureLintRule, function_length::FunctionLengthLintFactory,
    item_type::ItemTypeLintFactory, result_error::ResultErrorLintFactory,
    trait_impl::TraitImplLintFactory,
};

///
/// Collects a set of architecture lints configured
/// and ready to run.
///
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
    ItemTypeLintFactory::register();
    ResultErrorLintFactory::register();
}
