//@compile-flags: --crate-name test_function_lint
//@compile-flags: --crate-type lib

// This test verifies that the FunctionLint's MaxLength rule works correctly

// ====== Basic Function Length Tests ======

// Function exceeds the 5-line limit (rule: function_length_test)
fn too_long_function() { //~ ERROR: Function exceeds maximum length of 5 lines with 6 lines
    let a = 1;
    let b = 2;
    let c = 3;
    println!("This function has too many lines");
}

// This function is within the limit and should not trigger an error
fn acceptable_length() {
    let a = 1;
    let b = 2;
    let c = 3;
}

// ====== Name-based Function Length Tests ======

// This function matches the name-based rule (3 lines max for functions with "another_long" prefix)
fn another_long_function() { //~ ERROR: Function exceeds maximum length of 3 lines with 5 lines
    let x = 1;
    let y = 2;
    println!("This is also too long");
}

// This function doesn't match the name pattern and won't trigger an error
fn long_but_ok() {
    println!("This is OK");
}

// ====== Methods in Structs ======

pub struct TestStruct;

impl TestStruct {
    // This method is too long for the general 5-line rule
    fn too_long_method(&self) { //~ ERROR: Function exceeds maximum length of 5 lines with 6 lines
        let a = 1;
        let b = 2;
        let c = 3;
        println!("This method has too many lines");
    }
    
    // This method matches the name-based rule and exceeds the 3-line limit
    fn another_long_method(&self) { //~ ERROR: Function exceeds maximum length of 3 lines with 5 lines
        let x = 1;
        let y = 2;
        println!("This is also too long");
    }
    
    // This method is OK (3 lines total)
    fn short_method(&self) {
        println!("This method is OK");
    }
}

// ====== Module-based Function Length Tests ======

mod inner_module {
    // Even a tiny function exceeds the 2-line limit for this module
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