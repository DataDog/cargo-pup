use cargo_pup_common::project_context::ProjectContext;
use crate::{ConfiguredLint, GenerateFromContext, LintBuilder, ModuleMatch, ModuleRule, Severity};
use crate::module_lint::ModuleLint;

impl GenerateFromContext for ModuleLint {
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder) {
        // Rule 1: Global rule - mod.rs files must be empty
        let empty_mod_lint = ModuleLint {
            name: "empty_mod_rule".to_string(),
            matches: ModuleMatch::Module(".*mod\\.rs$".to_string()),
            rules: vec![
                ModuleRule::MustHaveEmptyModFile(Severity::Error)
            ],
        };
        builder.push(ConfiguredLint::Module(empty_mod_lint));
        
        // Rule 2: Global rule - errors must be error traits
        let error_trait_lint = ModuleLint {
            name: "result_error_trait".to_string(),
            matches: ModuleMatch::Module(".*".to_string()), 
            rules: vec![
                ModuleRule::DeniedItems {
                    items: vec![
                        "Result<*, i32>".to_string(),
                        "Result<*, String>".to_string(),
                        "Result<*, &str>".to_string(),
                        "Result<*, bool>".to_string(),
                        "Result<*, ()>".to_string(),
                    ],
                    severity: Severity::Error,
                },
            ],
        };
        builder.push(ConfiguredLint::Module(error_trait_lint));
        
        // Add context-specific rules if contexts are provided
        if !contexts.is_empty() {
            for context in contexts {
                if !context.module_root.is_empty() {
                    // For each project, add a no wildcard imports rule
                    let no_wildcard_imports = ModuleLint {
                        name: format!("no_wildcard_imports_{}", context.module_root),
                        matches: ModuleMatch::Module(format!("{}::*", context.module_root)),
                        rules: vec![
                            ModuleRule::NoWildcardImports(Severity::Warn)
                        ],
                    };
                    builder.push(ConfiguredLint::Module(no_wildcard_imports));
                }
            }
        }
    }
}