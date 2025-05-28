// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_result_error_impl
//@compile-flags: --crate-type lib

// This test verifies that the ResultErrorMustImplementError rule works correctly

use std::error::Error;
use std::fmt;

// ====== Basic Result Error Tests ======

// Function that returns a Result with a type that doesn't implement Error
fn test_result_error_no_impl() -> Result<(), NoErrorImpl> { 
    //~^ WARN: Error type 'NoErrorImpl' in Result does not implement Error trait
    //~| ERROR: Error type 'NoErrorImpl' in Result does not implement Error trait
    Ok(())
}

// Function that returns a Result with a type that implements Error
fn test_result_error_with_impl() -> Result<(), WithErrorImpl> {
    Ok(())
}

// Function with multi-line body and a Result with a type that doesn't implement Error
fn test_result_error_multi_line() -> Result<String, NoErrorImpl> { 
    //~^ WARN: Error type 'NoErrorImpl' in Result does not implement Error trait
    //~| ERROR: Error type 'NoErrorImpl' in Result does not implement Error trait
    let s = "test".to_string();
    Ok(s)
}

// Function with a more complex Result type that doesn't implement Error
fn test_result_error_complex() -> Result<Vec<String>, ComplexNoErrorImpl> { 
    //~^ WARN: Error type 'ComplexNoErrorImpl' in Result does not implement Error trait
    //~| ERROR: Error type 'ComplexNoErrorImpl' in Result does not implement Error trait
    Ok(vec!["test".to_string()])
}

// ====== Advanced Patterns ======

// This function should have an error because it returns Result without Error impl
// and matches the advanced pattern
fn advanced_no_error_impl() -> Result<(), NoErrorImpl> { //~ ERROR: Error type 'NoErrorImpl' in Result does not implement Error trait
    Ok(())
}

// This function should NOT have an error because it returns Result with Error impl
// and the advanced pattern explicitly excludes functions with Error impl
fn advanced_with_error_impl() -> Result<(), WithErrorImpl> {
    Ok(())
}

// ====== Module-based Tests ======

mod error_module {
    use super::*;
    
    // Function in error_module that returns Result with a non-Error type
    pub fn module_function_no_impl() -> Result<(), NoErrorImpl> { //~ ERROR: Error type 'NoErrorImpl' in Result does not implement Error trait
        Ok(())
    }
    
    // Function in error_module that returns Result with an Error type - should be OK
    pub fn module_function_with_impl() -> Result<(), WithErrorImpl> {
        Ok(())
    }
}

mod unaffected_module {
    use super::*;
    
    // This function should not trigger errors since it's not in the matched module
    pub fn unaffected_function() -> Result<(), NoErrorImpl> {
        Ok(())
    }
}

// ====== Type Definitions ======

// A type that doesn't implement Error
pub struct NoErrorImpl {
    pub code: i32,
}

// A more complex type that doesn't implement Error
pub struct ComplexNoErrorImpl {
    pub code: i32,
    pub details: String,
}

// A type that implements Error
pub struct WithErrorImpl {
    pub message: String,
}

impl fmt::Display for WithErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for WithErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WithErrorImpl {{ message: {} }}", self.message)
    }
}

impl Error for WithErrorImpl {} 
