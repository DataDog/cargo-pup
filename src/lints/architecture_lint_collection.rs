use crate::utils::configuration_factory::LintFactory;

use super::{
    ArchitectureLintRule, function_length::FunctionLengthLintFactory,
    namespace::NamespaceUsageLintFactory, trait_impl::TraitImplLintFactory,
};

///
/// Collects a set of architecture lints configured
/// and ready to run.
/// Provides an implementation of Callbacks and adapts the
/// rustc compiler expectations to what the lints need.
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
    NamespaceUsageLintFactory::register();
    FunctionLengthLintFactory::register();
    TraitImplLintFactory::register();
}
