//@compile-flags: --crate-name namespace_patterns
//@compile-flags: --crate-type lib

// This will not trigger an error, as it is in the root.
pub fn my_function() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
}

mod my_module {
    // This function should trigger the lint because it's in my_module
    pub fn my_module_function() { //~ ERROR: Function exceeds maximum length of 5 lines with 9 lines
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        let e = 5;
        let f = 6;
        let g = 7;
    }

    // This nested module also matches the namespace pattern
    pub struct MyStruct;

    impl MyStruct {
        // This should trigger the lint as it's in my_module
        pub fn too_long_method(&self) { //~ ERROR: Function exceeds maximum length of 5 lines with 9 lines
            let a = 1;
            let b = 2;
            let c = 3;
            let d = 4;
            let e = 5;
            let f = 6;
            let g = 7;
        }
    }
}

mod other_module {
    // This function won't trigger the lint because it's in other_module
    pub fn other_module_function() {
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        let e = 5;
        let f = 6;
        let g = 7;
    }
}