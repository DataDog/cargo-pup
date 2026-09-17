// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_logical_module_rules
//@compile-flags: --crate-type lib

pub mod and_bad {} //~ ERROR: Item does not satisfy the configured logical module rule

pub mod and_good {
    pub const VALUE: usize = 1;
}

pub mod or_named {}

pub mod or_content {
    pub const VALUE: usize = 1;
}

pub mod or_bad {} //~ ERROR: Item does not satisfy the configured logical module rule

pub mod must_be_empty {}

pub mod not_bad { //~ ERROR: Item does not satisfy the configured logical module rule
    pub const VALUE: usize = 1;
}
