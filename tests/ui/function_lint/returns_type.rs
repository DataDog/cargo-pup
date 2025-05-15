//@compile-flags: --crate-name test_returns_type
//@compile-flags: --crate-type lib

// This test verifies that the FunctionLint's ReturnsType matcher works correctly

use std::error::Error;
use std::fmt;

// ====== Result Type Tests ======

// Function returns Result<(), i32> and should trigger the ResultErrorMustImplementError rule
// because i32 doesn't implement Error trait
fn test_result_simple() -> Result<(), i32> { //~ ERROR: Error type 'i32' in Result does not implement Error trait
    Ok(())
}

// Function returns Result with a custom error type that doesn't implement Error
fn test_result_custom_error() -> Result<String, MyError> { //~ ERROR: Error type 'MyError' in Result does not implement Error trait
    Ok("Success".to_string())
}

// ====== Option Type Tests ======

// Function returns Option<i32>
fn test_option_simple() -> Option<i32> { //~ ERROR: Function exceeds maximum length of 1 lines with 3 lines
    Some(42)
}

// Function returns Option<String>
fn test_option_string() -> Option<String> { //~ ERROR: Function exceeds maximum length of 1 lines with 3 lines
    Some("test".to_string())
}

// ====== Named Type Tests ======

// Function returns CustomType (tests Named pattern)
fn test_custom_type_function() -> CustomType { //~ ERROR: Function exceeds maximum length of 1 lines with 5 lines
    CustomType {
        value: 42
    }
}

// Function returns MyError (tests Named pattern)
fn test_my_error_function() -> MyError { //~ ERROR: Function exceeds maximum length of 1 lines with 5 lines
    MyError {
        message: "Error".to_string()
    }
}

// ====== Regex Type Tests ======

// Function returns Vec<i32> (tests Regex pattern)
fn test_vec_integers() -> Vec<i32> { //~ ERROR: Function exceeds maximum length of 1 lines with 6 lines
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    return v;
}

// Function returns Vec<String> (tests Regex pattern)
fn test_vec_strings() -> Vec<String> { //~ ERROR: Function exceeds maximum length of 1 lines with 6 lines
    let mut v = Vec::new();
    v.push("Hello".to_string());
    v.push("World".to_string());
    return v;
}

// ====== Module-based Tests ======

mod inner_module {
    // Function in module returns Result
    pub fn module_result_function() -> Result<(), i32> { 
        //~^ ERROR: Function exceeds maximum length of 2 lines with 5 lines
        //~| ERROR: Error type 'i32' in Result does not implement Error trait
        Ok(())
    }
    
    // Function in module returns Option
    pub fn module_option_function() -> Option<i32> { 
        //~^ ERROR: Function exceeds maximum length of 2 lines with 5 lines
        //~| ERROR: Function exceeds maximum length of 1 lines with 5 lines
        Some(42)
    }
}

// ====== Type Definitions ======

pub struct CustomType {
    pub value: i32,
}

// An error type that intentionally doesn't implement Error trait
pub struct MyError {
    pub message: String,
} 