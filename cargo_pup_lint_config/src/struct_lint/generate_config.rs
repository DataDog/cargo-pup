use cargo_pup_common::project_context::ProjectContext;
use crate::{ConfiguredLint, GenerateFromContext, LintBuilder, Severity, StructMatch, StructRule};
use crate::struct_lint::StructLint;

impl GenerateFromContext for StructLint {
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder) {
        // Nothing to do so far!
    }
}