// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//@compile-flags: --crate-name test_combined_rules
//@compile-flags: --crate-type lib

// This test verifies that logical operators (And, Or, Not) in function matching work correctly

// ====== AND Matching Tests ======

// Function that matches the AND condition in pup.ron (matches both conditions)
fn too_long_with_prefix() {  //~ ERROR: Function exceeds maximum length of 5 lines with 8 lines
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    println!("This is too long and has the prefix we're looking for");
}

// Function that only matches one part of the AND condition (doesn't match the combined rule)
fn just_too_long() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    println!("This is too long but doesn't satisfy the AND condition");
}

// ====== OR Matching Tests ======

// Function that matches the first part of the OR condition
fn alternative_long_function() {  //~ ERROR: Function exceeds maximum length of 1 lines with 8 lines
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    println!("This is too long, triggering one part of OR");
}

// Function that matches the second part of the OR condition
fn short_function_with_suffix() {  //~ ERROR: Function exceeds maximum length of 1 lines with 3 lines
    println!("This is short but triggers the second condition of OR");
}

// ====== NOT Matching Tests ======

// Function that matches the NOT condition in pup.ron
fn normal_function() {
    println!("This is a normal function that's not exempt");
}

// Function exempt from NOT condition - should not get an error because of the NOT in the match
fn exempt_function() {
    // This function is exempt due to having "exempt" in the name
    println!("This should be exempt from the NOT rule");
} 
