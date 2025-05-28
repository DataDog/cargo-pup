// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_implements_trait
//@compile-flags: --crate-type lib

// This test verifies that the StructMatch::ImplementsTrait matcher works correctly by
// flagging structs that implement a specific trait.

// Define a test trait
pub trait TestTrait {
    fn test_method(&self) -> bool;
}

// Another trait that shouldn't be matched by our lint
pub trait UnrelatedTrait {
    fn unrelated_method(&self) -> i32;
}

// This struct implements TestTrait and should trigger the lint
pub struct ImplementsTestTrait { //~ ERROR: Struct must match pattern 'Compliant*', found 'ImplementsTestTrait'
    field: i32,
}

impl TestTrait for ImplementsTestTrait {
    fn test_method(&self) -> bool {
        true
    }
}

// This struct implements both traits and should also trigger the lint
pub struct ImplementsMultipleTraits { //~ ERROR: Struct must match pattern 'Compliant*', found 'ImplementsMultipleTraits'
    value: String,
}

impl TestTrait for ImplementsMultipleTraits {
    fn test_method(&self) -> bool {
        false
    }
}

impl UnrelatedTrait for ImplementsMultipleTraits {
    fn unrelated_method(&self) -> i32 {
        42
    }
}

// This struct doesn't implement TestTrait, should not trigger the lint
pub struct DoesNotImplementTestTrait {
    data: Vec<u8>,
}

impl UnrelatedTrait for DoesNotImplementTestTrait {
    fn unrelated_method(&self) -> i32 {
        123
    }
}

// This struct implements TestTrait but has a compliant name, so it shouldn't trigger an error
pub struct CompliantTraitImplementor {
    id: u64,
}

impl TestTrait for CompliantTraitImplementor {
    fn test_method(&self) -> bool {
        true
    }
} 
