use ansi_term::Color;
use rustc_driver::Callbacks;
use rustc_hir::ItemKind;
use rustc_hir::def_id::LocalModDefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;
use std::{collections::BTreeSet, path::Path};
use std::sync::Arc;

use crate::lints::{ArchitectureLintCollection, ArchitectureLintRule};

///
/// The mode our lint runner should operate in
///
#[derive(Clone, PartialEq)]
pub enum Mode {
    /// Run the lints
    Check,

    /// Print namespaces
    PrintModules,

    /// Print traits
    PrintTraits,

    /// Generate configuration
    GenerateConfig,
}

///
/// Runs architecture lints
///
pub struct ArchitectureLintRunner {
    mode: Mode,
    lint_collection: Arc<ArchitectureLintCollection>,

    // Arguments to the cargo-pup. We need these
    // so that we can tie the results of the session
    // back to them, and invalidate the cache when they
    // change.
    cli_args: String,

    // Cargo arguments that were passed through
    cargo_args: Vec<String>,

    // Because we gather our output within the compiler
    // Callback mechanism, we need somewhere we can stash our
    // results internally.
    result_text: String,
}

impl ArchitectureLintRunner {
    pub fn new(mode: Mode, cli_args: String, lint_collection: ArchitectureLintCollection) -> Self {
        ArchitectureLintRunner {
            mode,
            lint_collection: Arc::new(lint_collection),
            result_text: String::new(),
            cli_args,
            cargo_args: Vec::new(),
        }
    }

    /// Set cargo arguments that were passed through from the original command
    pub fn set_cargo_args(&mut self, args: Vec<String>) {
        self.cargo_args = args;
    }

    ///
    /// Borrow the lint results in formatted text style.
    ///
    pub fn lint_results_text(&self) -> &String {
        &self.result_text
    }

    ///
    /// Prints traits in the project.
    ///
    fn print_traits(
        &self,
        tcx: TyCtxt<'_>,
        _lints: &[Box<dyn ArchitectureLintRule + Send>],
    ) -> String {
        let mut trait_set: BTreeSet<(String, String)> = BTreeSet::new();

        let module_items = tcx.hir_crate_items(());
        for item_id in module_items.free_items() {
            let item = tcx.hir_item(item_id);
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
    // Prints the namespaces in the project.
    //
    fn print_namespaces(
        &self,
        tcx: TyCtxt<'_>,
        lints: &Vec<Box<dyn ArchitectureLintRule + Send>>,
    ) -> String {
        let mut namespace_set: BTreeSet<(String, String)> = BTreeSet::new();

        // Start recursive traversal from crate root
        collect_modules(tcx, LocalModDefId::CRATE_DEF_ID, &mut namespace_set);

        let mut output = String::new();
        for (module, path) in &namespace_set {
            let applicable_lints: Vec<String> = lints
                .iter()
                .filter(|lint| lint.applies_to_module(format!("{}::{}", module, path).as_str()))
                .map(|lint| lint.name())
                .collect();

            output.push_str(&format!(
                "{}::{} [{}]\n",
                Color::Blue.paint(module),
                path,
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
                self.result_text = self.print_namespaces(tcx, lints);
            }
            Mode::PrintTraits => {
                self.result_text = self.print_traits(tcx, lints);
            }
            Mode::GenerateConfig => self.generate_config(tcx),
        }
    }

    fn generate_config(&mut self, tcx: TyCtxt<'_>) {
        use crate::utils::config_generation::{GenerationContext, TraitInfo};
        use crate::utils::configuration_factory::LintConfigurationFactory;
        use std::collections::HashMap;
        
        // Create a GenerationContext with modules and traits info
        let mut namespace_set: BTreeSet<(String, String)> = BTreeSet::new();
        collect_modules(tcx, LocalModDefId::CRATE_DEF_ID, &mut namespace_set);
        
        // Collect all modules
        let modules: Vec<String> = namespace_set
            .iter()
            .map(|(module, path)| format!("{}::{}", module, path))
            .collect();
            
        // Get the current crate name (module root)
        // Just take the first entry's module name, which is the current crate
        let module_root = if let Some((crate_name, _)) = namespace_set.iter().next() {
            crate_name.clone()
        } else {
            "unknown_crate".to_string()
        };
        
        // Create filename with module root that we'll use later
        let config_filename = format!("pup.generated.{}.yaml", module_root);
            
        // Collect all traits and their implementors
        let mut trait_map: HashMap<String, Vec<String>> = HashMap::new();
        
        // Find all traits in the crate
        let module_items = tcx.hir_crate_items(());
        for item_id in module_items.free_items() {
            let item = tcx.hir_item(item_id);
            if let ItemKind::Trait(..) = item.kind {
                let trait_name = tcx.def_path_str(item.owner_id.to_def_id());
                let module = tcx
                    .crate_name(item.owner_id.to_def_id().krate)
                    .to_ident_string();
                let full_trait_name = format!("{}::{}", module, trait_name);
                
                // Initialize entry with empty vector
                trait_map.entry(full_trait_name).or_insert_with(Vec::new);
            }
        }
        
        // Find implementations
        for item_id in module_items.free_items() {
            let item = tcx.hir_item(item_id);
            if let ItemKind::Impl(impl_data) = &item.kind {
                if let Some(trait_ref) = impl_data.of_trait {
                    // This is a trait implementation
                    let trait_def_id = trait_ref.path.res.def_id();
                    let trait_name = tcx.def_path_str(trait_def_id);
                    let trait_module = tcx
                        .crate_name(trait_def_id.krate)
                        .to_ident_string();
                    let full_trait_name = format!("{}::{}", trait_module, trait_name);
                    
                    // Get the implementing type
                    let self_ty = tcx.type_of(item.owner_id).skip_binder();
                    let impl_type = format!("{:?}", self_ty);
                    
                    // Add implementor to trait
                    if let Some(implementors) = trait_map.get_mut(&full_trait_name) {
                        implementors.push(impl_type);
                    }
                }
            }
        }
        
        // Convert hashmap to vector of TraitInfo
        let traits: Vec<TraitInfo> = trait_map
            .into_iter()
            .map(|(name, implementors)| TraitInfo { name, implementors })
            .collect();
            
        // Create the context
        let context = GenerationContext { 
            modules, 
            module_root,
            traits 
        };
        
        // Generate config file
        match LintConfigurationFactory::generate_yaml(&context) {
            Ok(yaml) => {
                // Print the generated YAML to the result
                self.result_text = yaml;
                
                // Also write to file
                match LintConfigurationFactory::generate_config_file(&context, &config_filename) {
                    Ok(_) => self.result_text.push_str(&format!("\n\nConfig written to {}", config_filename)),
                    Err(e) => self.result_text.push_str(&format!("\n\nError writing file: {}", e)),
                }
            },
            Err(e) => {
                self.result_text = format!("Error generating configuration: {}", e);
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
        let cargo_args = self.cargo_args.clone();

        let lint_collection = Arc::clone(&self.lint_collection);
        config.register_lints = Some(Box::new(move |_sess, lint_store| {
            // If we're actually linting, recreate the lints and add them all
            if let Mode::Check = mode {
                //let lints = setup_lints_yaml().expect("can load lints");
                for lint in lint_collection.lints() {
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

            // Track cargo args
            if !cargo_args.is_empty() {
                let cargo_args_str = cargo_args.join(" ");
                psess.env_depinfo.get_mut().insert((
                    Symbol::intern("cargo_args"),
                    Some(Symbol::intern(&cargo_args_str)),
                ));
            }

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

// Fetch all the modules from a top-level module down
fn collect_modules(
    tcx: TyCtxt<'_>,
    mod_id: LocalModDefId,
    namespace_set: &mut BTreeSet<(String, String)>,
) {
    let (module, _, _) = tcx.hir_get_module(mod_id);

    for id in module.item_ids {
        let item = tcx.hir_item(*id);
        if let ItemKind::Mod(..) = item.kind {
            let namespace = tcx.def_path_str(item.owner_id.to_def_id());
            let module = tcx
                .crate_name(item.owner_id.to_def_id().krate)
                .to_ident_string();
            namespace_set.insert((module, namespace.clone()));
            let child_mod_id = LocalModDefId::new_unchecked(item.owner_id.def_id);
            collect_modules(tcx, child_mod_id, namespace_set);
        }
    }
}
