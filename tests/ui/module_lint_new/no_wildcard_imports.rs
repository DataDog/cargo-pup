//@compile-flags: --crate-name test_no_wildcard_imports
//@compile-flags: --crate-type lib

// Specific imports (allowed)
use std::fmt::Display;
use std::collections::HashMap;
use std::io::Read;

// Wildcard imports (should trigger errors)
use std::fmt::*; //~ ERROR: Wildcard imports are not allowed
use std::collections::*; //~ ERROR: Wildcard imports are not allowed

// Nested modules with wildcard imports
pub mod inner_module {
    // Specific import (allowed)
    use std::sync::Arc;
    
    // Wildcard import (now checked by the linter in nested modules)
    use std::sync::*; //~ ERROR: Wildcard imports are not allowed
} 