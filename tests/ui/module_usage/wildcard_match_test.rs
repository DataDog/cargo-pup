//@compile-flags: --crate-name wildcard_match
//@compile-flags: --crate-type lib

// Test demonstrating wildcard matches in import paths

// Module with imports that should only allow certain modules
mod test_wildcard_match {

    // This import is denied by the regex pattern
    use std::collections::HashMap; //~ ERROR: Use of module 'std::collections::HashMap' is denied; ["^std::.*"] are not permitted.

    // This import is also denied by the regex pattern
    use std::env; //~ ERROR: Use of module 'std::env' is denied; ["^std::.*"] are not permitted.

    fn test_function() {
        let mut map = HashMap::new();
        map.insert("key", "value");
        let _ = env::var("HOME");
    }
}
