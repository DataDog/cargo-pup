// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_must_be_named
//@compile-flags: --crate-type lib

// This module follows the required naming pattern (good_*)
pub mod good_module {
    pub fn valid_function() {
        println!("This is a valid module name following the pattern");
    }
}

// These modules don't follow the naming pattern and should trigger errors
pub mod incorrect_name { //~ ERROR: Module must match pattern 'good_*', found 'incorrect_name'
    pub fn some_function() {
        println!("This module doesn't follow the naming pattern");
    }
}

pub mod another_bad_name { //~ ERROR: Module must match pattern 'good_*', found 'another_bad_name'
    pub fn another_function() {
        println!("This module also doesn't follow the naming pattern");
    }
} 
