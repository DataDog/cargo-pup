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