// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

// Module to test NoAllocation lint

// This function should pass - no allocations
pub fn pure_math(x: i32, y: i32) -> i32 {
    x + y
}

// This function should fail - allocates a Box
pub fn allocates_box() -> Box<i32> {
    Box::new(42)
}

// This function should fail - allocates a Vec
pub fn allocates_vec() -> Vec<i32> {
    Vec::new()
}

// Helper function that allocates - this will be flagged
fn helper_that_allocates(x: i32) -> String {
    x.to_string()
}

// This function should fail - calls a function that allocates (transitive)
pub fn calls_allocating_helper(x: i32) -> String {
    helper_that_allocates(x)
}

// Chain of calls to demonstrate transitive detection
fn deep_helper() -> Vec<i32> {
    Vec::new()
}

fn middle_helper() -> Vec<i32> {
    deep_helper()
}

// This should fail - indirectly allocates through multiple levels
pub fn deeply_nested_allocation() -> Vec<i32> {
    middle_helper()
}
