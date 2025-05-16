mod types;
mod matcher;
mod builder;
mod tests;

pub use types::{StructLint, StructMatch, StructRule};
pub use matcher::{StructMatcher, StructMatchNode, matcher};
pub use builder::{StructLintExt, StructLintBuilder, StructNamedBuilder, StructConstraintBuilder};