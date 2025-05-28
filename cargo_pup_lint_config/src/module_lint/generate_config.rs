// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use crate::module_lint::ModuleLint;
use crate::{ConfiguredLint, GenerateFromContext, LintBuilder, ModuleMatch, ModuleRule, Severity};
use cargo_pup_common::project_context::ProjectContext;

impl GenerateFromContext for ModuleLint {
    fn generate_from_contexts(_contexts: &[ProjectContext], builder: &mut LintBuilder) {
        // Rule 1: Global rule - mod.rs files must be empty
        let empty_mod_lint = ModuleLint {
            name: "empty_mod_rule".to_string(),
            matches: ModuleMatch::Module(".*".to_string()),
            rules: vec![ModuleRule::MustHaveEmptyModFile(Severity::Error)],
        };
        builder.push(ConfiguredLint::Module(empty_mod_lint));
    }
}
