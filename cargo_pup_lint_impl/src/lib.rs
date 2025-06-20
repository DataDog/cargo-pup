#![feature(rustc_private)]
// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.
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
extern crate rustc_type_ir;

mod architecture_lint_rule;
pub mod lints;

// Re-export our public API
pub use architecture_lint_rule::ArchitectureLintRule;
pub use helpers::architecture_lint_collection::ArchitectureLintCollection;
pub use helpers::architecture_lint_runner::ArchitectureLintRunner;
pub use helpers::architecture_lint_runner::Mode;
