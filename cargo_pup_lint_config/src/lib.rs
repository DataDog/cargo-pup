pub mod lint_builder;
mod module_lint;
mod struct_lint;

// Make sure our extenions are visible
pub use module_lint::{ModuleMatch, ModuleLintExt, ModuleRule, ModuleMatcher, ModuleMatchNode, matcher as module_matcher};
pub use struct_lint::{StructMatch, StructLintExt, StructRule, StructMatcher, StructMatchNode, matcher as struct_matcher};

use serde::{Deserialize, Serialize};
use crate::module_lint::ModuleLint;
use crate::struct_lint::StructLint;

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfiguredLint {
    Module(ModuleLint),
    Struct(StructLint),
}
