//@compile-flags: --crate-name test_returns_type
//@compile-flags: --crate-type lib

// This test verifies that the FunctionLint's ReturnsType matcher works correctly

// ====== Basic Return Type Tests ======

// Function returns Result<(), i32> and should trigger the lint
fn function_with_result() -> Result<(), i32> {
    Ok(())
}

// Function returns Result with a custom error type
fn function_with_custom_result() -> Result<String, MyError> {
    Ok("Success".to_string())
}

// Function returns Option<i32>, not Result
fn function_with_option() -> Option<i32> {
    Some(42)
}

// Function returns a String, not Result or Option
fn function_with_string() -> String {
    "Not a result".to_string()
}

// Function returns a custom type, not Result or Option
fn function_with_custom_type() -> CustomType {
    CustomType { value: 10 }
}

// Function returns Result and exceeds the line limit (should trigger both matchers)
fn long_function_with_result() -> Result<(), i32> { //~ ERROR: Function exceeds maximum length of 3 lines with 5 lines
    let x = 1;
    let y = x + 1;
    Ok(())
}

// ====== Methods in Structs ======

pub struct TestStruct;

impl TestStruct {
    // Method returns Result
    fn method_with_result(&self) -> Result<(), i32> {
        Ok(())
    }
    
    // Method returns Option
    fn method_with_option(&self) -> Option<i32> {
        Some(42)
    }
    
    // Method returns String (not Result or Option)
    fn method_returns_string(&self) -> String {
        "Not a result".to_string()
    }
}

// ====== Module-based Return Type Tests ======

mod inner_module {
    // Function in module returning Result
    pub fn module_function_result() -> Result<(), i32> { //~ ERROR: Function exceeds maximum length of 2 lines with 3 lines
        Ok(())
    }
    
    // Function in module returning Option
    pub fn module_function_option() -> Option<i32> { //~ ERROR: Function exceeds maximum length of 2 lines with 3 lines
        Some(42)
    }
}

// Custom error type
#[derive(Debug)]
struct MyError {
    message: String,
}

// Custom return type
struct CustomType {
    value: i32,
} 