//@compile-flags: --crate-name test_returns_result
//@compile-flags: --crate-type lib

// This function returns Result<(), i32>
fn function_with_result() -> Result<(), i32> {
    Ok(())
}

// This function returns Result with a custom error type
fn function_with_custom_result() -> Result<String, MyError> {
    Ok("Success".to_string())
}

// This function returns Option<i32>, not Result
fn function_with_option() -> Option<i32> {
    Some(42)
}

// This function returns a String, not Result
fn function_with_string() -> String {
    "Not a result".to_string()
}

// This function returns a custom type, not Result
fn function_with_custom_type() -> CustomType {
    CustomType { value: 10 }
}

// Custom error type
struct MyError {
    message: String,
}

// Custom return type
struct CustomType {
    value: i32,
} 