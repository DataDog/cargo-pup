use rustc_hir::OwnerId;
use rustc_hir::def_id::DefId;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::ty::{self, ParamEnv, Ty, TyCtxt, TypingMode};
use rustc_span::symbol::sym;
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt;
use rustc_trait_selection::traits::{Obligation, ObligationCause};

///
/// Returns the name for a module_lint. If the module_lint is the root module_lint, returns just the crate name.
/// For submodules, includes the crate name with module_lint path.
///
pub fn get_full_module_name(tcx: &TyCtxt<'_>, module_def_id: &OwnerId) -> String {
    let krate_name = tcx
        .crate_name(module_def_id.to_def_id().krate)
        .to_ident_string();
    let module_name = tcx.def_path_str(module_def_id.to_def_id());
    
    // If the module_name is empty, this is the root module_lint
    // In that case, return just the crate name without "::"
    if module_name.is_empty() {
        krate_name
    } else {
        format!("{}::{}", krate_name, module_name)
    }
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

/// Creates a canonical trait name from a potentially generic trait name.
/// This removes any generic parameters (including lifetimes) from the trait name.
/// 
/// Example: "Iterator<'a, T>" becomes "Iterator"
///
/// Used to standardize trait names for display and for lint rule matching.
pub fn get_canonical_trait_name(trait_name: &str) -> String {
    if let Some(pos) = trait_name.find('<') {
        trait_name[0..pos].to_string()
    } else {
        trait_name.to_string()
    }
}

/// Creates a fully qualified canonical trait name with crate prefix.
/// 
/// Example: if crate_name = "std" and trait_name = "Iterator<'a, T>",
/// returns "std::Iterator"
pub fn get_full_canonical_trait_name(crate_name: &str, trait_name: &str) -> String {
    let canonical_name = get_canonical_trait_name(trait_name);
    format!("{}::{}", crate_name, canonical_name)
}

/// Gets the canonical trait name from TyCtxt and DefId.
/// Combines getting the trait name from compiler's DefId and canonicalizing it.
pub fn get_canonical_trait_name_from_def_id(tcx: &TyCtxt<'_>, def_id: DefId) -> String {
    let raw_trait_name = tcx.def_path_str(def_id);
    get_canonical_trait_name(&raw_trait_name)
}

/// Gets the fully qualified canonical trait name with crate from TyCtxt and DefId.
pub fn get_full_canonical_trait_name_from_def_id(tcx: &TyCtxt<'_>, def_id: DefId) -> String {
    let crate_name = tcx.crate_name(def_id.krate).to_string();
    let raw_trait_name = tcx.def_path_str(def_id);
    get_full_canonical_trait_name(&crate_name, &raw_trait_name)
}

/// Creates a canonical type representation by removing generic parameters.
/// 
/// Example: "Vec<String>" becomes "Vec"
pub fn get_canonical_type_name(type_name: &str) -> String {
    if let Some(pos) = type_name.find('<') {
        type_name[0..pos].to_string()
    } else {
        type_name.to_string()
    }
}
