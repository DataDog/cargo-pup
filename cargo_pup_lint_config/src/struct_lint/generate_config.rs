use cargo_pup_common::project_context::ProjectContext;
use crate::{ConfiguredLint, GenerateFromContext, LintBuilder, Severity, StructMatch, StructRule};
use crate::struct_lint::StructLint;
use std::collections::HashSet;

impl GenerateFromContext for StructLint {
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder) {
        // Not much to do here!
    }
}