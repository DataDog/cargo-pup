use ansi_term::Color;
use rustc_driver::Callbacks;
use rustc_hir::ItemKind;
use rustc_hir::def_id::LocalModDefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;
use std::sync::Arc;
use std::{collections::BTreeSet, path::Path};

use crate::lints::{ArchitectureLintCollection, ArchitectureLintRule};
use crate::utils::project_context::{ProjectContext, TraitInfo};

///
/// The mode our lint runner should operate in
///
#[derive(Clone, PartialEq, Debug)]
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

    // Store the ProjectContext for serialization
    project_context: Option<ProjectContext>,
}

impl ArchitectureLintRunner {
    pub fn new(mode: Mode, cli_args: String, lint_collection: ArchitectureLintCollection) -> Self {
        ArchitectureLintRunner {
            mode,
            lint_collection: Arc::new(lint_collection),
            result_text: String::new(),
            cli_args,
            cargo_args: Vec::new(),
            project_context: None,
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
    /// Get the built ProjectContext if available
    ///
    pub fn get_project_context(&self) -> Option<&ProjectContext> {
        self.project_context.as_ref()
    }

    ///
    /// Serialize the ProjectContext to JSON format
    ///
    pub fn context_as_json(&self) -> Option<String> {
        self.project_context
            .as_ref()
            .map(|ctx| serde_json::to_string(ctx).unwrap_or_else(|_| "{}".to_string()))
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

    // Handle each mode and return Result
    fn handle_mode(&mut self, tcx: TyCtxt<'_>) -> anyhow::Result<()> {
        use anyhow::Context;
        match self.mode {
            Mode::Check => {
                // Do nothing. Checking happens as part of the lints.
                let _ = self.lint_collection.lints();
                Ok(())
            }
            Mode::PrintModules | Mode::PrintTraits => {
                // For these modes, we build the project context, then serialize it
                // out to .pup. The outer call - e.g. cargo-pup - then grabs it all
                // and uses it to produce a complete view of all the nested projects.
                let context = self
                    .build_project_context(tcx)
                    .context("Failed to build project context for print-modules mode")?;

                // Serialize the context to a file
                if let Err(e) = context.serialize_to_file() {
                    eprintln!("Warning: Failed to serialize project context: {}", e);
                }

                Ok(())
            }
            Mode::GenerateConfig => {
                // For config generation, get the result and update result_text
                let result = self.generate_config(tcx)?;
                self.result_text = result;
                Ok(())
            }
        }
    }

    /// Build ProjectContext with modules and traits info common to multiple modes
    fn build_project_context(&self, tcx: TyCtxt<'_>) -> anyhow::Result<ProjectContext> {
        use std::collections::HashMap;

        // Create a namespace set with modules
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
                trait_map.entry(full_trait_name).or_default();
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
                    let trait_module = tcx.crate_name(trait_def_id.krate).to_ident_string();
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

        // Return the context
        Ok(ProjectContext {
            modules,
            module_root,
            traits,
        })
    }

    // Implementation function that returns Result
    fn generate_config(&mut self, tcx: TyCtxt<'_>) -> anyhow::Result<String> {
        use crate::utils::configuration_factory::LintConfigurationFactory;
        use anyhow::Context;

        // Build the project context
        let context = self
            .build_project_context(tcx)
            .context("Failed to build project context")?;

        // Create filename with module root that we'll use later
        let config_filename = format!("pup.generated.{}.yaml", context.module_root);

        // Generate config file
        let yaml = LintConfigurationFactory::generate_yaml(&context)
            .context("Failed to generate YAML configuration")?;

        // Write to file
        LintConfigurationFactory::generate_config_file(&context, &config_filename).context(
            format!("Failed to write configuration to {}", config_filename),
        )?;

        // Store the context for later use
        self.project_context = Some(context);

        // Return success message
        Ok(format!("{}\n\nConfig written to {}", yaml, config_filename))
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
        if let Err(e) = self.handle_mode(tcx) {
            // For fatal errors, print the error and exit
            eprintln!("Error: {:#}", e);
            std::process::exit(1);
        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::project_context::TraitInfo;

    // Test the Mode enum
    #[test]
    fn test_mode_debug_and_eq() {
        // Test Mode Debug and PartialEq implementations
        let check_mode = Mode::Check;
        let print_modules_mode = Mode::PrintModules;
        let print_traits_mode = Mode::PrintTraits;
        let generate_config_mode = Mode::GenerateConfig;

        // Verify Debug formatting
        assert_eq!(format!("{:?}", check_mode), "Check");
        assert_eq!(format!("{:?}", print_modules_mode), "PrintModules");
        assert_eq!(format!("{:?}", print_traits_mode), "PrintTraits");
        assert_eq!(format!("{:?}", generate_config_mode), "GenerateConfig");

        // Verify PartialEq
        assert_eq!(check_mode, Mode::Check);
        assert_eq!(print_modules_mode, Mode::PrintModules);
        assert_eq!(print_traits_mode, Mode::PrintTraits);
        assert_eq!(generate_config_mode, Mode::GenerateConfig);

        assert_ne!(check_mode, print_modules_mode);
        assert_ne!(check_mode, print_traits_mode);
        assert_ne!(check_mode, generate_config_mode);
    }

    // Test the Clone trait for Mode
    #[test]
    fn test_mode_clone() {
        let modes = vec![
            Mode::Check,
            Mode::PrintModules,
            Mode::PrintTraits,
            Mode::GenerateConfig,
        ];

        for mode in &modes {
            let cloned_mode = mode.clone();
            assert_eq!(*mode, cloned_mode);
        }
    }

    // Test creating a ProjectContext
    #[test]
    fn test_create_project_context() {
        // Create a project context
        let context = ProjectContext {
            modules: vec!["test::module1".to_string(), "test::module2".to_string()],
            module_root: "test".to_string(),
            traits: vec![TraitInfo {
                name: "test::Trait1".to_string(),
                implementors: vec!["Type1".to_string(), "Type2".to_string()],
            }],
        };

        // Verify the context properties
        assert_eq!(context.module_root, "test");
        assert_eq!(context.modules.len(), 2);
        assert_eq!(context.modules[0], "test::module1");
        assert_eq!(context.modules[1], "test::module2");
        assert_eq!(context.traits.len(), 1);
        assert_eq!(context.traits[0].name, "test::Trait1");
        assert_eq!(context.traits[0].implementors.len(), 2);
    }

    // Test serializing ProjectContext to JSON
    #[test]
    fn test_project_context_json_serialization() {
        // Create a project context
        let context = ProjectContext {
            modules: vec!["test::module".to_string()],
            module_root: "test".to_string(),
            traits: vec![TraitInfo {
                name: "test::Trait1".to_string(),
                implementors: vec!["Type1".to_string()],
            }],
        };

        // Serialize to JSON
        let json = serde_json::to_string(&context).expect("Failed to serialize context");

        // Verify JSON contains expected data
        assert!(json.contains("\"modules\":[\"test::module\"]"));
        assert!(json.contains("\"module_root\":\"test\""));
        assert!(json.contains("\"traits\":[{"));
        assert!(json.contains("\"name\":\"test::Trait1\""));
        assert!(json.contains("\"implementors\":[\"Type1\"]"));
    }

    // Test deserializing ProjectContext from JSON
    #[test]
    fn test_project_context_json_deserialization() {
        // Create a JSON string
        let json = r#"
        {
            "modules": ["test::module"],
            "module_root": "test",
            "traits": [
                {
                    "name": "test::Trait1",
                    "implementors": ["Type1"]
                }
            ]
        }
        "#;

        // Deserialize from JSON
        let context: ProjectContext =
            serde_json::from_str(json).expect("Failed to deserialize context");

        // Verify context properties
        assert_eq!(context.module_root, "test");
        assert_eq!(context.modules.len(), 1);
        assert_eq!(context.modules[0], "test::module");
        assert_eq!(context.traits.len(), 1);
        assert_eq!(context.traits[0].name, "test::Trait1");
        assert_eq!(context.traits[0].implementors.len(), 1);
        assert_eq!(context.traits[0].implementors[0], "Type1");
    }
}
