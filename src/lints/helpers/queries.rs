use rustc_hir::OwnerId;
use rustc_hir::def_id::DefId;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::ty::{self, ParamEnv, Ty, TyCtxt, TypingMode};
use rustc_span::symbol::sym;
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt;
use rustc_trait_selection::traits::{Obligation, ObligationCause};

///
/// Returns the complete name for a module, including the crate name.
///
pub fn get_full_module_name(tcx: &TyCtxt<'_>, module_def_id: &OwnerId) -> String {
    let krate_name = tcx
        .crate_name(module_def_id.to_def_id().krate)
        .to_ident_string();
    let module_name = tcx.def_path_str(module_def_id.to_def_id());
    format!("{}::{}", krate_name, module_name)
}

pub fn implements_trait<'tcx>(
    tcx: TyCtxt<'tcx>,
    param_env: ParamEnv<'tcx>,
    ty: Ty<'tcx>,
    trait_def_id: DefId,
) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::Coherence);

    let cause = ObligationCause::dummy();
    let trait_ref = ty::TraitRef::new(tcx, trait_def_id, [ty]);

    let obligation = Obligation::new(tcx, cause, param_env, trait_ref);

    infcx.predicate_may_hold(&obligation)
}

/// Checks if a type implements the standard `std::error::Error` trait.
pub fn implements_error_trait<'tcx>(
    tcx: TyCtxt<'tcx>,
    param_env: ParamEnv<'tcx>,
    ty: Ty<'tcx>,
) -> bool {
    // Check for primitive types that definitely don't implement Error
    match ty.kind() {
        ty::TyKind::Int(_)
        | ty::TyKind::Uint(_)
        | ty::TyKind::Float(_)
        | ty::TyKind::Bool
        | ty::TyKind::Char => return false,
        _ => {}
    }

    // Try the standard approach
    if let Some(error_trait_def_id) = tcx.get_diagnostic_item(sym::Error) {
        implements_trait(tcx, param_env, ty, error_trait_def_id)
    } else {
        // If we can't find the Error trait, be conservative and consider it might implement Error
        // unless it's a primitive type (which we checked above)
        !ty.is_primitive()
    }
}
