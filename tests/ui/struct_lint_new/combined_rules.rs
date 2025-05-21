//@compile-flags: --crate-name test_combined_rules
//@compile-flags: --crate-type lib

// This test verifies that multiple struct lint rules can be combined.
// In this case, structs matching 'TestStruct.*' must:
// 1. Match pattern 'TestStruct[A-Z].*' (start with TestStruct followed by a capital letter)
// 2. Not match pattern '.*Forbidden.*' (must not contain the word "Forbidden")

// Violates rule 1 - doesn't have a capital letter after "TestStruct"
pub struct TestStructlowercase { //~ ERROR: Struct must match pattern 'TestStruct[A-Z].*', found 'TestStructlowercase'
    data: i32,
}

// Violates rule 2 - contains "Forbidden"
pub struct TestStructAForbiddenName { //~ ERROR: Struct must not match pattern '.*Forbidden.*'
    name: String,
}

// Satisfies both rules - starts with "TestStruct" + capital letter, and doesn't contain "Forbidden"
pub struct TestStructValidName {
    id: u32,
    value: String,
}

// Doesn't trigger any rules because it doesn't match the initial selector pattern "TestStruct.*"
pub struct OtherStruct {
    field: bool,
} 