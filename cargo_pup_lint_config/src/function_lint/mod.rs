// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

/// Function lint module provides tools for creating and configuring function-level lints
/// 
/// These lints can verify properties like:
/// - Function length constraints
/// - Error handling patterns for Result types
/// - Module organization rules
/// 
/// # Example
/// ```
/// use cargo_pup_lint_config::{LintBuilder, Severity};
/// use cargo_pup_lint_config::function_lint::FunctionLintExt;
/// 
/// let mut builder = LintBuilder::new();
/// 
/// // Create a lint that enforces Result error types implement the Error trait
/// builder.function_lint()
///     .lint_named("result_error_impl")
///     .matching(|m| m.returns_result())
///     .with_severity(Severity::Error)
///     .enforce_error_trait_implementation()
///     .build();
/// ```
mod types;
mod matcher;
mod builder;
#[cfg(test)]
mod tests;
mod generate_config;

// Core types for defining function lints
pub use types::{FunctionLint, FunctionMatch, FunctionRule, ReturnTypePattern};

// Function matcher DSL for creating complex matching rules
pub use matcher::{FunctionMatcher, FunctionMatchNode, matcher};

// Builder API for creating function lints
pub use builder::{FunctionLintExt, FunctionLintBuilder, FunctionNamedBuilder, FunctionConstraintBuilder};
