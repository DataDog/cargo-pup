use cargo_pup_common::project_context::ProjectContext;
use crate::{ConfiguredLint, GenerateFromContext, LintBuilder, ModuleMatch, ModuleRule, Severity};
use crate::module_lint::ModuleLint;

impl GenerateFromContext for ModuleLint {
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder) {
        // Rule 1: Global rule - mod.rs files must be empty
        let empty_mod_lint = ModuleLint {
            name: "empty_mod_rule".to_string(),
            matches: ModuleMatch::Module(".*".to_string()),
            rules: vec![
                ModuleRule::MustHaveEmptyModFile(Severity::Error)
            ],
        };
        builder.push(ConfiguredLint::Module(empty_mod_lint));
    }
}