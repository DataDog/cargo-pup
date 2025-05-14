//@compile-flags: --crate-name test_returns_result_error
//@compile-flags: --crate-type lib

// This function returns Result<(), i32> and should trigger the lint (too long)
fn function_with_result_too_long() -> Result<(), i32> {
    // First line
    let x = 1;
    // Second line
    let y = 2;
    // Third line - this will exceed our 2-line limit
    let z = x + y;
    
    Ok(())
}

// This function returns Result with a custom error type and should trigger the lint (too long)
fn function_with_custom_result_too_long() -> Result<String, MyError> {
    // First line
    let message = "Success".to_string();
    // Second line 
    let other = "message".to_string();
    // Third line - this will exceed our 2-line limit
    let combined = format!("{} {}", message, other);
    
    Ok(combined)
}

// This function returns Option<i32>, not Result, so it shouldn't trigger our matcher
fn function_with_option_too_long() -> Option<i32> {
    // First line
    let value = 42;
    // Second line
    let other = 100;
    // Third line
    let result = value + other;
    
    Some(result)
}

// Custom error type
struct MyError {
    message: String,
} 