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

pub mod needs_nonempty_mod_file {} //~ ERROR: Module's mod.rs file must not be empty

pub mod needs_wildcard_import { //~ ERROR: Wildcard imports are required here
    pub use std::fmt;
}

pub mod mixed_wildcard_and_explicit_imports {
    pub use std::fmt::*;
    pub use std::io;
}

pub mod import_composite_still_checked {
    pub use std::io; //~ ERROR: Use of this module is not permitted by the configured import restrictions
}
