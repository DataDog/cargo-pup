// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use super::types::{FunctionMatch, ReturnTypePattern};

/// Fluent interface for creating function matchers
///
/// Used with the `matching()` method to create complex function matching criteria
pub struct FunctionMatcher;

impl FunctionMatcher {
    /// Match a function with exactly this name
    pub fn name(&self, name: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::NameEquals(name.into()))
    }

    /// Match functions whose names match this regex pattern
    pub fn name_regex(&self, pattern: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::NameRegex(pattern.into()))
    }

    /// Matches functions in a specific module, with the module
    /// name given as a regular expression.
    ///
    /// e.g., "^core::(utils|models)::[a-zA-Z]+$"
    ///
    /// Hint: you can use `cargo pup print-modules` to see the modules in your
    /// project and their fully-qualified names.
    ///
    pub fn in_module(&self, module: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::InModule(module.into()))
    }

    /// Matches functions that return a Result<T, E>
    pub fn returns_result(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Result))
    }

    /// Matches functions that return a Result type where the error type implements the Error trait
    pub fn returns_result_with_error_impl(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(
            ReturnTypePattern::ResultWithErrorImpl,
        ))
    }

    /// Matches functions that return an Option<T>
    pub fn returns_option(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Option))
    }

    /// Matches functions that return a specific named type
    pub fn returns_type(&self, name: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Named(
            name.into(),
        )))
    }

    /// Matches functions that return a type matching a regex pattern
    pub fn returns_type_regex(&self, pattern: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Regex(
            pattern.into(),
        )))
    }

    /// Matches functions that return `Self` by value
    pub fn returns_self(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::SelfValue))
    }

    /// Matches functions that return `&Self`
    pub fn returns_self_ref(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::SelfRef))
    }

    /// Matches functions that return `&mut Self`
    pub fn returns_self_mut_ref(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::SelfMutRef))
    }

    /// Matches async functions
    pub fn is_async(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::IsAsync)
    }
}

/// Node in the matcher expression tree
///
/// You can combine these nodes with logical operations (.and(), .or(), .not())
#[derive(Clone)]
pub enum FunctionMatchNode {
    Leaf(FunctionMatch),
    And(Box<FunctionMatchNode>, Box<FunctionMatchNode>),
    Or(Box<FunctionMatchNode>, Box<FunctionMatchNode>),
    Not(Box<FunctionMatchNode>),
}

impl FunctionMatchNode {
    /// Create a logical AND operation between two matchers
    pub fn and(self, other: FunctionMatchNode) -> Self {
        FunctionMatchNode::And(Box::new(self), Box::new(other))
    }

    /// Create a logical OR operation between two matchers
    pub fn or(self, other: FunctionMatchNode) -> Self {
        FunctionMatchNode::Or(Box::new(self), Box::new(other))
    }

    /// Create a logical NOT operation that inverts the matcher
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        FunctionMatchNode::Not(Box::new(self))
    }

    // Converts the DSL tree to the actual FunctionMatch
    pub fn build(self) -> FunctionMatch {
        match self {
            FunctionMatchNode::Leaf(matcher) => matcher,
            FunctionMatchNode::And(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                FunctionMatch::AndMatches(Box::new(a_match), Box::new(b_match))
            }
            FunctionMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                FunctionMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            }
            FunctionMatchNode::Not(m) => {
                let inner = m.build();
                FunctionMatch::NotMatch(Box::new(inner))
            }
        }
    }
}

/// Helper function that converts a matcher DSL expression to a FunctionMatch
///
/// This is used internally by the builder API and typically not called directly
pub fn matcher<F>(f: F) -> FunctionMatch
where
    F: FnOnce(&FunctionMatcher) -> FunctionMatchNode,
{
    let matcher = FunctionMatcher;
    let node = f(&matcher);
    node.build()
}
