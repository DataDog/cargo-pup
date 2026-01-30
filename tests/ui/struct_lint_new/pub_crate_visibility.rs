// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_pub_crate_visibility
//@compile-flags: --crate-type lib

// This test verifies that the StructRule::MustBePubCrate rule works correctly

// Testing MustBePubCrate rule - struct that is pub but should be pub(crate)
pub struct CrateInternal { //~ ERROR: Struct 'CrateInternal' has pub visibility, but must be pub(crate)
    field: i32,
}

// Testing MustBePubCrate rule - struct that is private but should be pub(crate)
struct AlsoInternal { //~ ERROR: Struct 'AlsoInternal' has private visibility, but must be pub(crate)
    value: String,
}

// This should not trigger any warnings/errors (pub(crate), as required)
pub(crate) struct CorrectlyPubCrate {
    data: Vec<u8>,
}
