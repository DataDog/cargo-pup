/// Module lint module provides tools for creating and configuring module-level lints
/// 
/// These lints can enforce architectural constraints like:
/// - Module organization and naming
/// - Import/export restrictions
/// - Content restrictions (empty vs. non-empty)
/// - Wildcard import prevention
/// 
/// # Example
/// ```
/// use cargo_pup_lint_config::{LintBuilder, Severity};
/// use cargo_pup_lint_config::module_lint::ModuleLintExt;
/// 
/// let mut builder = LintBuilder::new("my_crate");
/// 
/// // Create a lint that enforces handler modules must be empty (re-export only)
/// builder.module_lint()
///     .lint_named("empty_handlers")
///     .matching(|m| m.module("handlers"))
///     .with_severity(Severity::Warning)
///     .must_be_empty()
///     .build();
/// 
/// // Create a lint that prevents wildcard imports
/// builder.module_lint()
///     .lint_named("no_glob_imports")
///     .matching(|m| m.module(".*")) // match all modules
///     .with_severity(Severity::Error)
///     .no_wildcard_imports()
///     .build();
/// ```
mod types;
mod matcher;
mod builder;
mod generate_config;

// Core types for defining module lints
pub use types::{ModuleLint, ModuleMatch, ModuleRule};

// Module matcher DSL for creating complex matching rules
pub use matcher::{ModuleMatcher, ModuleMatchNode, matcher};

// Builder API for creating module lints
pub use builder::{ModuleLintExt, ModuleLintBuilder, ModuleNamedBuilder, ModuleConstraintBuilder};