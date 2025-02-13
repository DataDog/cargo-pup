use std::{collections::BTreeSet, fmt};

use ansi_term::Color;
use rustc_driver::Callbacks;
use rustc_hir::ItemKind;
use rustc_middle::ty::TyCtxt;

use crate::utils::configuration_factory::LintFactory;

use super::{
    ArchitectureLintRule, LintResult, function_length::FunctionLengthLintFactory,
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
    lint_results: Vec<LintResult>,
    lint_results_text: String,
    mode: Mode,
}

// The mode the lint collection should
// be configured into. This controls the
// action it performs when run through the
// compiler phase.
pub enum Mode {
    // Run the lints
    Check,

    // Print namespaces, with rule association listed alongside
    PrintNamespaces,
}

impl fmt::Display for ArchitectureLintCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lint_results_text)
    }
}

impl ArchitectureLintCollection {
    pub fn new(
        lints: Vec<Box<dyn ArchitectureLintRule + Send>>,
        mode: Mode,
    ) -> ArchitectureLintCollection {
        ArchitectureLintCollection {
            lints,
            lint_results: vec![],
            lint_results_text: String::new(),
            mode,
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
        // This shows up as dead because we only use it for tests.
        // It's not!
        #![allow(dead_code)]

        &self.lint_results
    }

    //
    // Runs the lints!
    // This is the main action we can perform
    //
    fn check(&mut self, tcx: TyCtxt<'_>) {
        let lints = &self.lints; // Extract lints to avoid borrowing `self` later
        let lint_results = lints.iter().flat_map(|lint| lint.lint(tcx)); // Collect results
        self.lint_results.extend(lint_results); // Mutate `self.lint_results` after iteration

        let source_map = tcx.sess.source_map();
        self.lint_results_text = self.results_to_text(source_map);
    }

    //
    // Prints the namespaces in the project.
    //
    fn print_namespaces(&mut self, tcx: TyCtxt<'_>) {
        let mut namespace_set: BTreeSet<String> = BTreeSet::new();

        for id in tcx.hir().items() {
            let item = tcx.hir().item(id);
            if let ItemKind::Mod(..) = item.kind {
                let namespace = tcx.def_path_str(item.owner_id.to_def_id());
                namespace_set.insert(namespace);
            }
        }

        let mut output = Color::Blue.bold().paint("Namespaces\n\n").to_string();
        for namespace in &namespace_set {
            let applicable_lints: Vec<String> = self
                .lints
                .iter()
                .filter(|lint| lint.applies_to_namespace(namespace))
                .map(|lint| lint.name())
                .collect();

            output.push_str(&format!(
                "{} [{}]\n",
                namespace,
                Color::Green.paint(applicable_lints.join(", "))
            ));
        }

        self.lint_results_text = output;
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
        match self.mode {
            Mode::Check => self.check(tcx),
            Mode::PrintNamespaces => self.print_namespaces(tcx),
        }
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
