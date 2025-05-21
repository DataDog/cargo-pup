//@compile-flags: --crate-name test_module_based
//@compile-flags: --crate-type lib

// This test verifies that functions in specific module paths can be targeted

// This function is in the root and doesn't trigger the module-based rule
fn root_function_too_long() {
    let a = 1;
    let b = 2;
    let c = 3;
    println!("This is not matched by the module-based rule");
}

// This function in the root is fine (not matched by any rule)
fn root_function_ok() {
    println!("This is OK");
}

// ====== Inner Module Tests ======

mod inner_module {
    // This function exceeds the 2-line limit for this module
    pub fn tiny_but_too_long() { //~ ERROR: Function exceeds maximum length of 2 lines with 3 lines
        println!("Just one line makes this too long");
    }
    
    // This function is OK (2 lines total)
    pub fn just_right() {}
    
    // A nested module that should also have the 2-line limit
    pub mod nested {
        // This exceeds the 2-line limit inherited from the parent module path rule
        pub fn nested_too_long() { //~ ERROR: Function exceeds maximum length of 2 lines with 3 lines
            println!("This violates the 2-line limit");
        }
        
        // This is OK (2 lines)
        pub fn nested_fine() {}
    }
}

// This module should have the default limit
mod unaffected_module {
    // This function would violate the inner_module's 2-line limit,
    // but it's fine under the default 5-line limit
    pub fn medium_function() {
        let x = 1;
        println!("This is OK for a 5-line limit");
    }
} 