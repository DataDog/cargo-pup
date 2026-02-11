// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_is_unsafe_match
//@compile-flags: --crate-type lib

// This test verifies that the FunctionLint's IsUnsafe matcher works correctly

// Unsafe function that should trigger the lint (unsafe functions are forbidden)
unsafe fn forbidden_unsafe_function() { //~ ERROR: Function 'forbidden_unsafe_function' is forbidden by lint rule
}

// Unsafe function with return type
unsafe fn forbidden_unsafe_with_return() -> i32 { //~ ERROR: Function 'forbidden_unsafe_with_return' is forbidden by lint rule
    42
}

// Regular safe function that should NOT trigger the lint
fn allowed_safe_function() {
}

// Safe function with return type
fn allowed_safe_with_return() -> i32 {
    42
}

// Unsafe method in impl block
struct TestStruct;

impl TestStruct {
    unsafe fn unsafe_method(&self) { //~ ERROR: Function 'unsafe_method' is forbidden by lint rule
    }

    fn safe_method(&self) {
    }
}

// Unsafe functions in traits
trait UnsafeTrait {
    unsafe fn trait_unsafe_method(&self);

    fn trait_safe_method(&self);
}

impl UnsafeTrait for TestStruct {
    unsafe fn trait_unsafe_method(&self) { //~ ERROR: Function 'trait_unsafe_method' is forbidden by lint rule
    }

    fn trait_safe_method(&self) {
    }
}
