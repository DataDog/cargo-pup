use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::hash::Hasher;
use std::path::{Path, PathBuf};

/// Context for configuration generation containing compile-time discoverable
/// context about the project we're running cargo-pup on. 
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectContext {
    /// List of all modules, fully qualified
    pub modules: Vec<String>,
    /// The top-level crate name (root module)
    pub module_root: String,
    /// List of all traits, fully qualified, and their implementations
    pub traits: Vec<TraitInfo>,
}

/// Information about a trait and its implementations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraitInfo {
    /// Fully qualified trait name
    pub name: String,
    /// List of types implementing this trait
    pub implementors: Vec<String>,
}

impl ProjectContext {
    /// Creates a new empty project context
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            module_root: String::new(),
            traits: Vec::new(),
        }
    }

    /// Serialize this project context to a file in the specified directory
    /// 
    /// Args:
    ///   dir_path: Directory path where file should be written
    ///   source_file: Optional source file (no longer used for filename generation)
    pub fn serialize_to_file(&self, dir_path: &str, _source_file: Option<&Path>) -> Result<()> {
        // Resolve directory path
        let dir_path = if dir_path == "." {
            std::env::current_dir()?
        } else {
            PathBuf::from(dir_path)
        };
        
        // Ensure the directory exists
        fs::create_dir_all(&dir_path)?;
        
        // Create a predictable filename using just the crate name
        // This must match what we register with rustc in architecture_lint_runner.rs
        let crate_name = &self.module_root;
        let filename = format!("{}_context.json", crate_name);
        
        let file_path = dir_path.join(filename);
        
        // Log the exact file we're writing to for debugging
        eprintln!("Writing ProjectContext to: {}", file_path.display());
        
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)?;
            
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }

    /// Merge another ProjectContext into this one
    pub fn merge(&mut self, other: &ProjectContext) {
        // Add the module root if ours is empty
        if self.module_root.is_empty() {
            self.module_root = other.module_root.clone();
        }
        
        // Add modules
        self.modules.extend(other.modules.clone());
        
        // Add traits
        self.traits.extend(other.traits.clone());
    }
    
    /// Deduplicates modules and traits after merging
    pub fn deduplicate(&mut self) {
        // Sort and deduplicate modules
        self.modules.sort();
        self.modules.dedup();
        
        // Create a map for deduplicating traits
        let mut trait_map: HashMap<String, TraitInfo> = HashMap::new();
        
        // Deduplicate traits by name and merge implementors
        for trait_info in self.traits.drain(..) {
            trait_map.entry(trait_info.name.clone())
                .and_modify(|existing| {
                    // Merge implementors
                    for implementor in &trait_info.implementors {
                        if !existing.implementors.contains(implementor) {
                            existing.implementors.push(implementor.clone());
                        }
                    }
                })
                .or_insert(trait_info);
        }
        
        // Convert back to vector
        self.traits = trait_map.into_values().collect();
        
        // Sort traits by name
        self.traits.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// Functions for reading and aggregating project contexts from multiple files
pub struct ProjectContextManager;

impl ProjectContextManager {
    /// Find and aggregate all project contexts in the .pup directory
    pub fn aggregate_contexts() -> Result<(ProjectContext, Vec<String>)> {
        let pup_dir = PathBuf::from(".pup");
        if !pup_dir.exists() {
            return Err(anyhow::anyhow!("No .pup directory found"));
        }
        
        // Create aggregated context
        let mut aggregated_context = ProjectContext::new();
        
        // Track crate names for better presentation
        let mut crate_names = Vec::new();
        
        // Read all JSON files in .pup directory
        let entries = fs::read_dir(&pup_dir)?;
        
        // Process each file
        let mut contexts_found = false;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                // Look specifically for our deterministic *_context.json pattern
                if filename.ends_with("_context.json") {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            match serde_json::from_str::<ProjectContext>(&content) {
                                Ok(context) => {
                                    // Found a valid context
                                    contexts_found = true;
                                    
                                    // Add crate name to our list
                                    if !crate_names.contains(&context.module_root) {
                                        crate_names.push(context.module_root.clone());
                                    }
                                    
                                    // Merge this context into our aggregate
                                    aggregated_context.merge(&context);
                                },
                                Err(e) => eprintln!("Error parsing context file {}: {}", path.display(), e),
                            }
                        },
                        Err(e) => eprintln!("Error reading context file {}: {}", path.display(), e),
                    }
                }
            }
        }
        
        if !contexts_found {
            return Err(anyhow::anyhow!("No project context files found"));
        }
        
        // Deduplicate the aggregated context
        aggregated_context.deduplicate();
        
        Ok((aggregated_context, crate_names))
    }
    
    /// Clean up context files after aggregation
    pub fn clean_context_files() -> Result<()> {
        let pup_dir = PathBuf::from(".pup");
        let entries = fs::read_dir(&pup_dir)?;
        
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.ends_with("_context.json") {
                    let _ = fs::remove_file(path); // Ignore errors
                }
            }
        }
        
        Ok(())
    }
}