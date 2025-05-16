#![feature(rustc_private)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(try_blocks)]

pub mod helpers;

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;

mod architecture_lint_rule;
mod configuration_factory;
pub mod lints;

// Re-export our public API
pub use architecture_lint_rule::ArchitectureLintRule;
pub use helpers::architecture_lint_collection::ArchitectureLintCollection;
pub use helpers::architecture_lint_collection::register_all_lints;
pub use helpers::architecture_lint_runner::ArchitectureLintRunner;
pub use helpers::architecture_lint_runner::Mode;
pub use configuration_factory::LintConfigurationFactory;
pub use configuration_factory::LintFactory;
pub use configuration_factory::setup_lints_yaml;
