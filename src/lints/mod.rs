mod architecture_lint_collection;
mod architecture_lint_rule;
mod function_length;
mod lint_result;
mod namespace;
mod trait_impl;

// Re-export our public API
pub use architecture_lint_collection::register_all_lints;
pub use architecture_lint_collection::ArchitectureLintCollection;
pub use architecture_lint_rule::ArchitectureLintRule;
pub use lint_result::LintResult;
pub use lint_result::Severity;
