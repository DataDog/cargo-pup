mod types;
mod matcher;
mod builder;
#[cfg(test)]
mod tests;
mod generate_config;

pub use types::{FunctionLint, FunctionMatch, FunctionRule, ReturnTypePattern};
pub use matcher::{FunctionMatcher, FunctionMatchNode, matcher};
pub use builder::{FunctionLintExt, FunctionLintBuilder, FunctionNamedBuilder, FunctionConstraintBuilder};