use super::types::{FunctionMatch, ReturnTypePattern};

// === Function Matcher DSL === //
pub struct FunctionMatcher;

impl FunctionMatcher {
    pub fn name(&self, name: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::NameEquals(name.into()))
    }
    
    pub fn name_regex(&self, pattern: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::NameRegex(pattern.into()))
    }
    
    /// Matches functions in a specific module
    /// 
    /// The module parameter can be either:
    /// - An exact module path (e.g., "core::utils")
    /// - A regular expression pattern (e.g., "^core::(utils|models)::[a-zA-Z]+$")
    ///
    /// The implementation will determine if it's a regex based on the presence of special regex characters.
    pub fn in_module(&self, module: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::InModule(module.into()))
    }
    
    /// Matches functions that return a Result<T, E>
    pub fn returns_result(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Result))
    }
    
    /// Matches functions that return a Result type where the error type implements the Error trait
    pub fn returns_result_with_error_impl(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::ResultWithErrorImpl))
    }
    
    /// Matches functions that return an Option<T>
    pub fn returns_option(&self) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Option))
    }
    
    /// Matches functions that return a specific named type
    pub fn returns_type(&self, name: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Named(name.into())))
    }
    
    /// Matches functions that return a type matching a regex pattern
    pub fn returns_type_regex(&self, pattern: impl Into<String>) -> FunctionMatchNode {
        FunctionMatchNode::Leaf(FunctionMatch::ReturnsType(ReturnTypePattern::Regex(pattern.into())))
    }
}

#[derive(Clone)]
pub enum FunctionMatchNode {
    Leaf(FunctionMatch),
    And(Box<FunctionMatchNode>, Box<FunctionMatchNode>),
    Or(Box<FunctionMatchNode>, Box<FunctionMatchNode>),
    Not(Box<FunctionMatchNode>),
}

impl FunctionMatchNode {
    pub fn and(self, other: FunctionMatchNode) -> Self {
        FunctionMatchNode::And(Box::new(self), Box::new(other))
    }
    
    pub fn or(self, other: FunctionMatchNode) -> Self {
        FunctionMatchNode::Or(Box::new(self), Box::new(other))
    }
    
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
            },
            FunctionMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                FunctionMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            },
            FunctionMatchNode::Not(m) => {
                let inner = m.build();
                FunctionMatch::NotMatch(Box::new(inner))
            }
        }
    }
}

// Factory function to create a matcher DSL
pub fn matcher<F>(f: F) -> FunctionMatch 
where 
    F: FnOnce(&FunctionMatcher) -> FunctionMatchNode 
{
    let matcher = FunctionMatcher;
    let node = f(&matcher);
    node.build()
}