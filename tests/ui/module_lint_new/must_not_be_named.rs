//@compile-flags: --crate-name test_must_not_be_named
//@compile-flags: --crate-type lib

// These modules are allowed (don't match the bad_* pattern)
pub mod good_module {
    pub fn valid_function() {
        println!("This is an acceptable module name");
    }
}

pub mod another_good_module {
    pub fn another_function() {
        println!("This is also an acceptable module name");
    }
}

// These modules match the forbidden pattern and should trigger errors
pub mod bad_module { //~ ERROR: Module must not match pattern 'bad_*'
    pub fn invalid_function() {
        println!("This module has a forbidden name pattern");
    }
}

pub mod bad_name_module { //~ ERROR: Module must not match pattern 'bad_*'
    pub fn another_invalid_function() {
        println!("This module also has a forbidden name pattern");
    }
} 