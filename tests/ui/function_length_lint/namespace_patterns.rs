//@compile-flags: --crate-name namespace_patterns
//@compile-flags: --crate-type lib

// This module should be matched by the pattern in the pup.ron file
pub mod my_module {
    // This function exceeds the length limit and should trigger an error
    pub fn too_long_function() { //~ ERROR: Function exceeds maximum length of 5 lines with 9 lines
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        let e = 5;
        let f = 6;
        let g = 7;
    }
    
    // This function is within the limit and should not trigger an error
    pub fn acceptable_length() {
        let a = 1;
        let b = 2;
        let c = 3;
    }
    
    // Methods in impl blocks should also be checked
    pub struct MyStruct;
    
    impl MyStruct {
        // This method exceeds the length limit
        pub fn too_long_method(&self) { //~ ERROR: Function exceeds maximum length of 5 lines with 9 lines
            let a = 1;
            let b = 2;
            let c = 3;
            let d = 4;
            let e = 5;
            let f = 6;
            let g = 7;
        }
        
        // This method is within the limit
        pub fn acceptable_method(&self) {
            let a = 1;
            let b = 2;
            let c = 3;
        }
    }
}

// This module should NOT match the pattern
pub mod other_module {
    // This function exceeds the length limit but shouldn't trigger an error
    pub fn too_long_function_no_error() {
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        let e = 5;
        let f = 6;
        let g = 7;
    }
} 