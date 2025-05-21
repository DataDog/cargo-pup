
use crate::{
    ArchitectureLintRule,
};

///
/// Collects a set of architecture lints configured
/// and ready to run.
///
///
///
pub struct ArchitectureLintCollection {
    lints: Vec<Box<dyn ArchitectureLintRule + Send>>,
}

impl ArchitectureLintCollection {
    pub fn new(lints: Vec<Box<dyn ArchitectureLintRule + Send>>) -> ArchitectureLintCollection {
        ArchitectureLintCollection { lints }
    }

    pub fn lints(&self) -> &Vec<Box<dyn ArchitectureLintRule + Send>> {
        &self.lints
    }
}
