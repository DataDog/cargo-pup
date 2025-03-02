#![allow(dead_code)]

mod declare_lint;
pub mod clippy_utils;
pub mod architecture_lint_collection;
pub mod architecture_lint_runner;
mod queries;

pub use queries::get_full_module_name;
