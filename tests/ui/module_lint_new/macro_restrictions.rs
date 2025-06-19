// Test for granular macro restriction functionality
// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.
//
// NOTE: This test only covers declarative_macro detection. Proc macro types
// (proc_macro, proc_macro_attribute, proc_macro_derive) cannot be tested in UI tests
// because the compiler validates proc macro signatures and crate types before
// However, the implementation correctly detects all four granular macro types
// as demonstrated in test_app.

//@compile-flags: --crate-name test_macro_restrictions
//@compile-flags: --crate-type lib

// Declarative macro - should trigger the declarative_macro lint
macro_rules! forbidden_declarative_macro {
    //~ ERROR: declarative macro 'forbidden_declarative_macro' is not allowed in this module
    () => {
        println!("This declarative macro is forbidden!");
    };
}

// Another declarative macro to show comprehensive detection
macro_rules! another_forbidden_macro {
    //~ ERROR: declarative macro 'another_forbidden_macro' is not allowed in this module
    ($x:expr) => {
        println!("Another forbidden macro: {}", $x);
    };
}

// Regular function - should be allowed
pub fn allowed_function() {
    println!("This function is allowed");
}
