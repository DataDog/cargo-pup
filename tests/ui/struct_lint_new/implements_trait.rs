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

pub trait GenericTrait<T> {
    type Output;
}

pub trait BlanketTrait<T> {}
pub trait ConditionalTrait<T> {}
pub trait MarkerTrait {}
pub trait MultipleTrait<T> {}
pub trait LifetimeTrait<'a> {}
pub trait ConstTrait<const N: usize> {}

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

// Generic trait arguments are inferred when testing implementations.
pub struct ImplementsGenericTrait; //~ ERROR: Struct must match pattern 'GenericCompliant*', found 'ImplementsGenericTrait'

impl GenericTrait<String> for ImplementsGenericTrait {
    type Output = usize;
}

pub struct GenericContainer<T>(T); //~ ERROR: Struct must match pattern 'GenericCompliant*', found 'GenericContainer'
impl<T> GenericTrait<T> for GenericContainer<T> {
    type Output = T;
}

pub struct DoesNotImplementGenericTrait;

pub struct BlanketImplementor; //~ ERROR: Struct must match pattern 'BlanketCompliant*', found 'BlanketImplementor'
impl<T> BlanketTrait<T> for BlanketImplementor {}

pub struct Marker;
impl MarkerTrait for Marker {}

pub struct ConditionalImplementor; //~ ERROR: Struct must match pattern 'ConditionalCompliant*', found 'ConditionalImplementor'
impl<T: MarkerTrait> ConditionalTrait<T> for ConditionalImplementor {}

pub struct MultipleImplementor; //~ ERROR: Struct must match pattern 'MultipleCompliant*', found 'MultipleImplementor'
impl MultipleTrait<u8> for MultipleImplementor {}
impl MultipleTrait<String> for MultipleImplementor {}

pub struct BroadRegexImplementor; //~ ERROR: Struct must match pattern 'BroadCompliant*', found 'BroadRegexImplementor'
impl UnrelatedTrait for BroadRegexImplementor {
    fn unrelated_method(&self) -> i32 {
        0
    }
}

pub struct ImplementsAsRefStr; //~ ERROR: Struct must match pattern 'AsRefCompliant*', found 'ImplementsAsRefStr'

impl AsRef<str> for ImplementsAsRefStr {
    fn as_ref(&self) -> &str {
        ""
    }
}

pub struct ImplementsLifetimeTrait; //~ ERROR: Struct must match pattern 'LifetimeCompliant*', found 'ImplementsLifetimeTrait'
impl<'a> LifetimeTrait<'a> for ImplementsLifetimeTrait {}

pub struct ImplementsConstTrait; //~ ERROR: Struct must match pattern 'ConstCompliant*', found 'ImplementsConstTrait'
impl ConstTrait<8> for ImplementsConstTrait {}
