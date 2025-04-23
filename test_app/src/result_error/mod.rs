use anyhow::{Result as AnyhowResult, anyhow, Error};

// This should be allowed - String implements Error
pub fn good_result() -> Result<String, String> {
    Ok("good".to_string())
}

// This should be allowed - std::io::Error implements Error
pub fn good_io_result() -> Result<String, std::io::Error> {
    Ok("good".to_string())
}

// This should trigger a warning - i32 doesn't implement Error
pub fn bad_result() -> Result<String, i32> {
    Ok("bad".to_string())
}

// This should trigger a warning - custom type doesn't implement Error
pub struct CustomError {
    message: String,
}

pub fn custom_error_result() -> Result<String, CustomError> {
    Ok("bad".to_string())
}

// This should be allowed - custom type implements Error
#[derive(Debug)]
pub struct GoodCustomError {
    message: String,
}

impl std::error::Error for GoodCustomError {}

impl std::fmt::Display for GoodCustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn good_custom_error_result() -> Result<String, GoodCustomError> {
    Ok("good".to_string())
}

// New examples using anyhow

// This should be allowed - anyhow::Error implements Error
pub fn anyhow_result() -> AnyhowResult<String> {
    Ok("good".to_string())
}

// This should be allowed - returning anyhow::Error directly
pub fn anyhow_direct_error() -> Result<String, Error> {
    Ok("good".to_string())
}

// This should be allowed - creating and returning anyhow errors
pub fn anyhow_error_creation() -> AnyhowResult<String> {
    if rand::random::<bool>() {
        return Err(anyhow!("Something went wrong"));
    }
    
    // Chain errors
    std::fs::read_to_string("nonexistent_file.txt")
        .map_err(|e| anyhow!("Failed to read file: {}", e))?;
    
    Ok("good".to_string())
}

// This should trigger a warning - wrapping a non-Error type with anyhow
// but still returning it directly as the error type
pub fn bad_anyhow_usage() -> Result<String, i32> {
    // This would be fine if we returned AnyhowResult instead
    let _ = anyhow!("Just demonstrating anyhow");
    
    Ok("bad".to_string())
}