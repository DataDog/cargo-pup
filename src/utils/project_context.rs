use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

pub const PUP_DIR: &str = ".pup";
pub const CONTEXT_FILE_SUFFIX: &str = "_context.json";

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
    /// Base directory for storing context files (not serialized)
    #[serde(skip)]
    base_dir: PathBuf,
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
    /// Creates a new empty project context with default base directory (.pup)
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            module_root: String::new(),
            traits: Vec::new(),
            base_dir: PathBuf::from(PUP_DIR),
        }
    }
    
    /// Creates a new empty project context with a custom base directory
    pub fn with_base_dir(dir_path: impl AsRef<Path>) -> Self {
        Self {
            modules: Vec::new(),
            module_root: String::new(),
            traits: Vec::new(),
            base_dir: dir_path.as_ref().to_path_buf(),
        }
    }
    
    /// Creates a project context with provided data and default base directory (.pup)
    /// This helps migrate code that previously used struct initialization
    pub fn with_data(
        modules: Vec<String>, 
        module_root: String, 
        traits: Vec<TraitInfo>
    ) -> Self {
        Self {
            modules,
            module_root,
            traits,
            base_dir: PathBuf::from(PUP_DIR),
        }
    }
    
    /// Creates a project context with provided data and a custom base directory
    pub fn with_data_and_base_dir(
        modules: Vec<String>, 
        module_root: String, 
        traits: Vec<TraitInfo>,
        dir_path: impl AsRef<Path>
    ) -> Self {
        Self {
            modules,
            module_root,
            traits,
            base_dir: dir_path.as_ref().to_path_buf(),
        }
    }

    /// Serialize this project context to a file in the base directory
    /// with a name based on the module_root
    pub fn serialize_to_file(&self) -> Result<PathBuf> {
        if self.module_root.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot serialize ProjectContext with empty module_root"
            ));
        }

        // Ensure the base directory exists
        fs::create_dir_all(&self.base_dir)
            .context(format!("Failed to create directory: {}", self.base_dir.display()))?;

        // Create a predictable filename using just the crate name
        let filename = format!("{}{}", self.module_root, CONTEXT_FILE_SUFFIX);
        let file_path = self.base_dir.join(&filename);

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

    /// Load all project contexts from the default .pup directory and return the merged result
    pub fn load_all_contexts() -> Result<ProjectContext> {
        let (context, _) = Self::load_all_contexts_with_crate_names()?;
        Ok(context)
    }

    /// Load all project contexts from the default .pup directory and return the merged result
    /// along with a list of all crate names that were found
    pub fn load_all_contexts_with_crate_names() -> Result<(ProjectContext, Vec<String>)> {
        Self::load_all_contexts_from_dir(&PathBuf::from(PUP_DIR))
    }

    /// Load all project contexts from a specific directory and return the merged result
    /// along with a list of all crate names that were found
    pub fn load_all_contexts_from_dir(dir_path: &Path) -> Result<(ProjectContext, Vec<String>)> {
        if !dir_path.exists() {
            return Err(anyhow::anyhow!("Directory not found: {}", dir_path.display()));
        }

        // Create aggregated context with the specified base directory
        let mut aggregated_context = ProjectContext::with_base_dir(dir_path);

        // Track crate names for better presentation
        let mut crate_names = Vec::new();

        // Read all JSON files in the directory
        let entries = fs::read_dir(dir_path)
            .context(format!("Failed to read directory: {}", dir_path.display()))?;

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
                dir_path.display()
            ));
        }

        // Deduplicate the aggregated context
        aggregated_context.deduplicate();

        Ok((aggregated_context, crate_names))
    }

    /// Clean up all context files from the base directory
    pub fn clean_context_files(&self) -> Result<()> {
        if !self.base_dir.exists() {
            return Ok(()); // Nothing to clean if directory doesn't exist
        }

        let entries = fs::read_dir(&self.base_dir)
            .context(format!("Failed to read directory: {}", self.base_dir.display()))?;

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

    /// Clean up all context files from the default .pup directory
    pub fn clean_default_context_files() -> Result<()> {
        let default_context = ProjectContext::new();
        default_context.clean_context_files()
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

        // Add traits (since each trait has a unique fully-qualified name,
        // we can just add them without worrying about duplicates)
        self.traits.extend(other.traits.clone());
    }

    /// Deduplicates modules and sorts traits for consistent ordering
    fn deduplicate(&mut self) {
        // Sort and deduplicate modules
        self.modules.sort();
        self.modules.dedup();

        // Sort traits by name for consistent ordering
        self.traits.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// Format and print the modules in the project context
pub fn print_modules(context: &ProjectContext, crate_names: &[String]) -> Result<()> {
    use ansi_term::Colour::{Blue, Green, Red, Yellow, Cyan};
    use ansi_term::Style;
    use std::collections::BTreeMap;
    
    // Print a header
    println!("{}", Cyan.paint(r#"
     / \__
    (    @\___
    /         O
   /   (_____/
  /_____/   U
"#));
    
    if crate_names.len() > 1 {
        println!("Modules from multiple crates: {}", crate_names.join(", "));
    } else {
        println!("Modules from crate: {}", context.module_root);
    }
    println!();
    
    // Print modules with applicable lints
    let mut modules_by_crate: BTreeMap<String, Vec<String>> = BTreeMap::new();
    
    // Group modules by crate
    for module_path in &context.modules {
        // Extract crate name from module path (everything before the first ::)
        if let Some(idx) = module_path.find("::") {
            let crate_name = &module_path[..idx];
            let module_suffix = &module_path[idx..];
            
            modules_by_crate.entry(crate_name.to_string())
                .or_insert_with(Vec::new)
                .push(module_suffix.to_string());
        } else {
            // Handle case where there's no :: in the path
            modules_by_crate.entry(module_path.clone())
                .or_insert_with(Vec::new);
        }
    }
    
    // Print modules organized by crate
    for (crate_name, modules) in modules_by_crate {
        println!("{}", Blue.paint(&crate_name));
        
        for module_suffix in modules {
            println!("  {}", module_suffix);
        }
        println!();
    }
    
    Ok(())
}

/// Format and print the traits in the project context
pub fn print_traits(context: &ProjectContext, crate_names: &[String]) -> Result<()> {
    use ansi_term::Colour::{Blue, Green, Red, Yellow, Cyan};
    use ansi_term::Style;
    use std::collections::BTreeMap;
    
    // Print a header
    println!("{}", Cyan.paint(r#"
     / \__
    (    @\___
    /         O
   /   (_____/
  /_____/   U
"#));
    
    if crate_names.len() > 1 {
        println!("Traits from multiple crates: {}", crate_names.join(", "));
    } else {
        println!("Traits from crate: {}", context.module_root);
    }
    println!();
    
    // Print traits with their implementations
    let mut traits_by_crate: BTreeMap<String, Vec<(&String, &Vec<String>)>> = BTreeMap::new();
    
    // Group traits by crate
    for trait_info in &context.traits {
        // Extract crate name from trait path (everything before the first ::)
        if let Some(idx) = trait_info.name.find("::") {
            let crate_name = &trait_info.name[..idx];
            
            traits_by_crate.entry(crate_name.to_string())
                .or_insert_with(Vec::new)
                .push((&trait_info.name, &trait_info.implementors));
        } else {
            // Handle case where there's no :: in the path
            traits_by_crate.entry(trait_info.name.clone())
                .or_insert_with(Vec::new);
        }
    }
    
    // Print traits organized by crate
    for (crate_name, traits) in traits_by_crate {
        println!("{}", Blue.paint(&crate_name));
        
        for (trait_name, implementors) in traits {
            // Extract the part after the crate name
            let trait_suffix = if let Some(idx) = trait_name.find("::") {
                &trait_name[idx..]
            } else {
                trait_name
            };
            
            println!("  {}", trait_suffix);
            
            // Print implementors with indentation
            if !implementors.is_empty() {
                for implementor in implementors {
                    println!("    → {}", Green.paint(implementor));
                }
            }
        }
        println!();
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_empty_context() {
        let context = ProjectContext::new();
        assert!(context.modules.is_empty());
        assert!(context.module_root.is_empty());
        assert!(context.traits.is_empty());
    }

    #[test]
    fn test_serialization_and_deserialization() {
        // Create a test context
        let mut context = ProjectContext::new();
        context.module_root = "test_crate".to_string();
        context.modules = vec![
            "test_crate::module1".to_string(),
            "test_crate::module2".to_string(),
        ];

        context.traits = vec![TraitInfo {
            name: "test_crate::Trait1".to_string(),
            implementors: vec![
                "test_crate::Type1".to_string(),
                "test_crate::Type2".to_string(),
            ],
        }];

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&context).expect("Serialization failed");

        // Deserialize back to ProjectContext
        let deserialized: ProjectContext =
            serde_json::from_str(&json).expect("Deserialization failed");

        // Verify the deserialized context matches the original
        assert_eq!(deserialized.module_root, "test_crate");
        assert_eq!(deserialized.modules.len(), 2);
        assert_eq!(deserialized.traits.len(), 1);
        assert_eq!(deserialized.traits[0].name, "test_crate::Trait1");
        assert_eq!(deserialized.traits[0].implementors.len(), 2);
    }

    #[test]
    fn test_serialize_empty_module_root_error() {
        // Create a context with empty module_root
        let mut context = ProjectContext::new();
        context.modules = vec!["test::module".to_string()];

        // This doesn't actually try to write to a file, just checks the validation logic
        let result = context.serialize_to_file();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("empty module_root")
        );
    }

    // We no longer need these custom test implementations since we've incorporated
    // the functionality into the main ProjectContext implementation

    #[test]
    fn roundtrip_through_files() {
        use tempfile::TempDir;
        
        // Create a test-specific temp directory
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let test_dir_path = temp_dir.path();
        
        // Create first context with custom base directory
        let mut context1 = ProjectContext::with_base_dir(test_dir_path);
        context1.module_root = "crate1".to_string();
        context1.modules = vec![
            "crate1::module1".to_string(),
            "crate1::module2".to_string(),
        ];
        context1.traits = vec![
            TraitInfo {
                name: "crate1::Trait1".to_string(),
                implementors: vec!["crate1::Type1".to_string()],
            }
        ];

        // Create second context with same custom base directory
        let mut context2 = ProjectContext::with_base_dir(test_dir_path);
        context2.module_root = "crate2".to_string(); 
        context2.modules = vec![
            "crate2::moduleA".to_string(),
            "crate2::moduleB".to_string(),
        ];
        context2.traits = vec![
            TraitInfo {
                name: "crate2::TraitX".to_string(),
                implementors: vec!["crate2::TypeX".to_string()],
            }
        ];
        
        // Serialize both contexts to the temp directory
        let file1 = context1.serialize_to_file().expect("Failed to serialize context1");
        let file2 = context2.serialize_to_file().expect("Failed to serialize context2");

        // Verify files exist
        assert!(file1.exists(), "Context file 1 should exist");
        assert!(file2.exists(), "Context file 2 should exist");

        // Load all contexts back from our test directory
        let (loaded_context, crate_names) = ProjectContext::load_all_contexts_from_dir(test_dir_path)
            .expect("Failed to load contexts");

        // Validate the loaded context
        // Should have a valid module root
        assert!(!loaded_context.module_root.is_empty(), "Module root should not be empty");
        
        // Should contain all modules from both contexts
        assert_eq!(loaded_context.modules.len(), 4, "Should have all 4 modules");
        assert!(loaded_context.modules.contains(&"crate1::module1".to_string()));
        assert!(loaded_context.modules.contains(&"crate1::module2".to_string()));
        assert!(loaded_context.modules.contains(&"crate2::moduleA".to_string()));
        assert!(loaded_context.modules.contains(&"crate2::moduleB".to_string()));
        
        // Should have both traits
        assert_eq!(loaded_context.traits.len(), 2, "Should have both traits");
        
        // Verify first trait exists
        let trait1 = loaded_context.traits.iter()
            .find(|t| t.name == "crate1::Trait1")
            .expect("Should find first trait");
        assert_eq!(trait1.implementors.len(), 1);
        assert_eq!(trait1.implementors[0], "crate1::Type1");
        
        // Verify second trait exists
        let trait2 = loaded_context.traits.iter()
            .find(|t| t.name == "crate2::TraitX")
            .expect("Should find second trait");
        assert_eq!(trait2.implementors.len(), 1);
        assert_eq!(trait2.implementors[0], "crate2::TypeX");
        
        // Verify both crate names were detected
        assert_eq!(crate_names.len(), 2, "Should have found 2 crate names");
        assert!(crate_names.contains(&"crate1".to_string()));
        assert!(crate_names.contains(&"crate2".to_string()));
        
        // temp_dir will be automatically cleaned up when it goes out of scope
    }
}
