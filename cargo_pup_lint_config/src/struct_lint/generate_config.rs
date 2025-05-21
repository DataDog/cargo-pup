use crate::struct_lint::StructLint;
use crate::{GenerateFromContext, LintBuilder};
use cargo_pup_common::project_context::ProjectContext;

impl GenerateFromContext for StructLint {
    fn generate_from_contexts(_contexts: &[ProjectContext], _builder: &mut LintBuilder) {
        // Not much to do here!
    }
}
