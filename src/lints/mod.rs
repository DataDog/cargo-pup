mod architecture_lint_collection;
mod architecture_lint_rule;
mod architecture_lint_runner;
mod function_length;
mod lint_result;
mod namespace;
mod trait_impl;

// Re-export our public API
pub use architecture_lint_collection::ArchitectureLintCollection;
pub use architecture_lint_collection::register_all_lints;
pub use architecture_lint_rule::ArchitectureLintRule;
pub use architecture_lint_runner::ArchitectureLintRunner;
pub use architecture_lint_runner::Mode;
pub use lint_result::LintResult;
pub use lint_result::Severity;
