// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_must_be_empty_nested
//@compile-flags: --crate-type lib

// Parent module (not targeted by lint, allowed to have content)
pub mod parent_module {
    // This module can have content, it's not targeted
    pub fn allowed_function() {
        println!("This is allowed in the parent");
    }
    
    // Nested module that IS targeted by the lint (should trigger an error)
    pub mod nested_target {
        pub fn invalid_function() { //~ ERROR: Item 'invalid_function' not allowed in empty module
            println!("This shouldn't be here");
        }
        
        pub struct InvalidStruct; //~ ERROR: Item 'InvalidStruct' not allowed in empty module
    }
    
    // Another nested module (not targeted, allowed to have content)
    pub mod another_nested {
        pub const ALLOWED_CONST: &str = "This is fine";
    }
} 
