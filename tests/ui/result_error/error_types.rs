//@compile-flags: --crate-name test
//@compile-flags: --crate-type lib

mod test {
    use std::fmt;

    // Custom error type that doesn't implement std::error::Error
    struct CustomError {
        message: String,
    }

    // For Display formatting, but not implementing std::error::Error
    impl fmt::Display for CustomError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    // Custom error type that properly implements std::error::Error
    #[derive(Debug)]
    struct GoodCustomError {
        message: String,
    }

    impl std::error::Error for GoodCustomError {}

    impl fmt::Display for GoodCustomError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    // This function should trigger a lint error - i32 doesn't implement Error
    fn bad_primitive_error() -> Result<(), i32> { //~ ERROR: Type 'i32' is used as an error type in Result but does not implement Error trait
        Ok(())
    }

    // This function should trigger a lint error - CustomError doesn't implement Error
    fn bad_custom_error() -> Result<(), CustomError> { //~ ERROR: Type 'test::CustomError' is used as an error type in Result but does not implement Error trait
        Ok(())
    }

    // This function should be allowed - String implements Error
    fn good_string_error() -> Result<(), String> {
        Ok(())
    }

    // This function should be allowed - std::io::Error implements Error
    fn good_io_error() -> Result<(), std::io::Error> {
        Ok(())
    }

    // This function should be allowed - GoodCustomError implements Error
    fn good_custom_error() -> Result<(), GoodCustomError> {
        Ok(())
    }
}