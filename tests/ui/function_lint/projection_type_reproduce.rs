//@compile-flags: --crate-name projection_type_reproduce
//@compile-flags: --crate-type lib

use std::fmt;

// A trait with an associated error type
trait MyTrait {
    type Error: std::error::Error + Default;
}

// A concrete error type
#[derive(Debug, Default)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError")
    }
}

impl std::error::Error for MyError {}

impl MyTrait for () {
    type Error = MyError;
}

// This is the function under lint
fn test<T: MyTrait>() -> Result<(), T::Error> //~ ERROR: Function exceeds maximum length of 1 lines with 4 line
where
    T::Error: std::error::Error + Default,
{
    // This triggers the check: does `<T as MyTrait>::Error` implement `Error`?
    Err(T::Error::default())
}