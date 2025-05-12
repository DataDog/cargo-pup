pub mod lint_builder;
mod module_lint;
mod struct_lint;

// Make sure our extenions are visible
pub use module_lint::{ModuleMatch, ModuleLintExt, ModuleRule};
pub use struct_lint::{StructMatch, StructLintExt, StructRule};

use serde::{Deserialize, Serialize};
use crate::module_lint::ModuleLint;
use crate::struct_lint::StructLint;

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfiguredLint {
    Module(ModuleLint),
    Struct(StructLint),
}
