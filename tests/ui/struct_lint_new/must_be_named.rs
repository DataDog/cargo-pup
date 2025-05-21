//@compile-flags: --crate-name test_must_be_named
//@compile-flags: --crate-type lib

// This test verifies that the StructRule::MustBeNamed rule works correctly by
// flagging structs that don't match the naming pattern "Good*".

// These structs don't match the required pattern, should trigger errors
pub struct IncorrectName { //~ ERROR: Struct must match pattern 'Good*', found 'IncorrectName'
    field: i32,
}

pub struct AnotherBadName { //~ ERROR: Struct must match pattern 'Good*', found 'AnotherBadName'
    value: String,
}

// These structs match the pattern, shouldn't trigger errors
pub struct GoodName {
    data: Vec<u8>,
}

pub struct GoodStruct {
    id: u64,
    name: String,
} 