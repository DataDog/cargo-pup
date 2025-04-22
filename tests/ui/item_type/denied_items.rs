//@compile-flags: --crate-name test
//@compile-flags: --crate-type lib

// This struct should trigger the lint (denied item type)
pub struct ForbiddenStruct { //~ ERROR: struct 'ForbiddenStruct' is not allowed in this module
    pub field: String,
}

// This enum should trigger the lint (denied item type)
pub enum ForbiddenEnum { //~ ERROR: enum 'ForbiddenEnum' is not allowed in this module
    VariantA,
    VariantB,
}

// This trait should trigger the lint (denied item type)
pub trait ForbiddenTrait { //~ ERROR: trait 'ForbiddenTrait' is not allowed in this module
    fn method(&self);
}

// This nested module should trigger the lint (denied item type)
pub mod nested_module { //~ ERROR: module 'nested_module' is not allowed in this module
    pub fn allowed_function() {
        println!("This function is inside a forbidden module!");
    }
}

// These items should be allowed (not in denied_items list)
pub fn allowed_function() {
    println!("This function is allowed!");
}

pub const ALLOWED_CONST: i32 = 42;

pub static ALLOWED_STATIC: &str = "This static is allowed";

// Impl blocks not for a trait should be allowed
impl ForbiddenStruct {
    pub fn new(value: String) -> Self {
        Self { field: value }
    }
}