// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_must_not_be_empty
//@compile-flags: --crate-type lib

// Non-empty module (allowed)
pub mod non_empty_module {
    pub fn valid_function() {
        println!("This module has content so it's valid");
    }
    
    pub const VALID_CONST: &str = "Also valid content";
}

// Empty module (should trigger an error)
pub mod empty_module { //~ ERROR: Module must not be empty
}

// Another empty module (should also trigger an error)
pub mod another_empty_module { //~ ERROR: Module must not be empty
} 
