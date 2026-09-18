// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use super::types::StructMatch;

// === Struct Matcher DSL === //
pub struct StructMatcher;

impl StructMatcher {
    /// Matches structs by name, given a regular expression.
    /// e.g., "^[A-Z][a-z]+Model$")
    pub fn name(&self, name: impl Into<String>) -> StructMatchNode {
        StructMatchNode::Leaf(StructMatch::Name(name.into()))
    }

    /// Matches structs by an attribute name that remains attached after macro expansion.
    ///
    /// The regular expression must match the complete, compiler-normalised name. For example,
    /// `"repr"` matches `#[repr(C)]`, but attribute arguments such as `C` are not available.
    ///
    /// Supported built-in names are `deprecated`, `doc`, `must_use`, `non_exhaustive`, and `repr`.
    /// Custom and tool attributes can also match if they remain attached after expansion. This
    /// does not work for attributes consumed while expanding code, including `derive` and
    /// attribute procedural macros. The original `cfg_attr` expression is not available, but a
    /// supported attribute produced by an active `cfg_attr` can match by its name.
    ///
    pub fn has_attribute(&self, attr: impl Into<String>) -> StructMatchNode {
        StructMatchNode::Leaf(StructMatch::HasAttribute(attr.into()))
    }

    /// Matches structs that implement a specific trait. The trait name is given as a regular
    /// expression, e.g. `"^(Read|Write)$"`.
    ///
    /// Generic arguments are not included in the name. A generic trait matches when the struct
    /// implements it for any arguments; associated type values are not constrained.
    ///
    pub fn implements_trait(&self, trait_name: impl Into<String>) -> StructMatchNode {
        StructMatchNode::Leaf(StructMatch::ImplementsTrait(trait_name.into()))
    }
}

#[derive(Clone)]
pub enum StructMatchNode {
    Leaf(StructMatch),
    And(Box<StructMatchNode>, Box<StructMatchNode>),
    Or(Box<StructMatchNode>, Box<StructMatchNode>),
    Not(Box<StructMatchNode>),
}

impl StructMatchNode {
    pub fn and(self, other: StructMatchNode) -> Self {
        StructMatchNode::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: StructMatchNode) -> Self {
        StructMatchNode::Or(Box::new(self), Box::new(other))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        StructMatchNode::Not(Box::new(self))
    }

    // Converts the DSL tree to the actual StructMatch
    pub fn build(self) -> StructMatch {
        match self {
            StructMatchNode::Leaf(matcher) => matcher,
            StructMatchNode::And(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                StructMatch::AndMatches(Box::new(a_match), Box::new(b_match))
            }
            StructMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                StructMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            }
            StructMatchNode::Not(m) => {
                let inner = m.build();
                StructMatch::NotMatch(Box::new(inner))
            }
        }
    }
}

// Factory function to create a matcher DSL
pub fn matcher<F>(f: F) -> StructMatch
where
    F: FnOnce(&StructMatcher) -> StructMatchNode,
{
    let matcher = StructMatcher;
    let node = f(&matcher);
    node.build()
}
