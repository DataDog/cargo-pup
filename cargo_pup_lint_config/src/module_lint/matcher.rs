use super::types::ModuleMatch;

/// Fluent interface for creating module matchers
/// 
/// Used with the `matching()` method to create module matching criteria
pub struct ModuleMatcher;

impl ModuleMatcher {
    /// Match a module by name or path
    /// 
    /// The module parameter can be either:
    /// - An exact module path (e.g., "crate::api::v1")
    /// - A regular expression pattern (e.g., "^crate::api::.*")
    pub fn module(&self, module: impl Into<String>) -> ModuleMatchNode {
        ModuleMatchNode::Leaf(ModuleMatch::Module(module.into()))
    }
}

/// Node in the matcher expression tree
/// 
/// You can combine these nodes with logical operations (.and(), .or(), .not())
#[derive(Clone)]
pub enum ModuleMatchNode {
    Leaf(ModuleMatch),
    And(Box<ModuleMatchNode>, Box<ModuleMatchNode>),
    Or(Box<ModuleMatchNode>, Box<ModuleMatchNode>),
    Not(Box<ModuleMatchNode>),
}

impl ModuleMatchNode {
    /// Create a logical AND operation between two matchers
    pub fn and(self, other: ModuleMatchNode) -> Self {
        ModuleMatchNode::And(Box::new(self), Box::new(other))
    }

    /// Create a logical OR operation between two matchers
    pub fn or(self, other: ModuleMatchNode) -> Self {
        ModuleMatchNode::Or(Box::new(self), Box::new(other))
    }

    /// Create a logical NOT operation that inverts the matcher
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        ModuleMatchNode::Not(Box::new(self))
    }

    // Converts the DSL tree to the actual ModuleMatch
    pub fn build(self) -> ModuleMatch {
        match self {
            ModuleMatchNode::Leaf(matcher) => matcher,
            ModuleMatchNode::And(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                ModuleMatch::AndMatches(Box::new(a_match), Box::new(b_match))
            }
            ModuleMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                ModuleMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            }
            ModuleMatchNode::Not(m) => {
                let inner = m.build();
                ModuleMatch::NotMatch(Box::new(inner))
            }
        }
    }
}

/// Helper function that converts a matcher DSL expression to a ModuleMatch
/// 
/// This is used internally by the builder API and typically not called directly
pub fn matcher<F>(f: F) -> ModuleMatch
where
    F: FnOnce(&ModuleMatcher) -> ModuleMatchNode,
{
    let matcher = ModuleMatcher;
    let node = f(&matcher);
    node.build()
}
