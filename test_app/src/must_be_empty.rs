// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

// This module should be empty according to the lint configuration
// Having content here should trigger a lint warning/error

pub fn this_should_not_be_here() {
    println!("This function violates the MustBeEmpty lint rule");
}

pub const ALSO_NOT_ALLOWED: &str = "This const also violates the MustBeEmpty rule"; 
