//@compile-flags: --crate-name test_must_be_empty
//@compile-flags: --crate-type lib

// Empty module (allowed)
pub mod empty_module {
    // This module is intentionally empty and should not trigger errors
}

// Another empty module (also allowed)
pub mod another_empty_module {
    // This module is also intentionally empty
}

// Non-empty module (should trigger an error)
pub mod non_empty_module {
    pub fn invalid_function() { //~ ERROR: Item 'invalid_function' not allowed in empty module
        println!("This module should be empty");
    }

    // reproduce for ICE item_name: no name for DefPath
    // use statements don't have a name, and also don't count towards
    // a module being empty.
    pub use std::str;
    
    pub const INVALID_CONST: &str = "Content not allowed"; //~ ERROR: Item 'INVALID_CONST' not allowed in empty module
} 