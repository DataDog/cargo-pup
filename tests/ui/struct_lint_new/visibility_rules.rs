// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_visibility_rules
//@compile-flags: --crate-type lib

// This test verifies that the StructRule::MustBePrivate and StructRule::MustBePublic rules work correctly

// Testing MustBePrivate rule:
pub struct InternalData { //~ ERROR: Struct 'InternalData' is public, but must be private
    field: i32,
}

// This should not trigger any warnings/errors (private, as required)
struct CorrectlyPrivate {
    value: String,
}

// Testing MustBePublic rule:
struct HiddenApi { //~ WARN: Struct 'HiddenApi' is private, but must be public
    id: u64,
}

// This should not trigger any warnings/errors (public, as required)
pub struct CorrectlyPublic {
    data: Vec<u8>,
}

// Also testing name-pattern + visibility combination
pub struct InternalModel { //~ ERROR: Struct 'InternalModel' is public, but must be private
    sensitive_data: String,
}

// This should not trigger any warnings/errors (private, as required by pattern match)
struct AnotherInternalModel {
    more_data: bool,
} 
