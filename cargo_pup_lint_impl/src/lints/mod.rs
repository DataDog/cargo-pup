use cargo_pup_lint_config::ConfiguredLint;
use crate::ArchitectureLintRule;

use rustc_lint::{LateContext, LateLintPass, Lint, LintStore};

pub mod configuration_factory;
mod module_lint;
mod struct_lint;
