use std::collections::HashMap;
use std::env;
use std::io::*;

///
/// A function that uses
/// functions that are banned in this module.
///
pub fn _test_fn() -> usize {
    let mut map = HashMap::new(); // Allowed
    map.insert("mykey", "myvalue");
    let current_dir = env::current_dir().unwrap_or_default(); // Denied
    map.len() + current_dir.as_os_str().len()
}