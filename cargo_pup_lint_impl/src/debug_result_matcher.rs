// This is a temporary debugger to test the Result matcher
use rustc_lint::{LateContext, LateLintPass, Lint, LintStore};
use rustc_hir::{Item, ItemKind, ImplItem, ImplItemKind, def_id::LOCAL_CRATE};
use rustc_middle::ty::TyKind;
use rustc_session::impl_lint_pass;
use rustc_span::BytePos;
use regex::Regex;

pub struct DebugResultMatcher;

#[derive(Copy, Clone, Debug)]
pub struct DebugResult {
    pub name: &'static str,
}

impl DebugResult {
    const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

declare_lint! {
    pub DEBUG_RESULT,
    Warn,
    "Matches functions that return Result<T, E>"
}

impl_lint_pass!(DebugResultMatcher => [DEBUG_RESULT]);

impl<'tcx> LateLintPass<'tcx> for DebugResultMatcher {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // Only care about functions
        if let ItemKind::Fn { .. } = item.kind {
            let item_name = ctx.tcx.item_name(item.owner_id.def_id.to_def_id()).to_string();
            let _crate_name = ctx.tcx.crate_name(LOCAL_CRATE).to_string();
            let fn_def_id = item.owner_id.to_def_id();
            
            eprintln!("DEBUG: Checking function: {}", item_name);
            
            // Get the function signature
            let fn_sig = ctx.tcx.fn_sig(fn_def_id).skip_binder();
            let return_ty = fn_sig.output().skip_binder();
            
            eprintln!("DEBUG: Return type: {:?}", return_ty);
            
            // Check if return type is Result
            let is_result = match return_ty.kind() {
                TyKind::Adt(adt_def, _) => {
                    let path = ctx.tcx.def_path_str(adt_def.did());
                    eprintln!("DEBUG: Type path: {}", path);
                    path.contains("result::Result")
                },
                _ => {
                    let type_string = return_ty.to_string();
                    eprintln!("DEBUG: Type string fallback: {}", type_string);
                    type_string.contains("Result<")
                }
            };
            
            eprintln!("DEBUG: Is Result type: {}", is_result);
            
            if is_result {
                // Report a lint for debugging
                ctx.struct_span_lint(
                    DEBUG_RESULT, 
                    item.span, 
                    |diag| {
                        diag.build(&format!("Function '{}' returns a Result type", item_name))
                            .emit();
                    }
                );
            }
        }
    }
} 