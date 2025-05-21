//@compile-flags: --crate-name test_must_not_be_named
//@compile-flags: --crate-type lib

// This test verifies that the StructRule::MustNotBeNamed rule works correctly by
// flagging structs that match the forbidden naming pattern "Bad*".

// These structs match the forbidden pattern, should trigger errors
pub struct BadName { //~ ERROR: Struct must not match pattern 'Bad*'
    field: i32,
}

pub struct BadStruct { //~ ERROR: Struct must not match pattern 'Bad*'
    value: String,
}

// These structs don't match the forbidden pattern, shouldn't trigger errors
pub struct GoodName {
    data: Vec<u8>,
}

pub struct AcceptableName {
    id: u64,
    name: String,
} 