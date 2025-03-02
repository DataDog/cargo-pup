use ansi_term::Color;
use rustc_driver::Callbacks;
use rustc_hir::ItemKind;
use rustc_hir::def_id::LocalModDefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;
use std::{collections::BTreeSet, path::Path};

use crate::utils::configuration_factory::setup_lints_yaml;

use crate::lints::{ArchitectureLintCollection, ArchitectureLintRule};

///
/// The mode our lint runner should operate in
///
#[derive(Clone)]
pub enum Mode {
    /// Run the lints
    Check,

    /// Print modules
    PrintModules,

    /// Print traits
    PrintTraits,
}

///
/// Runs architecture lints. Can run lints in a couple of different
/// modes - either actual linting, or diagnostic modes that print
/// out things like the namespace tree within the cate cargo-pup
/// is being run on.
///
pub struct ArchitectureLintRunner {
    mode: Mode,
    lint_collection: ArchitectureLintCollection,

    // Arguments to the cargo-pup. We need these
    // so that we can tie the results of the session
    // back to them, and invalidate the cache when they
    // change.
    cli_args: String,

    // Because we gather our output within the compiler
    // Callback mechanism, we need somewhere we can stash our
    // results internally.
    result_text: String,
}

impl ArchitectureLintRunner {
    pub fn new(mode: Mode, cli_args: String, lint_collection: ArchitectureLintCollection) -> Self {
        ArchitectureLintRunner {
            mode,
            lint_collection,
            result_text: String::new(),
            cli_args,
        }
    }

    ///
    /// Borrow the lint results in formatted text style.
    ///
    pub fn lint_results_text(&self) -> &String {
        &self.result_text
    }

    ///
    /// Prints traits in the project.
    /// TODO - this is broken since we upgraded the rust compiler! 
    ///
    fn print_traits(
        &self,
        tcx: TyCtxt<'_>,
        _lints: &[Box<dyn ArchitectureLintRule + Send>],
    ) -> String {
        let mut trait_set: BTreeSet<(String, String)> = BTreeSet::new();

        let (module, _, _) = tcx.hir_get_module(LocalModDefId::CRATE_DEF_ID);
        for id in module.item_ids {
            let item = tcx.hir_item(*id);
            if let ItemKind::Trait(..) = item.kind {
                let trait_name = tcx.def_path_str(item.owner_id.to_def_id());
                let module = tcx
                    .crate_name(item.owner_id.to_def_id().krate)
                    .to_ident_string();
                trait_set.insert((module, trait_name));
            }
        }

        let mut output = String::new();
        for (module, trait_name) in &trait_set {
            output.push_str(&format!("{}::{}\n", Color::Blue.paint(module), trait_name));
        }

        output
    }

    //
    // Prints the modules in the project.
    //
    fn print_modules(
        &self,
        tcx: TyCtxt<'_>,
        lints: &Vec<Box<dyn ArchitectureLintRule + Send>>,
    ) -> String {
        let mut namespace_set: BTreeSet<(String, String)> = BTreeSet::new();
        let (module, _, _) = tcx.hir_get_module(LocalModDefId::CRATE_DEF_ID);

        for id in module.item_ids {
            let item = tcx.hir_item(*id);
            if let ItemKind::Mod(..) = item.kind {
                let namespace = tcx.def_path_str(item.owner_id.to_def_id());
                let module = tcx
                    .crate_name(item.owner_id.to_def_id().krate)
                    .to_ident_string();
                namespace_set.insert((module, namespace));
            }
        }

        let mut output = String::new();
        for (module, namespace) in &namespace_set {
            let applicable_lints: Vec<String> = lints
                .iter()
                .filter(|lint| lint.applies_to_module(namespace))
                .map(|lint| lint.name())
                .collect();

            output.push_str(&format!(
                "{}::{} [{}]\n",
                Color::Blue.paint(module),
                namespace,
                Color::Green.paint(applicable_lints.join(", "))
            ));
        }
        output
    }

    /// Called back from the compiler
    fn callback(&mut self, tcx: TyCtxt<'_>) {
        let lints = self.lint_collection.lints();
        match self.mode {
            Mode::Check => {
                // Do nothing. Checking happens as part of the lints.
            }
            Mode::PrintModules => {
                self.result_text = self.print_modules(tcx, lints);
            }
            Mode::PrintTraits => {
                self.result_text = self.print_traits(tcx, lints);
            }
        }
    }
}

///
/// Adapt rustc's callbacks mechanism to our lints, collecting
/// lint results as we go.
///
impl Callbacks for ArchitectureLintRunner {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        let cli_args = self.cli_args.clone();
        let mode = self.mode.clone();

        config.register_lints = Some(Box::new(move |_sess, lint_store| {
            // If we're actually linting, recreate the lints and add them all
            if let Mode::Check = mode {
                let lints = setup_lints_yaml().expect("can load lints");
                for lint in lints {
                    lint.register_late_pass(lint_store);
                }
            }
        }));

        config.psess_created = Some(Box::new(move |psess| {
            // track CLI args
            psess
                .env_depinfo
                .get_mut()
                .insert((Symbol::intern(""), Some(Symbol::intern(&cli_args))));

            // Track config file
            if Path::new("../../../pup.yaml").exists() {
                psess
                    .file_depinfo
                    .get_mut()
                    .insert(Symbol::intern("pup.yaml"));
            }

            // Add our test lint
        }));
    }

    ///
    /// This is where we are running our "manual" lints.
    /// E.g., ones that are not meeting the rust lint interface.
    ///
    fn after_expansion(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'_>,
    ) -> rustc_driver::Compilation {
        self.callback(tcx);
        rustc_driver::Compilation::Continue
    }

    ///
    /// We can use this to filter
    /// lint results, probably, if we need
    /// to do this dynamically (e.g., raising level if we cross some threshold).
    ///
    /// TODO - we should check this to make sure it works and we can keep it up our
    /// sleeve.
    ///
    fn after_analysis(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _tcx: TyCtxt<'_>,
    ) -> rustc_driver::Compilation {
        _compiler.sess.coverage_discard_all_spans_in_codegen();

        rustc_driver::Compilation::Continue
    }
}
