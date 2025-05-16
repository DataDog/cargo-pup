use cargo_pup_common::project_context::ProjectContext;
use crate::{ConfiguredLint, GenerateFromContext, LintBuilder, Severity, StructMatch, StructRule};
use crate::struct_lint::StructLint;
use std::collections::HashSet;

impl GenerateFromContext for StructLint {
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder) {
        // Skip if no contexts provided
        if contexts.is_empty() {
            return;
        }
        
        // Add default struct naming convention lint
        let default_struct_lint = StructLint {
            name: "struct_naming_convention".to_string(),
            matches: StructMatch::Name(".*".to_string()),
            rules: vec![
                StructRule::MustBeNamed(".*".to_string(), Severity::Warn),
            ],
        };
        builder.push(ConfiguredLint::Struct(default_struct_lint));
        
        // Keep track of traits we've processed to avoid duplicates
        let mut processed_traits = HashSet::new();
        
        // Process all traits from all contexts
        for context in contexts {
            // Generate struct lints based on traits in the context
            if !context.traits.is_empty() {
                for trait_info in &context.traits {
                    // Only create lints for traits with implementors
                    if !trait_info.implementors.is_empty() {
                        // Skip if we've already processed this trait
                        if processed_traits.contains(&trait_info.name) {
                            continue;
                        }
                        
                        // Mark this trait as processed
                        processed_traits.insert(trait_info.name.clone());
                        
                        // Create a struct lint that enforces trait implementation
                        let trait_lint = StructLint {
                            name: format!("implements_{}", trait_info.name),
                            matches: StructMatch::Name(".*".to_string()),
                            rules: vec![
                                StructRule::ImplementsTrait(trait_info.name.clone(), Severity::Warn),
                            ],
                        };
                        builder.push(ConfiguredLint::Struct(trait_lint));
                    }
                }
            }
        }
    }
}