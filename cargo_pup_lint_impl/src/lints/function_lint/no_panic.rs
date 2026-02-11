// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use rustc_hir::def_id::DefId;
use rustc_middle::mir::{AssertKind, Body, TerminatorKind};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use std::collections::{HashMap, HashSet};

/// Resolves a span to its original call site if it comes from a macro expansion.
/// This is important for macros like panic!(), unreachable!(), etc. where the MIR
/// span points to the macro definition, not where it was called.
fn resolve_span_to_callsite(span: Span) -> Span {
    // Use source_callsite() to get the original call site, walking through macro expansions
    span.source_callsite()
}

/// Internal categorization of panic sources
/// Note: MIR-level analysis cannot distinguish between panic!(), unreachable!(),
/// unimplemented!(), todo!(), and assert!() macros - they all compile to similar
/// underlying panic functions and are grouped under ExplicitPanic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanicCategory {
    /// Option/Result unwrap/expect calls
    Unwrap,
    /// All panic-family macros: panic!(), unreachable!(), unimplemented!(), todo!(), assert!()
    ExplicitPanic,
    /// Index bounds checking panics
    IndexBounds,
}

/// Classifies a function path into a panic category
pub fn classify_panic_source(path: &str) -> Option<PanicCategory> {
    // Option::unwrap/expect
    if (path.contains("Option::<") || path.contains("::Option::"))
        && (path.ends_with(">::unwrap") || path.ends_with(">::expect"))
    {
        return Some(PanicCategory::Unwrap);
    }

    // Result::unwrap/expect/unwrap_err/expect_err
    if (path.contains("Result::<") || path.contains("::Result::"))
        && (path.ends_with(">::unwrap")
            || path.ends_with(">::expect")
            || path.ends_with(">::unwrap_err")
            || path.ends_with(">::expect_err"))
    {
        return Some(PanicCategory::Unwrap);
    }

    // Index bounds - slice/array index panics
    if path.contains("slice_index")
        || path.contains("index_len_fail")
        || path.contains("slice_start_index")
        || path.contains("slice_end_index")
    {
        return Some(PanicCategory::IndexBounds);
    }

    // All panic-family functions: panic!(), unreachable!(), unimplemented!(), todo!(), assert!()
    // At MIR level these all compile to similar underlying panic functions
    if path.contains("core::panicking::")
        || path.contains("std::panicking::")
        || path.contains("begin_panic")
        || path.contains("panic_fmt")
        || path.contains("unreachable")
        || path.contains("assert_failed")
    {
        return Some(PanicCategory::ExplicitPanic);
    }

    None
}

/// Represents a violation of the no-panic rule
#[derive(Debug)]
pub struct PanicViolation {
    pub span: Span,
    pub reason: String,
}

/// Detects panic-inducing calls in a function's MIR, filtered by categories.
/// Performs transitive analysis - if function A calls B which panics, A is flagged.
pub fn detect_panic_in_mir<'tcx>(
    tcx: TyCtxt<'tcx>,
    mir: &Body<'tcx>,
    categories: &HashSet<PanicCategory>,
) -> Option<PanicViolation> {
    let mut cache = HashMap::new();
    analyze_mir(tcx, mir, &mut cache, categories)
}

/// Core MIR analysis. Separate function to allow recursive calls with shared cache.
fn analyze_mir<'tcx>(
    tcx: TyCtxt<'tcx>,
    mir: &Body<'tcx>,
    cache: &mut HashMap<DefId, bool>,
    categories: &HashSet<PanicCategory>,
) -> Option<PanicViolation> {
    for (_bb, bb_data) in mir.basic_blocks.iter_enumerated() {
        let Some(terminator) = &bb_data.terminator else {
            continue;
        };

        match &terminator.kind {
            // Check Assert terminators - these are compiler-inserted checks (bounds, overflow, etc.)
            // Note: The assert!() macro does NOT use this - it compiles to function calls
            TerminatorKind::Assert { msg, .. } => {
                // Only check bounds checks for NoIndexPanic
                // Overflow/division checks are implicit compiler checks, not explicit user assertions
                if let AssertKind::BoundsCheck { .. } = &**msg
                    && categories.contains(&PanicCategory::IndexBounds)
                {
                    return Some(PanicViolation {
                        span: resolve_span_to_callsite(terminator.source_info.span),
                        reason: "index bounds check may panic".to_string(),
                    });
                }
            }

            // Check function calls
            TerminatorKind::Call { func, args, .. } => {
                // Extract function DefId using const_fn_def
                if let Some((callee_def_id, _generics)) = func.const_fn_def() {
                    let path = tcx.def_path_str(callee_def_id);

                    // Check arguments for closures that might panic
                    for arg in args.iter() {
                        use rustc_middle::mir::Operand;

                        // Try to extract closure DefId from the operand
                        let closure_def_id = match &arg.node {
                            Operand::Constant(constant) => {
                                // Check if this is a closure type
                                let ty = constant.const_.ty();
                                if let rustc_middle::ty::TyKind::Closure(def_id, _) = ty.kind() {
                                    Some(*def_id)
                                } else if let rustc_middle::ty::TyKind::FnDef(def_id, _) = ty.kind()
                                {
                                    Some(*def_id)
                                } else {
                                    None
                                }
                            }
                            Operand::Move(place) | Operand::Copy(place) => {
                                // For Move/Copy operands, check the type of the place
                                let ty = place.ty(mir, tcx).ty;
                                if let rustc_middle::ty::TyKind::Closure(def_id, _) = ty.kind() {
                                    Some(*def_id)
                                } else if let rustc_middle::ty::TyKind::FnDef(def_id, _) = ty.kind()
                                {
                                    Some(*def_id)
                                } else {
                                    None
                                }
                            }
                            // RuntimeChecks are UB checks inserted by the compiler, not relevant
                            Operand::RuntimeChecks(_) => None,
                        };

                        if let Some(closure_def_id) = closure_def_id {
                            // Analyze the closure if it's local
                            if closure_def_id.krate == rustc_hir::def_id::LOCAL_CRATE
                                && tcx.is_mir_available(closure_def_id)
                                && function_panics_with_categories(
                                    tcx,
                                    closure_def_id,
                                    cache,
                                    categories,
                                )
                            {
                                return Some(PanicViolation {
                                    span: resolve_span_to_callsite(terminator.source_info.span),
                                    reason: format!("passes panicking closure to {path}"),
                                });
                            }
                        }
                    }

                    // Check if it's a known panicking function in the requested categories
                    if let Some(category) = classify_panic_source(&path)
                        && categories.contains(&category)
                    {
                        return Some(PanicViolation {
                            span: resolve_span_to_callsite(terminator.source_info.span),
                            reason: format!("calls panicking function: {path}"),
                        });
                    }

                    // Check transitively (with cycle detection)
                    if should_analyze_transitively(tcx, callee_def_id)
                        && function_panics_with_categories(tcx, callee_def_id, cache, categories)
                    {
                        return Some(PanicViolation {
                            span: resolve_span_to_callsite(terminator.source_info.span),
                            reason: format!("calls function that may panic: {path}"),
                        });
                    }
                }
            }

            // Other terminator kinds don't represent panics we're looking for
            _ => {}
        }
    }

    None
}

/// Determines if we should recursively analyze a function
fn should_analyze_transitively(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    // Only analyze functions in the local crate
    // External crates are harder to analyze and may not have MIR available
    def_id.krate == rustc_hir::def_id::LOCAL_CRATE && tcx.is_mir_available(def_id)
}

/// Recursively checks if a function panics with specific categories, with memoization
fn function_panics_with_categories<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: DefId,
    cache: &mut HashMap<DefId, bool>,
    categories: &HashSet<PanicCategory>,
) -> bool {
    // Check cache
    if let Some(&result) = cache.get(&def_id) {
        return result;
    }

    // Mark as false initially (cycle detection)
    cache.insert(def_id, false);

    // Try to get MIR
    if !tcx.is_mir_available(def_id) {
        // Conservative: assume external functions don't panic
        // This prevents false positives for standard library functions
        return false;
    }

    let mir = tcx.optimized_mir(def_id);
    let panics = analyze_mir(tcx, mir, cache, categories).is_some();

    // Update cache with actual result
    cache.insert(def_id, panics);
    panics
}
