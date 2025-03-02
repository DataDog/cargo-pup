use rustc_hir::OwnerId;
use rustc_middle::ty::TyCtxt;


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