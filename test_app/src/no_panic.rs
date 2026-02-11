// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//! Test module for NoPanic rule validation
//!
//! This module tests that the NoPanic lint correctly identifies:
//! - Direct unwrap/expect calls on Option
//! - Direct unwrap/expect calls on Result
//! - Transitive panic paths through function calls

// ============================================================================
// Functions that should PASS (no panic paths)
// ============================================================================

/// Pure computation - no panic possible
pub fn pure_math(x: i32, y: i32) -> i32 {
    x + y
}

/// Using unwrap_or - safe alternative to unwrap
pub fn safe_option_handling(opt: Option<i32>) -> i32 {
    opt.unwrap_or(0)
}

/// Using unwrap_or_else - safe alternative with closure
pub fn safe_option_with_closure(opt: Option<i32>) -> i32 {
    opt.unwrap_or_else(|| 42)
}

/// Using match - explicit handling of both cases
pub fn safe_result_match(res: Result<i32, &str>) -> i32 {
    match res {
        Ok(v) => v,
        Err(_) => -1,
    }
}

/// Using if let - safe pattern matching
pub fn safe_if_let(opt: Option<i32>) -> i32 {
    if let Some(v) = opt {
        v
    } else {
        0
    }
}

/// Using ok() to convert Result to Option safely
pub fn safe_result_to_option(res: Result<i32, &str>) -> Option<i32> {
    res.ok()
}

// ============================================================================
// Functions that should FAIL (may panic)
// ============================================================================

/// Direct unwrap on Option - will panic if None
pub fn panics_option_unwrap(opt: Option<i32>) -> i32 {
    opt.unwrap()
}

/// Direct expect on Option - will panic if None
pub fn panics_option_expect(opt: Option<i32>) -> i32 {
    opt.expect("expected a value")
}

/// Direct unwrap on Result - will panic if Err
pub fn panics_result_unwrap(res: Result<i32, &str>) -> i32 {
    res.unwrap()
}

/// Direct expect on Result - will panic if Err
pub fn panics_result_expect(res: Result<i32, &str>) -> i32 {
    res.expect("expected success")
}

/// Direct unwrap_err on Result - will panic if Ok
pub fn panics_result_unwrap_err(res: Result<i32, &str>) -> &str {
    res.unwrap_err()
}

/// Direct expect_err on Result - will panic if Ok
pub fn panics_result_expect_err(res: Result<i32, &str>) -> &str {
    res.expect_err("expected error")
}

// ============================================================================
// Transitive panic tests
// ============================================================================

/// Helper function that panics
fn helper_that_panics(opt: Option<i32>) -> i32 {
    opt.unwrap()
}

/// Calls a function that may panic - should be flagged
pub fn calls_panicking_helper(opt: Option<i32>) -> i32 {
    helper_that_panics(opt)
}

/// Deeply nested helper that panics
fn deep_panicking_helper(opt: Option<i32>) -> i32 {
    opt.expect("deep panic")
}

/// Middle helper in call chain
fn middle_helper(opt: Option<i32>) -> i32 {
    deep_panicking_helper(opt)
}

/// Top-level function with deeply nested panic
pub fn deeply_nested_panic(opt: Option<i32>) -> i32 {
    middle_helper(opt)
}

// ============================================================================
// Method tests
// ============================================================================

pub struct PanicService {
    value: Option<i32>,
}

impl PanicService {
    pub fn new(value: Option<i32>) -> Self {
        Self { value }
    }

    /// Safe method using unwrap_or
    pub fn get_safe(&self) -> i32 {
        self.value.unwrap_or(0)
    }

    /// Panicking method using unwrap
    pub fn get_panicking(&self) -> i32 {
        self.value.unwrap()
    }
}

// ============================================================================
// NoPanic rule tests (panic-family macros)
// ============================================================================

/// Explicit panic - forbidden by NoPanic
pub fn uses_panic() {
    panic!("explicit panic");
}

/// Assert macro - forbidden by NoPanic
pub fn uses_assert(x: i32) {
    assert!(x > 0);
}

/// Assert_eq macro - forbidden by NoPanic
pub fn uses_assert_eq(x: i32, y: i32) {
    assert_eq!(x, y);
}

/// Unreachable macro - forbidden by NoPanic
pub fn uses_unreachable() -> i32 {
    unreachable!()
}

/// Unimplemented macro - forbidden by NoPanic
pub fn uses_unimplemented() -> i32 {
    unimplemented!()
}

/// Todo macro - forbidden by NoPanic
pub fn uses_todo() -> i32 {
    todo!()
}

// ============================================================================
// NoIndexPanic rule tests (bounds checking)
// ============================================================================

/// Direct slice indexing - forbidden by NoIndexPanic
pub fn slice_index(arr: &[i32]) -> i32 {
    arr[0]
}

/// Array indexing with variable - forbidden by NoIndexPanic
pub fn array_index(arr: [i32; 3], idx: usize) -> i32 {
    arr[idx]
}

/// Safe alternative using get()
pub fn safe_slice_get(arr: &[i32]) -> Option<&i32> {
    arr.get(0)
}

/// Safe alternative using first()
pub fn safe_slice_first(arr: &[i32]) -> Option<&i32> {
    arr.first()
}
