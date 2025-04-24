use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;

// Constants
const PUP_DIR: &str = ".pup";
const CONTEXT_FILE_SUFFIX: &str = "_context.json";

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

#[allow(dead_code)]
impl ProjectContext {
    /// Creates a new empty project context
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            module_root: String::new(),
            traits: Vec::new(),
        }
    }

    /// Serialize this project context to a file in the .pup directory
    /// with a name based on the module_root
    pub fn serialize_to_file(&self) -> Result<PathBuf> {
        if self.module_root.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot serialize ProjectContext with empty module_root"
            ));
        }

        // Ensure the .pup directory exists
        let pup_dir = PathBuf::from(PUP_DIR);
        fs::create_dir_all(&pup_dir).context(format!("Failed to create directory: {}", PUP_DIR))?;

        // Create a predictable filename using just the crate name
        let filename = format!("{}{}", self.module_root, CONTEXT_FILE_SUFFIX);
        let file_path = pup_dir.join(&filename);

        // Log the exact file we're writing to for debugging
        eprintln!("Writing ProjectContext to: {}", file_path.display());

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .context(format!(
                "Failed to open file for writing: {}",
                file_path.display()
            ))?;

        serde_json::to_writer_pretty(file, &self).context(format!(
            "Failed to serialize ProjectContext to: {}",
            file_path.display()
        ))?;

        Ok(file_path)
    }

    /// Load all project contexts from the .pup directory and return the merged result
    pub fn load_all_contexts() -> Result<ProjectContext> {
        let (context, _) = Self::load_all_contexts_with_crate_names()?;
        Ok(context)
    }

    /// Load all project contexts from the .pup directory and return the merged result
    /// along with a list of all crate names that were found
    pub fn load_all_contexts_with_crate_names() -> Result<(ProjectContext, Vec<String>)> {
        let pup_dir = PathBuf::from(PUP_DIR);
        if !pup_dir.exists() {
            return Err(anyhow::anyhow!("No .pup directory found"));
        }

        // Create aggregated context
        let mut aggregated_context = ProjectContext::new();

        // Track crate names for better presentation
        let mut crate_names = Vec::new();

        // Read all JSON files in .pup directory
        let entries =
            fs::read_dir(&pup_dir).context(format!("Failed to read directory: {}", PUP_DIR))?;

        // Process each file
        let mut contexts_found = false;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                // Look specifically for our deterministic *_context.json pattern
                if filename.ends_with(CONTEXT_FILE_SUFFIX) {
                    let content = fs::read_to_string(&path)
                        .context(format!("Failed to read file: {}", path.display()))?;

                    let context: ProjectContext = serde_json::from_str(&content)
                        .context(format!("Failed to parse JSON from: {}", path.display()))?;

                    // Found a valid context
                    contexts_found = true;

                    // Add crate name to our list
                    if !crate_names.contains(&context.module_root) {
                        crate_names.push(context.module_root.clone());
                    }

                    // Merge this context into our aggregate
                    aggregated_context.merge(&context);
                }
            }
        }

        if !contexts_found {
            return Err(anyhow::anyhow!(
                "No project context files found in {}",
                PUP_DIR
            ));
        }

        // Deduplicate the aggregated context
        aggregated_context.deduplicate();

        Ok((aggregated_context, crate_names))
    }

    /// Clean up all context files from the .pup directory
    pub fn clean_context_files() -> Result<()> {
        let pup_dir = PathBuf::from(PUP_DIR);
        if !pup_dir.exists() {
            return Ok(()); // Nothing to clean if directory doesn't exist
        }

        let entries =
            fs::read_dir(&pup_dir).context(format!("Failed to read directory: {}", PUP_DIR))?;

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.ends_with(CONTEXT_FILE_SUFFIX) {
                    let _ = fs::remove_file(&path); // Ignore errors on deletion
                }
            }
        }

        Ok(())
    }

    // Private implementation methods

    /// Merge another ProjectContext into this one
    fn merge(&mut self, other: &ProjectContext) {
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
    fn deduplicate(&mut self) {
        // Sort and deduplicate modules
        self.modules.sort();
        self.modules.dedup();

        // Create a map for deduplicating traits
        let mut trait_map: HashMap<String, TraitInfo> = HashMap::new();

        // Deduplicate traits by name and merge implementors
        for trait_info in self.traits.drain(..) {
            trait_map
                .entry(trait_info.name.clone())
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
