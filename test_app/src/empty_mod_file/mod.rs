// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

/// This module's mod.rs file should only contain re-exports
/// Any other content should trigger the MustHaveEmptyModFile lint

// This is allowed (re-export)
pub use std::time::Duration;

// This is disallowed (struct definition)
pub struct DisallowedInModRs {
    field: String,
}

// This is disallowed (function definition)
pub fn disallowed_function() -> i32 {
    println!("This function should not be directly in mod.rs");
    42
}
