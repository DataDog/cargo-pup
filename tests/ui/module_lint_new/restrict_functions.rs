//@compile-flags: --crate-name test_restrict_functions
//@compile-flags: --crate-type lib

// This test verifies that the ModuleRule::RestrictFunctions works correctly
// by flagging functions that exceed the maximum allowed length (3 lines in this case)

// The following function is too long (6 lines including signature and closing brace)
// It should trigger the lint with the maximum length of 3 lines
fn too_long_function() { //~ ERROR: Function exceeds maximum length of 3 lines with 6 lines
    let a = 1;
    let b = 2;
    let c = 3;
    println!("This function has too many lines");
}

// This function is ok (3 lines including signature and closing brace)
fn short_function() {
    println!("This function is OK");
}

// Ensure the lint also works for methods inside impls
pub struct TestStruct;

impl TestStruct {
    // This method is too long (6 lines)
    fn too_long_method(&self) { //~ ERROR: Function exceeds maximum length of 3 lines with 6 lines
        let x = 1;
        let y = 2;
        let z = 3;
        println!("This method has too many lines");
    }
    
    // This method is OK (3 lines)
    fn short_method(&self) {
        println!("This method is OK");
    }
} 