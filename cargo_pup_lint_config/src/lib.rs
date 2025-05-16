pub mod lint_builder;
pub mod function_lint;
pub mod module_lint;
pub mod struct_lint;

pub use lint_builder::LintBuilder;

// Make sure our extensions are visible
pub use module_lint::{ModuleMatch, ModuleLintExt, ModuleRule, ModuleMatcher, ModuleMatchNode, matcher as module_matcher};
pub use struct_lint::{StructMatch, StructLintExt, StructRule, StructMatcher, StructMatchNode, matcher as struct_matcher};
pub use function_lint::{FunctionMatch, FunctionLintExt, FunctionRule, FunctionMatcher, FunctionMatchNode, ReturnTypePattern, matcher as function_matcher};

use cargo_pup_common::project_context::ProjectContext;
use serde::{Deserialize, Serialize};
use crate::module_lint::ModuleLint;
use crate::struct_lint::StructLint;
use crate::function_lint::FunctionLint;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Warning - prints a warning but doesn't cause failure
    Warn,
    /// Error - causes the lint check to fail
    Error,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Warn
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfiguredLint {
    Module(ModuleLint),
    Struct(StructLint),
    Function(FunctionLint),
}