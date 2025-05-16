use cargo_pup_common::project_context::ProjectContext;
use crate::function_lint::FunctionLint;
use crate::{ConfiguredLint, FunctionMatch, FunctionRule, GenerateFromContext, LintBuilder, ReturnTypePattern, Severity};

impl GenerateFromContext for FunctionLint {
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder) {
        // Rule 1: Global function length rule
        let function_length_lint = FunctionLint {
            name: "function_length_limit".to_string(),
            matches: FunctionMatch::NameRegex(".*".to_string()),
            rules: vec![
                FunctionRule::MaxLength(50, Severity::Error),
            ],
        };
        builder.push(ConfiguredLint::Function(function_length_lint));
        
        // Rule 2: Global result error rule
        let result_error_lint = FunctionLint {
            name: "result_error_must_implement_error".to_string(),
            matches: FunctionMatch::ReturnsType(ReturnTypePattern::Result),
            rules: vec![
                FunctionRule::ResultErrorMustImplementError(Severity::Error),
            ],
        };
        builder.push(ConfiguredLint::Function(result_error_lint));
        
        // Add context-specific rules if contexts are provided
        if !contexts.is_empty() {
            for context in contexts {
                if !context.module_root.is_empty() {
                    // For each project, add a function rule that has a stricter length limit
                    // for functions within that module
                    let module_functions_rule = FunctionLint {
                        name: format!("func_rules_for_{}", context.module_root),
                        matches: FunctionMatch::InModule(format!("{}::*", context.module_root)),
                        rules: vec![
                            FunctionRule::MaxLength(30, Severity::Warn),
                        ],
                    };
                    builder.push(ConfiguredLint::Function(module_functions_rule));
                }
            }
        }
    }
}