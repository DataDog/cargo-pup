// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//! This module demonstrates async function detection with cargo-pup

use std::future::Future;

/// This async function should trigger the IsAsync lint rule
pub async fn forbidden_async_function() -> String {
    "This is an async function".to_string()
}

/// Another async function that returns a Result
pub async fn async_result_function() -> Result<i32, String> {
    Ok(42)
}

/// Regular sync function that should NOT trigger the IsAsync rule
pub fn allowed_sync_function() -> String {
    "This is a sync function".to_string()
}

/// A struct with async methods
pub struct AsyncService {
    name: String,
}

impl AsyncService {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Async method that should trigger the IsAsync rule
    pub async fn process_async(&self) -> Result<String, String> {
        Ok(format!("Processing {}", self.name))
    }

    /// Sync method that should NOT trigger the IsAsync rule
    pub fn process_sync(&self) -> String {
        format!("Sync processing {}", self.name)
    }
}

/// Trait with async methods
pub trait AsyncProcessor {
    /// Async trait method
    async fn process_item(&self, item: String) -> Result<String, String>;
    
    /// Sync trait method
    fn validate_item(&self, item: &str) -> bool;
}

/// Implementation with async methods
impl AsyncProcessor for AsyncService {
    /// This async implementation should trigger the IsAsync rule
    async fn process_item(&self, item: String) -> Result<String, String> {
        Ok(format!("{}: {}", self.name, item))
    }
    
    fn validate_item(&self, item: &str) -> bool {
        !item.is_empty()
    }
}