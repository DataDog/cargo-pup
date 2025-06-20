// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use crate::ArchitectureLintCollection;
use cargo_pup_common::project_context::{ModuleInfo, ProjectContext, TraitInfo};
use rustc_driver::Callbacks;
use rustc_hir::ItemKind;
use rustc_hir::def_id::LocalModDefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;
use std::sync::Arc;
use std::{collections::BTreeSet, path::Path};

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

    // Handles the different execution modes we have, potentially returning a failure
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
                //
                // We don't print any of our own output! We're just discovering project
                // structure info for cargo-pup.
                let context = self
                    .build_project_context(tcx)
                    .context("Failed to build project context for print-modules mode")?;

                // Serialize the context to a file
                if let Err(e) = context.serialize_to_file() {
                    eprintln!("Warning: Failed to serialize project context: {e}");
                }

                Ok(())
            }
            Mode::GenerateConfig => {
                // For config generation, just build the project context and serialize it
                // like we do for PrintModules and PrintTraits. The cargo-pup tool will handle
                // generating the config from the serialized context.
                let context = self
                    .build_project_context(tcx)
                    .context("Failed to build project context for generate-config mode")?;

                // Serialize the context to a file
                if let Err(e) = context.serialize_to_file() {
                    eprintln!("Warning: Failed to serialize project context: {e}");
                }

                // Set a simple success message
                self.result_text = format!(
                    "Project context successfully generated for crate {}",
                    context.module_root
                );
                Ok(())
            }
        }
    }

    /// Build ProjectContext. This includes module and trait information - and is typically
    /// used by cargo-pup - on the outside of the pup-driver execution - to display project
    /// info to the user.
    fn build_project_context(&self, tcx: TyCtxt<'_>) -> anyhow::Result<ProjectContext> {
        use std::collections::HashMap;

        // Create a namespace set with modules
        let mut namespace_set: BTreeSet<(String, String)> = BTreeSet::new();
        collect_modules(tcx, LocalModDefId::CRATE_DEF_ID, &mut namespace_set);

        // Get the current crate name (module root)
        // Just take the first entry's module name, which is the current crate
        let module_root = if let Some((crate_name, _)) = namespace_set.iter().next() {
            crate_name.clone()
        } else {
            "unknown_crate".to_string()
        };

        // Create ModuleInfo objects with applicable lints
        let mut module_infos = Vec::new();
        let lints = self.lint_collection.lints();

        for (module, path) in &namespace_set {
            let full_module_path = format!("{module}::{path}");

            // Find lints that apply to this module
            let applicable_lints: Vec<String> = lints
                .iter()
                .filter(|lint| lint.applies_to_module(&full_module_path))
                .map(|lint| lint.name())
                .collect();

            // Create ModuleInfo with applicable lints
            module_infos.push(ModuleInfo {
                name: full_module_path,
                applicable_lints,
            });
        }

        // Collect all traits and their implementors
        let mut trait_map: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();

        // Find all traits in the crate
        let module_items = tcx.hir_crate_items(());
        for item_id in module_items.free_items() {
            let item = tcx.hir_item(item_id);
            if let ItemKind::Trait(..) = item.kind {
                // Get the canonical trait name using the centralized helper
                let def_id = item.owner_id.to_def_id();
                let canonical_full_name =
                    crate::helpers::queries::get_full_canonical_trait_name_from_def_id(
                        &tcx, def_id,
                    );

                // Use the canonical name as the map key
                trait_map
                    .entry(canonical_full_name)
                    .or_insert_with(|| (Vec::new(), Vec::new()));
            }
        }

        // Find implementations
        for item_id in module_items.free_items() {
            let item = tcx.hir_item(item_id);
            if let ItemKind::Impl(impl_data) = &item.kind
                && let Some(trait_ref) = impl_data.of_trait {
                    // This is a trait implementation
                    // Get the canonical trait name using the centralized helper
                    let trait_def_id = trait_ref.path.res.def_id();
                    let canonical_full_name =
                        crate::helpers::queries::get_full_canonical_trait_name_from_def_id(
                            &tcx,
                            trait_def_id,
                        );

                    // Get the implementing type and clean up the display
                    let self_ty = tcx.type_of(item.owner_id).skip_binder();
                    let impl_type_raw = format!("{self_ty}");

                    // Clean up implementation type by removing generic parameters using the centralized helper
                    let impl_type =
                        crate::helpers::queries::get_canonical_type_name(&impl_type_raw);

                    // Add implementor to trait if it's not already in the list
                    if let Some((implementors, _)) = trait_map.get_mut(&canonical_full_name)
                        && !implementors.contains(&impl_type) {
                            implementors.push(impl_type);
                        }
                }
        }

        // Find lints that apply to each trait
        for (canonical_name, (_, applicable_lints)) in &mut trait_map {
            // Add lints that apply to this trait
            for lint in lints.iter() {
                // Use the canonical name (without generics or lifetimes) for matching
                // This ensures consistent behavior across all lint rules
                if lint.applies_to_trait(canonical_name) {
                    applicable_lints.push(lint.name());
                }
            }
        }

        // Convert hashmap to vector of TraitInfo
        let traits: Vec<TraitInfo> = trait_map
            .into_iter()
            .map(|(name, (implementors, applicable_lints))| TraitInfo {
                name,
                implementors,
                applicable_lints,
            })
            .collect();

        // Build and return the context
        let mut context = ProjectContext::new();
        context.module_root = module_root;
        context.modules = module_infos;
        context.traits = traits;

        Ok(context)
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
            if Path::new("../../../pup.ron").exists() {
                psess
                    .file_depinfo
                    .get_mut()
                    .insert(Symbol::intern("pup.ron"));
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
            eprintln!("Error: {e:#}");
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
        let context = ProjectContext::with_data(
            vec!["test::module1".to_string(), "test::module2".to_string()],
            "test".to_string(),
            vec![TraitInfo {
                name: "test::Trait1".to_string(),
                implementors: vec!["Type1".to_string(), "Type2".to_string()],
                applicable_lints: vec![],
            }],
        );

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
        let context = ProjectContext::with_data(
            vec!["test::module".to_string()],
            "test".to_string(),
            vec![TraitInfo {
                name: "test::Trait1".to_string(),
                implementors: vec!["Type1".to_string()],
                applicable_lints: vec![],
            }],
        );

        // Serialize to JSON
        let json = serde_json::to_string(&context).expect("Failed to serialize context");

        // Verify JSON contains expected data
        assert!(json.contains("\"name\":\"test::module\""));
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
            "modules": [
                {
                    "name": "test::module",
                    "applicable_lints": []
                }
            ],
            "module_root": "test",
            "traits": [
                {
                    "name": "test::Trait1",
                    "implementors": ["Type1"],
                    "applicable_lints": []
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
