use ansi_term::Color;
use rustc_driver::Callbacks;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use serde::Deserialize;

use crate::utils::configuration_factory::LintFactory;

use super::{
    function_length::FunctionLengthLintFactory, namespace::NamespaceUsageLintFactory,
    trait_impl::TraitImplLintFactory, ArchitectureLintRule, LintResult,
};

///
/// Collects a set of architecture lints configured
/// and ready to run.
/// Provides an implementation of Callbacks and adapts the
/// rustc compiler expectations to what the lints need.
///
pub struct ArchitectureLintCollection {
    lints: Vec<Box<dyn ArchitectureLintRule + Send>>,
    lint_results: Vec<LintResult>,
    lint_results_text: String,
}

impl ArchitectureLintCollection {
    pub fn new(lints: Vec<Box<dyn ArchitectureLintRule + Send>>) -> ArchitectureLintCollection {
        ArchitectureLintCollection {
            lints,
            lint_results: vec![],
            lint_results_text: String::new(),
        }
    }

    ///
    /// Print out all the lint results to a string
    ///
    fn results_to_text(&self, source_map: &rustc_span::source_map::SourceMap) -> String {
        self.lint_results
            .iter()
            .map(|result| result.to_string(source_map))
            .collect::<Vec<String>>()
            .join("\n")
    }

    ///
    /// Returns all the lint results as a collection
    ///
    pub fn lint_results(&self) -> &Vec<LintResult> {
        &self.lint_results
    }

    pub fn to_string(&self) -> String {
        self.lint_results_text.clone()
    }
}

///
/// Adapt rustc's callbacks mechanism to our lints, collecting
/// lint results as we go.
///
impl Callbacks for ArchitectureLintCollection {
    fn after_expansion(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'_>,
    ) -> rustc_driver::Compilation {
        let lints = &self.lints; // Extract lints to avoid borrowing `self` later
        let lint_results = lints.iter().flat_map(|lint| lint.lint(tcx)); // Collect results
        self.lint_results.extend(lint_results); // Mutate `self.lint_results` after iteration

        let source_map = tcx.sess.source_map();
        self.lint_results_text = self.results_to_text(source_map);

        rustc_driver::Compilation::Continue
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
