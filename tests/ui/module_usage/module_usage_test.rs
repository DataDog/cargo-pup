//@compile-flags: --crate-name module_usage
//@compile-flags: --crate-type lib

// Test demonstrating the module_usage lint capabilities
// This shows both "DenyWildcard" and "Deny" rule types

// Module with a wildcard import that triggers DenyWildcard
mod test_wildcard {
    // This import will be flagged by the DenyWildcard rule
    use std::io::*; //~ ERROR: Use of wildcard imports in 'std::io' is denied.

    // Regular imports are fine
    use std::fmt;

    fn test_function() {
        let _x = 42;
    }
}

// Module with an import that triggers Deny rule
mod test_denied {
    // This import will be flagged by the Deny rule for collections
    use std::collections::HashMap; //~ ERROR: Use of module 'std::collections::HashMap' is denied; ["std::collections"] are not permitted.

    // Other imports are allowed
    use std::env;

    fn test_function() {
        let mut map = HashMap::new();
        map.insert("key", "value");
    }
}
