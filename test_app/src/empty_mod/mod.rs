/// This should be allowed
pub use std::i32;

///
/// A struct that should not be defined directly in this module.
///
pub struct ThisSHouldntBeHere {
    pub(crate) name: String,
}

///
/// A function that shouldn't be defined directly in this module
///
/// TODO - this is not detected by empty_module but should be
///
pub fn _function_shouldnt_be_here() -> i32 {
    // A function body that should be
    // less than 5 lines long but isn't.
    let a = 1+1;
    let b = 1+1;
    let c = 1+1;
    a + b + c
}