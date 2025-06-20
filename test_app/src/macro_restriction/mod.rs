// Test module for macro restriction functionality

// Declarative macro - should trigger the lint
macro_rules! forbidden_macro {
    () => {
        println!("This macro is forbidden!");
    };
}

// Regular function - should be allowed
pub fn regular_function() {
    println!("This is a regular function");
}