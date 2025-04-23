//@compile-flags: --crate-name test
//@compile-flags: --crate-type lib

// This module simulates a mod.rs file with forbidden items
mod non_empty {
    // This is a re-export which should be allowed in mod.rs
    pub use std::collections::HashMap;
    
    // This struct definition should trigger the lint
    pub struct ForbiddenStruct { //~ ERROR: Item ForbiddenStruct disallowed in mod.rs due to empty-module policy
        pub field: String,
    }
    
    // This enum definition should trigger the lint
    pub enum ForbiddenEnum { //~ ERROR: Item ForbiddenEnum disallowed in mod.rs due to empty-module policy
        VariantA,
        VariantB,
    }
    
    // This trait definition should trigger the lint
    pub trait ForbiddenTrait { //~ ERROR: Item ForbiddenTrait disallowed in mod.rs due to empty-module policy
        fn method(&self);
    }
    
    // This struct impl should trigger the lint
    impl ForbiddenStruct { //~ ERROR: Item ForbiddenStruct disallowed in mod.rs due to empty-module policy
        pub fn new(value: String) -> Self {
            Self { field: value }
        }
    }
}