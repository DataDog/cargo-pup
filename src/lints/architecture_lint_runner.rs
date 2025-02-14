use std::collections::BTreeSet;

use ansi_term::Color;
use rustc_driver::Callbacks;
use rustc_hir::ItemKind;
use rustc_middle::ty::TyCtxt;

use super::{ArchitectureLintCollection, ArchitectureLintRule, LintResult};

///
/// The mode our lint runner should operate in
///
pub enum Mode {
    /// Run the lints
    Check,

    /// Print namespaces
    PrintNamespaces,
}

///
/// Runs architecture lints
///
pub struct ArchitectureLintRunner {
    mode: Mode,
    lint_collection: ArchitectureLintCollection,

    // Because we gather our output within the compiler
    // Callback mechanism, we need somewhere we can stash our
    // results internally.
    lint_results: Vec<LintResult>,
    result_text: String,
}

impl ArchitectureLintRunner {
    pub fn new(mode: Mode, lint_collection: ArchitectureLintCollection) -> Self {
        ArchitectureLintRunner {
            mode,
            lint_collection,
            lint_results: vec![],
            result_text: String::new(),
        }
    }

    ///
    /// Borrow the lint results.
    ///
    pub fn lint_results(&self) -> &Vec<LintResult> {
        #![allow(dead_code)]
        &self.lint_results
    }

    ///
    /// Borrow the lint results in formatted text style.
    ///
    pub fn lint_results_text(&self) -> &String {
        &self.result_text
    }

    //
    // Runs the lints!
    // This is the main action we can perform
    //
    fn check(
        &self,
        tcx: TyCtxt<'_>,
        lints: &Vec<Box<dyn ArchitectureLintRule + Send>>,
    ) -> (Vec<LintResult>, String) {
        let lint_results = lints.iter().flat_map(|lint| lint.lint(tcx)).collect();
        let source_map = tcx.sess.source_map();
        let lint_results_text = self.results_to_text(source_map, &lint_results);
        (lint_results, lint_results_text)
    }

    //
    // Prints the namespaces in the project.
    //
    fn print_namespaces(
        &self,
        tcx: TyCtxt<'_>,
        lints: &Vec<Box<dyn ArchitectureLintRule + Send>>,
    ) -> String {
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
            let applicable_lints: Vec<String> = lints
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

        output
    }

    ///
    /// Print out all the lint results to a string
    ///
    fn results_to_text(
        &self,
        source_map: &rustc_span::source_map::SourceMap,
        lint_results: &Vec<LintResult>,
    ) -> String {
        lint_results
            .iter()
            .map(|result| result.to_string(source_map))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Called back from the compiler
    fn callback(&mut self, tcx: TyCtxt<'_>) {
        match self.mode {
            Mode::Check => {
                let lints = self.lint_collection.lints();
                let (lint_results, lint_results_text) = self.check(tcx, lints);
                self.lint_results = lint_results;
                self.result_text = lint_results_text;
            }
            Mode::PrintNamespaces => {
                let lints = self.lint_collection.lints();
                self.result_text = self.print_namespaces(tcx, lints);
            }
        }
    }
}

///
/// Adapt rustc's callbacks mechanism to our lints, collecting
/// lint results as we go.
///
impl Callbacks for ArchitectureLintRunner {
    fn after_expansion(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'_>,
    ) -> rustc_driver::Compilation {
        self.callback(tcx);
        rustc_driver::Compilation::Continue
    }
}
