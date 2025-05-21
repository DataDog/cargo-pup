pub mod function_lint;
pub mod lint_builder;
pub mod module_lint;
pub mod struct_lint;

pub use lint_builder::LintBuilder;

// Make sure our extensions are visible
pub use function_lint::{
    FunctionLintExt, FunctionMatch, FunctionMatchNode, FunctionMatcher, FunctionRule,
    ReturnTypePattern, matcher as function_matcher,
};
pub use module_lint::{
    ModuleLintExt, ModuleMatch, ModuleMatchNode, ModuleMatcher, ModuleRule,
    matcher as module_matcher,
};
pub use struct_lint::{
    StructLintExt, StructMatch, StructMatchNode, StructMatcher, StructRule,
    matcher as struct_matcher,
};

use crate::function_lint::FunctionLint;
use crate::module_lint::ModuleLint;
use crate::struct_lint::StructLint;
use cargo_pup_common::project_context::ProjectContext;
use serde::{Deserialize, Serialize};

/// Trait for lint types that can generate configurations from multiple ProjectContexts
pub trait GenerateFromContext {
    /// Generate lint configuration based on project contexts and add it to the provided builder
    ///
    /// This accepts a vector of contexts, allowing generation of lints that span
    /// multiple crates or projects. Multiple lint types can contribute to the same
    /// LintBuilder, creating a composable API.
    fn generate_from_contexts(contexts: &[ProjectContext], builder: &mut LintBuilder);
}

/// Severity level for lint rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Severity {
    /// Warning - prints a warning but doesn't cause failure
    #[default]
    Warn,
    /// Error - causes the lint check to fail
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfiguredLint {
    Module(ModuleLint),
    Struct(StructLint),
    Function(FunctionLint),
}
