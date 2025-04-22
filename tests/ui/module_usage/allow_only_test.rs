//@compile-flags: --crate-name allow_only
//@compile-flags: --crate-type lib

// Test demonstrating the AllowOnly rule type of module_usage lint

// Module with imports that should only allow certain modules
mod test_allow_only {
    // These imports are allowed
    use std::fmt;
    use std::io;

    // This import is not in the allowed list and should trigger the lint
    use std::collections::HashMap; //~ ERROR: Use of module 'std::collections::HashMap' is not allowed; only ["std::fmt", "std::io"] are permitted.

    // This import is also not allowed
    use std::env; //~ ERROR: Use of module 'std::env' is not allowed; only ["std::fmt", "std::io"] are permitted.

    fn test_function() {
        let mut map = HashMap::new();
        map.insert("key", "value");
        let _ = std::env::var("HOME");
    }
}
