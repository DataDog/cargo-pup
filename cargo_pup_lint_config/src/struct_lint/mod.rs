/// Struct lint module provides tools for creating and configuring struct-level lints
/// 
/// These lints can enforce architectural constraints like:
/// - Struct naming conventions
/// - Visibility rules (public vs. private)
/// - Required trait implementations
/// - Attribute requirements
/// 
/// # Example
/// ```
/// use cargo_pup_lint_config::{LintBuilder, Severity, StructRule};
/// use cargo_pup_lint_config::struct_lint::StructLintExt;
///
/// let mut builder = LintBuilder::new();
///
/// // Enforce that model structs must be private
/// builder.struct_lint()
///     .lint_named("model_visibility")
///     .matching(|m| m.name(".*Model"))
///     .with_severity(Severity::Error)
///     .must_be_private()
///     .build();
///     
/// // Require error structs to implement the Error trait
/// builder.struct_lint()
///     .lint_named("error_trait_impl")
///     .matching(|m| m.name(".*Error"))
///     .with_severity(Severity::Error)
///     .add_rule(StructRule::ImplementsTrait("std::error::Error".into(), Severity::Error))
///     .build();
/// ```
mod types;
mod matcher;
mod builder;
mod tests;
mod generate_config;

// Core types for defining struct lints
pub use types::{StructLint, StructMatch, StructRule};

// Struct matcher DSL for creating complex matching rules
pub use matcher::{StructMatcher, StructMatchNode, matcher};

// Builder API for creating struct lints
pub use builder::{StructLintExt, StructLintBuilder, StructNamedBuilder, StructConstraintBuilder};