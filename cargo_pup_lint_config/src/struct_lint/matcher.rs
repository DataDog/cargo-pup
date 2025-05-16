use super::types::StructMatch;

// === Struct Matcher DSL === //
pub struct StructMatcher;

impl StructMatcher {
    /// Matches structs by name
    /// 
    /// The name parameter can be either:
    /// - An exact struct name (e.g., "User")
    /// - A regular expression pattern (e.g., "^[A-Z][a-z]+Model$")
    ///
    /// The implementation will determine if it's a regex based on the presence of special regex characters.
    pub fn name(&self, name: impl Into<String>) -> StructMatchNode {
        StructMatchNode::Leaf(StructMatch::Name(name.into()))
    }
    
    /// Matches structs by attribute
    /// 
    /// The attribute parameter can be either:
    /// - An exact attribute (e.g., "derive(Debug)")
    /// - A regular expression pattern (e.g., "derive\\(.*Debug.*\\)")
    ///
    /// The implementation will determine if it's a regex based on the presence of special regex characters.
    pub fn has_attribute(&self, attr: impl Into<String>) -> StructMatchNode {
        StructMatchNode::Leaf(StructMatch::HasAttribute(attr.into()))
    }
    
    /// Matches structs that implement a specific trait
    ///
    /// The trait_name parameter can be either:
    /// - An exact trait name (e.g., "Debug")
    /// - A trait with path (e.g., "std::fmt::Debug")
    /// - A regular expression pattern (e.g., "^(Read|Write)$")
    ///
    /// The implementation will determine if it's a regex based on the presence of special regex characters.
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
            },
            StructMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                StructMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            },
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
    F: FnOnce(&StructMatcher) -> StructMatchNode 
{
    let matcher = StructMatcher;
    let node = f(&matcher);
    node.build()
}