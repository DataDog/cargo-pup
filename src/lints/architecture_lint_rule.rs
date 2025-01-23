use rustc_middle::ty::TyCtxt;

use super::LintResult;

/// Trait for defining architecture-specific lint rules
pub trait ArchitectureLintRule {
    fn lint(&self, ctx: TyCtxt<'_>) -> Vec<LintResult>;
}
