//@compile-flags: --crate-name test
//@compile-flags: --crate-type lib

// This is a mod.rs file with forbidden items that should trigger the empty_mod lint

// Re-exports are allowed in mod.rs
pub use std::collections::HashMap;

// This struct definition should trigger the lint
pub struct ForbiddenStruct {
    pub field: String,
}

// This enum definition should trigger the lint
pub enum ForbiddenEnum {
    VariantA,
    VariantB,
}

// This trait definition should trigger the lint
pub trait ForbiddenTrait {
    fn method(&self);
}

// This struct impl should trigger the lint (non-trait impl)
impl ForbiddenStruct {
    pub fn new(value: String) -> Self {
        Self { field: value }
    }
}

// Functions should be allowed according to the lint implementation
pub fn this_should_be_allowed() {
    println!("This function should not trigger the lint");
}
