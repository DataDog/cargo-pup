//@compile-flags: --crate-name test
//@compile-flags: --crate-type lib

// Define traits for testing different aspects of the lint
pub trait MyTrait {
    fn do_something(&self);
}

pub trait AnotherTrait {
    fn another_method(&self);
}

// Test name pattern constraint (should fail)
pub struct BadlyNamedStruct { //~ ERROR: Struct 'BadlyNamedStruct' does not match the required pattern '.*MyTraitImpl$'
    value: String,
}

impl MyTrait for BadlyNamedStruct {
    fn do_something(&self) {
        println!("This implementation has incorrect naming");
    }
}

// Test visibility constraint (should fail)
pub struct VisibilityImpl { //~ ERROR: Struct 'VisibilityImpl' is public, but should be private
    value: String,
}

impl AnotherTrait for VisibilityImpl {
    fn another_method(&self) {
        println!("This implementation has incorrect visibility");
    }
}

// Test compliant implementations (should not fail)
struct GoodMyTraitImpl {
    value: String,
}

impl MyTrait for GoodMyTraitImpl {
    fn do_something(&self) {
        println!("This implementation follows all constraints");
    }
}

struct GoodImpl {
    value: String,
}

impl AnotherTrait for GoodImpl {
    fn another_method(&self) {
        println!("This implementation follows all constraints");
    }
}