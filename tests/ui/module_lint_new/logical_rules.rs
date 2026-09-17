// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_logical_module_rules
//@compile-flags: --crate-type lib

pub mod and_bad {} //~ ERROR: Module must match pattern '^and_good$', found 'and_bad'
//~^ ERROR: Module must not be empty

pub mod and_good {
    pub const VALUE: usize = 1;
}

pub mod or_named {}

pub mod or_content {
    pub const VALUE: usize = 1;
}

pub mod or_bad {} //~ ERROR: Module must match pattern '^or_named$', found 'or_bad'
//~^ ERROR: Module must not be empty

pub mod must_be_empty {}

pub mod not_bad { //~ ERROR: Module must be empty
    pub const VALUE: usize = 1;
}
