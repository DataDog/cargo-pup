// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use rustc_hir::def_id::DefId;
use rustc_middle::mir::{Body, TerminatorKind};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use std::collections::HashMap;

/// Represents a violation of the no-allocation rule
#[derive(Debug)]
pub struct AllocationViolation {
    pub span: Span,
    pub reason: String,
}

/// Detects heap allocations in a function's MIR
pub fn detect_allocation_in_mir<'tcx>(
    tcx: TyCtxt<'tcx>,
    mir: &Body<'tcx>,
    _fn_def_id: DefId,
    cache: &mut HashMap<DefId, bool>,
) -> Option<AllocationViolation> {
    // Iterate through basic blocks
    for (_bb, bb_data) in mir.basic_blocks.iter_enumerated() {
        // Check terminator for calls
        if let Some(terminator) = &bb_data.terminator
            && let TerminatorKind::Call { func, args, .. } = &terminator.kind
        {
            // Extract function DefId using const_fn_def
            if let Some((callee_def_id, _generics)) = func.const_fn_def() {
                let path = tcx.def_path_str(callee_def_id);

                // Check arguments for closures
                for arg in args.iter() {
                    use rustc_middle::mir::Operand;

                    // Try to extract closure DefId from the operand
                    let closure_def_id = match &arg.node {
                        Operand::Constant(constant) => {
                            // Check if this is a closure type
                            let ty = constant.const_.ty();
                            if let rustc_middle::ty::TyKind::Closure(def_id, _) = ty.kind() {
                                Some(*def_id)
                            } else if let rustc_middle::ty::TyKind::FnDef(def_id, _) = ty.kind() {
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
                            } else if let rustc_middle::ty::TyKind::FnDef(def_id, _) = ty.kind() {
                                Some(*def_id)
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(closure_def_id) = closure_def_id {
                        // Analyze the closure if it's local
                        if closure_def_id.krate == rustc_hir::def_id::LOCAL_CRATE
                            && tcx.is_mir_available(closure_def_id)
                            && function_allocates(tcx, closure_def_id, cache)
                        {
                            return Some(AllocationViolation {
                                span: terminator.source_info.span,
                                reason: format!("passes allocating closure to {path}"),
                            });
                        }
                    }
                }

                // Check if it's a known allocating function
                if is_allocating_function(&path) {
                    return Some(AllocationViolation {
                        span: terminator.source_info.span,
                        reason: format!("calls allocating function: {path}"),
                    });
                }

                // Check transitively (with cycle detection)
                if should_analyze_transitively(tcx, callee_def_id)
                    && function_allocates(tcx, callee_def_id, cache)
                {
                    return Some(AllocationViolation {
                        span: terminator.source_info.span,
                        reason: format!("calls function that allocates: {path}"),
                    });
                }
            }
        }
    }

    None
}

/// Checks if a function path corresponds to a known allocating function
fn is_allocating_function(path: &str) -> bool {
    // Direct allocation functions - these are the low-level allocators
    if path.contains("alloc::alloc::")
        && (path.contains("::alloc")
            || path.contains("::allocate")
            || path.contains("::exchange_malloc")
            || path.contains("::box_free"))
    {
        return true;
    }

    // Box allocations - check for various Box patterns
    if (path.contains("::Box::") || path.contains("::Box::<")) && path.contains("::new") {
        return true;
    }

    // Vec allocations and operations that may allocate
    if (path.contains("::Vec::") || path.contains("::Vec::<"))
        && (path.contains("::new")
            || path.contains("::with_capacity")
            || path.contains("::push")
            || path.contains("::insert")
            || path.contains("::extend")
            || path.contains("::append")
            || path.contains("::resize")
            || path.contains("::from_elem"))
    {
        return true;
    }

    // String allocations
    if path.contains("::String::")
        && (path.contains("::new")
            || path.contains("::from")
            || path.contains("::from_utf8")
            || path.contains("::from_utf16")
            || path.contains("::push_str")
            || path.contains("::push")
            || path.contains("::insert")
            || path.contains("::insert_str"))
    {
        return true;
    }

    // Format macro and related
    if path.contains("::format") || path.contains("fmt::format") {
        return true;
    }

    // Rc and Arc
    if (path.contains("::Rc::")
        || path.contains("::Rc::<")
        || path.contains("::Arc::")
        || path.contains("::Arc::<"))
        && (path.contains("::new") || path.contains("::clone"))
    {
        return true;
    }

    // Collection types - broader matching
    if (path.contains("HashMap")
        || path.contains("BTreeMap")
        || path.contains("HashSet")
        || path.contains("BTreeSet")
        || path.contains("VecDeque")
        || path.contains("LinkedList")
        || path.contains("BinaryHeap"))
        && (path.contains(">::new")
            || path.contains(">::with_capacity")
            || path.contains(">::insert")
            || path.contains(">::push"))
    {
        return true;
    }

    // to_string, to_owned methods - these allocate
    if path.contains("::to_string") || path.contains("::to_owned") {
        return true;
    }

    // RawVec - internal vec allocator
    if path.contains("RawVec") && (path.contains("::new") || path.contains("::allocate")) {
        return true;
    }

    false
}

/// Determines if we should recursively analyze a function
fn should_analyze_transitively(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    // Only analyze functions in the local crate
    // External crates are harder to analyze and may not have MIR available
    def_id.krate == rustc_hir::def_id::LOCAL_CRATE && tcx.is_mir_available(def_id)
}

/// Recursively checks if a function allocates, with memoization
fn function_allocates<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: DefId,
    cache: &mut HashMap<DefId, bool>,
) -> bool {
    // Check cache
    if let Some(&result) = cache.get(&def_id) {
        return result;
    }

    // Mark as false initially (cycle detection)
    cache.insert(def_id, false);

    // Try to get MIR
    if !tcx.is_mir_available(def_id) {
        // Conservative: assume external functions don't allocate
        // This prevents false positives for standard library functions
        return false;
    }

    let mir = tcx.optimized_mir(def_id);
    let allocates = detect_allocation_in_mir(tcx, mir, def_id, cache).is_some();

    // Update cache with actual result
    cache.insert(def_id, allocates);
    allocates
}
