use super::types::ModuleMatch;

// === Module Matcher DSL === //
pub struct ModuleMatcher;

impl ModuleMatcher {
    pub fn module(&self, module: impl Into<String>) -> ModuleMatchNode {
        ModuleMatchNode::Leaf(ModuleMatch::Module(module.into()))
    }
}

#[derive(Clone)]
pub enum ModuleMatchNode {
    Leaf(ModuleMatch),
    And(Box<ModuleMatchNode>, Box<ModuleMatchNode>),
    Or(Box<ModuleMatchNode>, Box<ModuleMatchNode>),
    Not(Box<ModuleMatchNode>),
}

impl ModuleMatchNode {
    pub fn and(self, other: ModuleMatchNode) -> Self {
        ModuleMatchNode::And(Box::new(self), Box::new(other))
    }
    
    pub fn or(self, other: ModuleMatchNode) -> Self {
        ModuleMatchNode::Or(Box::new(self), Box::new(other))
    }
    
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
            },
            ModuleMatchNode::Or(a, b) => {
                let a_match = a.build();
                let b_match = b.build();
                ModuleMatch::OrMatches(Box::new(a_match), Box::new(b_match))
            },
            ModuleMatchNode::Not(m) => {
                let inner = m.build();
                ModuleMatch::NotMatch(Box::new(inner))
            }
        }
    }
}

// Factory function to create a matcher DSL
pub fn matcher<F>(f: F) -> ModuleMatch 
where 
    F: FnOnce(&ModuleMatcher) -> ModuleMatchNode 
{
    let matcher = ModuleMatcher;
    let node = f(&matcher);
    node.build()
}