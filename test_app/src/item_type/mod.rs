// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

// These items should trigger warnings
pub struct DeniedStruct {
    field: String,
}

pub enum DeniedEnum {
    Variant1,
    Variant2,
}

pub trait DeniedTrait {
    fn some_method(&self);
}

// These items should be allowed
pub fn allowed_function() {
    println!("This function is allowed!");
}

pub const ALLOWED_CONST: i32 = 42;

// Add a nested module that should be blocked
pub mod nested {
    // This module declaration itself should trigger a warning
    // since we've configured modules to be denied
} 
