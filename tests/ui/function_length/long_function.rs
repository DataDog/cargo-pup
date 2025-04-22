//@compile-flags: --crate-name test
//@compile-flags: --crate-type lib

// This function has more than 5 lines and should trigger a lint error
fn too_long_function() { //~ ERROR: Function exceeds maximum length of 5 lines with 9 lines
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
}

// This function is within the limit and should not trigger an error
fn acceptable_length() {
    let a = 1;
    let b = 2;
    let c = 3;
}

// Methods in impl blocks should also be checked
struct MyStruct;

impl MyStruct {
    // This method exceeds the length limit
    fn too_long_method(&self) { //~ ERROR: Function exceeds maximum length of 5 lines with 9 lines
        let a = 1;
        let b = 2;
        let c = 3;
        let d = 4;
        let e = 5;
        let f = 6;
        let g = 7;
    }
    
    // This method is within the limit
    fn acceptable_method(&self) {
        let a = 1;
        let b = 2;
        let c = 3;
    }
} 