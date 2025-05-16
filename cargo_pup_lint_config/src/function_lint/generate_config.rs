use cargo_pup_common::project_context::ProjectContext;
use crate::function_lint::FunctionLint;
use crate::{ConfiguredLint, FunctionMatch, FunctionRule, GenerateFromContext, LintBuilder, ModuleMatch, ModuleRule, Severity};
use crate::module_lint::ModuleLint;

impl GenerateFromContext for FunctionLint {
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder) {
        for context in contexts {

            // For each project, add a function rule that limits length and
            // requires that functions that return a result must implement error.
            let short_functions_rule = FunctionLint {
                name: format!("func_rules_for_{}", context.module_root),
                matches: FunctionMatch::InModule(format!("{0}::*", context.module_root)),
                rules: vec![
                    FunctionRule::MaxLength(50, Severity::Warn),
                    FunctionRule::ResultErrorMustImplementError(Severity::Warn)
                ],
            };
            builder.push(ConfiguredLint::Function(short_functions_rule));

        }
    }
}