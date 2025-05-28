// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_restrict_imports
//@compile-flags: --crate-type lib

// Define dummy modules to be used for imports
pub mod dummy_denied {
    pub fn anything() {}
}

// Allowed imports (std and core are in allowed list)
use std::fmt;
use std::io;
use core::ops::Add;

// Disallowed import (not in allowed list) - will trigger both rules
use crate::dummy_denied::anything; //~ ERROR: Use of module 'crate::dummy_denied::anything' is not allowed; only ["std::*", "core::*"] are permitted
                                  //~^ ERROR: Use of module 'crate::dummy_denied::anything' is denied

// Nested modules with allowed imports
pub mod nested {
    // These should also be checked against the rules
    use std::collections::HashMap; // Allowed - in allowed list
} 
