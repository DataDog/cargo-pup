//@compile-flags: --crate-name test_must_be_empty_nested
//@compile-flags: --crate-type lib

// Parent module (not targeted by lint, allowed to have content)
pub mod parent_module {
    // This module can have content, it's not targeted
    pub fn allowed_function() {
        println!("This is allowed in the parent");
    }
    
    // Nested module that IS targeted by the lint (should trigger an error)
    pub mod nested_target { //~ ERROR: Module must be empty
        pub fn invalid_function() {
            println!("This shouldn't be here");
        }
        
        pub struct InvalidStruct;
    }
    
    // Another nested module (not targeted, allowed to have content)
    pub mod another_nested {
        pub const ALLOWED_CONST: &str = "This is fine";
    }
} 