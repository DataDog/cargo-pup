// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use crate::ArchitectureLintRule;
use crate::declare_variable_severity_lint;
use crate::helpers::lint_helpers::span_lint_and_help;
use crate::helpers::queries::{get_full_module_name, implements_error_trait};
use cargo_pup_lint_config::{
    ConfiguredLint, FunctionMatch, FunctionRule, ReturnTypePattern};
use regex::Regex;
use rustc_hir::{ImplItem, ImplItemKind, Item, ItemKind, def_id::LOCAL_CRATE};
use rustc_lint::{LateContext, LateLintPass, LintStore};
use rustc_middle::ty::TyKind;
use rustc_session::impl_lint_pass;
use rustc_span::BytePos;

pub struct FunctionLint {
    name: String,
    matches: FunctionMatch,
    function_rules: Vec<FunctionRule>,
}

impl FunctionLint {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: &ConfiguredLint) -> Box<dyn ArchitectureLintRule + Send> {
        if let ConfiguredLint::Function(f) = config {
            Box::new(Self {
                name: f.name.clone(),
                matches: f.matches.clone(),
                function_rules: f.rules.clone(),
            })
        } else {
            panic!("Expected a Function lint configuration")
        }
    }

    // Helper method to check if a function in a given module with a given name should be linted
    fn matches_function(
        &self,
        ctx: &LateContext<'_>,
        module_path: &str,
        function_name: &str,
        fn_def_id: rustc_hir::def_id::DefId,
    ) -> bool {
        evaluate_function_match(&self.matches, ctx, module_path, function_name, fn_def_id)
    }

    // Evaluates the complex matcher structure to determine if a function matches
}

fn evaluate_function_match(
    matcher: &FunctionMatch,
    ctx: &LateContext<'_>,
    module_path: &str,
    function_name: &str,
    fn_def_id: rustc_hir::def_id::DefId,
) -> bool {
    match matcher {
        FunctionMatch::NameEquals(name) => function_name == name,
        FunctionMatch::NameRegex(pattern) => match Regex::new(pattern) {
            Ok(regex) => regex.is_match(function_name),
            Err(_) => false,
        },
        FunctionMatch::InModule(pattern) => match Regex::new(pattern) {
            Ok(regex) => regex.is_match(module_path),
            Err(_) => module_path == pattern,
        },
        FunctionMatch::ReturnsType(pattern) => {
            // Get the correct return type from the function signature
            let fn_sig = ctx.tcx.fn_sig(fn_def_id).skip_binder();
            let return_ty = fn_sig.output().skip_binder();

            match pattern {
                ReturnTypePattern::Result => {
                    // Check for Adt with Result path
                    if let TyKind::Adt(adt_def, _) = return_ty.kind() {
                        let path = ctx.tcx.def_path_str(adt_def.did());
                        return path.contains("result::Result");
                    }

                    // Fallback: use the string representation
                    let type_string = return_ty.to_string();
                    type_string.contains("Result<")
                }
                ReturnTypePattern::ResultWithErrorImpl => {
                    // First check if it's a Result type
                    if let TyKind::Adt(adt_def, substs) = return_ty.kind() {
                        let path = ctx.tcx.def_path_str(adt_def.did());

                        // If it's a Result type
                        if path.contains("result::Result") && substs.len() >= 2 {
                            // Get the error type (second type parameter)
                            let error_ty = substs[1].expect_ty();

                            // Check if the error type implements Error trait
                            let param_env = ctx.param_env;
                            return implements_error_trait(ctx.tcx, param_env, error_ty);
                        }
                    }

                    // Not a Result type or couldn't determine if error type implements Error
                    false
                }
                ReturnTypePattern::Option => {
                    // Check for Adt with Option path
                    if let TyKind::Adt(adt_def, _) = return_ty.kind() {
                        let path = ctx.tcx.def_path_str(adt_def.did());
                        return path.contains("option::Option");
                    }

                    // Fallback: use the string representation
                    let type_string = return_ty.to_string();
                    type_string.contains("Option<")
                }
                ReturnTypePattern::Named(name) => {
                    // Check for Adt with the exact name
                    if let TyKind::Adt(adt_def, _) = return_ty.kind() {
                        let path = ctx.tcx.def_path_str(adt_def.did());

                        // Try to match the simple name at the end of the path
                        if path.ends_with(&format!("::{}", name)) || path == *name {
                            return true;
                        }

                        // Extract the type name without module path
                        if let Some(last_segment) = path.split("::").last() {
                            if last_segment == *name {
                                return true;
                            }
                        }
                    }

                    // Fallback: use the string representation
                    let type_string = return_ty.to_string();
                    type_string == *name || type_string.ends_with(&format!("::{}", name))
                }
                ReturnTypePattern::Regex(regex_pattern) => {
                    // Try to compile and use the regex pattern
                    match Regex::new(regex_pattern) {
                        Ok(regex) => {
                            // Check the string representation of the type against the regex
                            let type_string = return_ty.to_string();
                            regex.is_match(&type_string)
                        }
                        Err(_) => false,
                    }
                }
            }
        }
        FunctionMatch::AndMatches(left, right) => {
            evaluate_function_match(left, ctx, module_path, function_name, fn_def_id)
                && evaluate_function_match(right, ctx, module_path, function_name, fn_def_id)
        }
        FunctionMatch::OrMatches(left, right) => {
            evaluate_function_match(left, ctx, module_path, function_name, fn_def_id)
                || evaluate_function_match(right, ctx, module_path, function_name, fn_def_id)
        }
        FunctionMatch::NotMatch(inner) => {
            !evaluate_function_match(inner, ctx, module_path, function_name, fn_def_id)
        }
    }
}

// Declare the function_lint lint with variable severity
declare_variable_severity_lint!(
    pub,
    FUNCTION_LINT,
    FUNCTION_LINT_DENY,
    FUNCTION_LINT_WARN,
    "Function properties and constraints"
);

impl_lint_pass!(FunctionLint => [FUNCTION_LINT_DENY, FUNCTION_LINT_WARN]);

impl ArchitectureLintRule for FunctionLint {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn applies_to_module(&self, _namespace: &str) -> bool {
        false
    }

    fn applies_to_trait(&self, _trait_path: &str) -> bool {
        false
    }

    fn register_late_pass(&self, lint_store: &mut LintStore) {
        let name = self.name.clone();
        let matches = self.matches.clone();
        let function_rules = self.function_rules.clone();

        lint_store.register_late_pass(move |_| {
            Box::new(FunctionLint {
                name: name.clone(),
                matches: matches.clone(),
                function_rules: function_rules.clone(),
            })
        });
    }
}

impl<'tcx> LateLintPass<'tcx> for FunctionLint {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // Only care about functions
        if let ItemKind::Fn { body, .. } = item.kind {
            let item_name = ctx
                .tcx
                .item_name(item.owner_id.def_id.to_def_id())
                .to_string();
            let _crate_name = ctx.tcx.crate_name(LOCAL_CRATE).to_string();
            let parent_item = ctx.tcx.hir_get_parent_item(item.hir_id());
            let module_path = get_full_module_name(&ctx.tcx, &parent_item);
            let fn_def_id = item.owner_id.to_def_id();

            // Check if this function matches our patterns
            if !self.matches_function(ctx, &module_path, &item_name, fn_def_id) {
                return;
            }

            // Apply rules
            for rule in &self.function_rules {
                match rule {
                    FunctionRule::MaxLength(max_lines, severity) => {
                        let body = ctx.tcx.hir_body(body);
                        let source_map = ctx.tcx.sess.source_map();

                        if let Ok(file_lines) = source_map.span_to_lines(body.value.span) {
                            if file_lines.lines.len() > *max_lines {
                                // Create a span that only covers the function signature
                                let sig_span = item.span.with_hi(
                                    item.span.lo() + BytePos((item_name.len() + 5) as u32), // "fn name"
                                );

                                span_lint_and_help(
                                    ctx,
                                    FUNCTION_LINT::get_by_severity(*severity),
                                    self.name().as_str(),
                                    sig_span,
                                    format!(
                                        "Function exceeds maximum length of {} lines with {} lines",
                                        max_lines,
                                        file_lines.lines.len()
                                    ),
                                    None,
                                    "Consider breaking this function into smaller parts",
                                );
                            }
                        }
                    }
                    FunctionRule::ResultErrorMustImplementError(severity) => {
                        // Get the return type
                        let fn_sig = ctx.tcx.fn_sig(fn_def_id).skip_binder();
                        let return_ty = fn_sig.output().skip_binder();

                        // Check if it's a Result type
                        if let TyKind::Adt(adt_def, substs) = return_ty.kind() {
                            let path = ctx.tcx.def_path_str(adt_def.did());

                            // If it's a Result type
                            if path.contains("result::Result") && substs.len() >= 2 {
                                let error_ty = substs[1].expect_ty();
                                let param_env = ctx.param_env;

                                // Check if error type does NOT implement Error trait
                                if !implements_error_trait(ctx.tcx, param_env, error_ty) {
                                    let error_type_name = error_ty.to_string();

                                    // Create a span that only covers the function signature
                                    let sig_span = item.span.with_hi(
                                        item.span.lo() + BytePos((item_name.len() + 5) as u32), // "fn name"
                                    );

                                    span_lint_and_help(
                                        ctx,
                                        FUNCTION_LINT::get_by_severity(*severity),
                                        self.name().as_str(),
                                        sig_span,
                                        format!(
                                            "Error type '{}' in Result does not implement Error trait",
                                            error_type_name
                                        ),
                                        None,
                                        "Consider implementing the Error trait for this type or using a type that already implements it",
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_impl_item(&mut self, ctx: &LateContext<'tcx>, impl_item: &'tcx ImplItem<'tcx>) {
        if let ImplItemKind::Fn(_, body_id) = &impl_item.kind {
            let item_name = impl_item.ident.to_string();

            // Get the module path using the original code's approach
            let impl_block = ctx.tcx.hir_get_parent_item(impl_item.owner_id.into());
            let module = ctx.tcx.hir_get_parent_item(impl_block.into());
            let module_path = get_full_module_name(&ctx.tcx, &module);
            let fn_def_id = impl_item.owner_id.to_def_id();

            // Check if this method matches our patterns
            if !self.matches_function(ctx, &module_path, &item_name, fn_def_id) {
                return;
            }

            // Apply rules
            for rule in &self.function_rules {
                match rule {
                    FunctionRule::MaxLength(max_lines, severity) => {
                        let body = ctx.tcx.hir_body(*body_id);
                        let source_map = ctx.tcx.sess.source_map();

                        if let Ok(file_lines) = source_map.span_to_lines(body.value.span) {
                            if file_lines.lines.len() > *max_lines {
                                // Create a span that only covers the method signature
                                let sig_span = impl_item.span.with_hi(
                                    impl_item.span.lo() + BytePos((item_name.len() + 5) as u32), // "fn name"
                                );

                                span_lint_and_help(
                                    ctx,
                                    FUNCTION_LINT::get_by_severity(*severity),
                                    self.name().as_str(),
                                    sig_span,
                                    format!(
                                        "Function exceeds maximum length of {} lines with {} lines",
                                        max_lines,
                                        file_lines.lines.len()
                                    ),
                                    None,
                                    "Consider breaking this function into smaller parts",
                                );
                            }
                        }
                    }
                    FunctionRule::ResultErrorMustImplementError(severity) => {
                        // Get the return type
                        let fn_sig = ctx.tcx.fn_sig(fn_def_id).skip_binder();
                        let return_ty = fn_sig.output().skip_binder();

                        // Check if it's a Result type
                        if let TyKind::Adt(adt_def, substs) = return_ty.kind() {
                            let path = ctx.tcx.def_path_str(adt_def.did());

                            // If it's a Result type
                            if path.contains("result::Result") && substs.len() >= 2 {
                                let error_ty = substs[1].expect_ty();
                                let param_env = ctx.param_env;

                                // Check if error type does NOT implement Error trait
                                if !implements_error_trait(ctx.tcx, param_env, error_ty) {
                                    let error_type_name = error_ty.to_string();

                                    // Create a span that only covers the method signature
                                    let sig_span = impl_item.span.with_hi(
                                        impl_item.span.lo() + BytePos((item_name.len() + 5) as u32), // "fn name"
                                    );

                                    span_lint_and_help(
                                        ctx,
                                        FUNCTION_LINT::get_by_severity(*severity),
                                        self.name().as_str(),
                                        sig_span,
                                        format!(
                                            "Error type '{}' in Result does not implement Error trait",
                                            error_type_name
                                        ),
                                        None,
                                        "Consider implementing the Error trait for this type or using a type that already implements it",
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
