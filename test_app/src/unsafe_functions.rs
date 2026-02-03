// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//! Test module for IsUnsafe matcher validation
//!
//! This module tests that the IsUnsafe function matcher correctly identifies:
//! - Unsafe free functions
//! - Unsafe methods
//! - Unsafe trait implementations

// ============================================================================
// Functions that should PASS (safe functions - not matched by IsUnsafe)
// ============================================================================

/// Regular safe function
pub fn allowed_safe_function() -> i32 {
    42
}

/// Safe function with complex logic
pub fn allowed_safe_with_logic(x: i32) -> i32 {
    if x > 0 { x * 2 } else { x }
}

// ============================================================================
// Functions that should FAIL (unsafe functions - matched by IsUnsafe)
// ============================================================================

/// Unsafe free function - forbidden
pub unsafe fn forbidden_unsafe_function() {
    // Pretend to do something unsafe
}

/// Unsafe function with return value - forbidden
pub unsafe fn forbidden_unsafe_with_return() -> i32 {
    42
}

/// Unsafe function with parameters - forbidden
pub unsafe fn forbidden_unsafe_with_params(ptr: *const i32) -> i32 {
    *ptr
}

// ============================================================================
// Struct with mixed methods
// ============================================================================

pub struct UnsafeService {
    data: i32,
}

impl UnsafeService {
    /// Safe constructor
    pub fn new(data: i32) -> Self {
        Self { data }
    }

    /// Safe method
    pub fn get_data(&self) -> i32 {
        self.data
    }

    /// Unsafe method - forbidden
    pub unsafe fn get_data_unsafe(&self) -> i32 {
        self.data
    }

    /// Another unsafe method - forbidden
    pub unsafe fn mutate_unsafe(&mut self, ptr: *const i32) {
        self.data = *ptr;
    }
}

// ============================================================================
// Trait with unsafe methods
// ============================================================================

pub trait UnsafeProcessor {
    /// Safe trait method
    fn process_safe(&self) -> i32;

    /// Unsafe trait method
    unsafe fn process_unsafe(&self) -> i32;
}

impl UnsafeProcessor for UnsafeService {
    fn process_safe(&self) -> i32 {
        self.data
    }

    /// Unsafe trait implementation - forbidden
    unsafe fn process_unsafe(&self) -> i32 {
        self.data * 2
    }
}
