use cargo_pup_common::project_context::ProjectContext;
use crate::{ConfiguredLint, GenerateFromContext, LintBuilder, ModuleMatch, ModuleRule, Severity};
use crate::module_lint::ModuleLint;

impl GenerateFromContext for ModuleLint {
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder) {
        for context in contexts {
            // For each project, add an empty
            let empty_mod_lint = ModuleLint {
                name: "empty_mod_rule".to_string(),
                matches: ModuleMatch::Module(format!("{0}::*", context.module_root)),
                rules: vec![
                    ModuleRule::MustHaveEmptyModFile(Severity::Warn)
                ],
            };
            builder.push(ConfiguredLint::Module(empty_mod_lint));

        }
    }
}