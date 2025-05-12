use std::{
    collections::HashMap,
    fs,
    io::Write,
    sync::{Arc, LazyLock, Mutex},
};

use crate::ArchitectureLintRule;
use cargo_pup_common::project_context::ProjectContext;
use anyhow::{Result, anyhow};

pub trait LintFactory: Send + Sync {
    ///
    /// Registers itself with the ConfigurationFactory
    ///
    fn register()
    where
        Self: Sized;

    ////
    /// Create a list of `ArchitectureLintRule`s from a YAML configuration
    /// These let us do non-lint checks - e.g., for printing our namespace
    /// or trait trees. These are not run as part of the rustc lint process
    /// itself.
    ///
    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>>;

    ///
    /// Generate a sample configuration for this lint factory based on the provided context
    /// Returns a map of (rule_name, yaml_string) pairs with commented YAML
    ///
    fn generate_config(&self, _context: &ProjectContext) -> Result<HashMap<String, String>> {
        // Default implementation returns empty
        Ok(HashMap::new())
    }
}

pub struct LintConfigurationFactory {
    factories: HashMap<String, Arc<dyn LintFactory>>,
}

// Define the static singleton instance
static INSTANCE: LazyLock<Mutex<LintConfigurationFactory>> =
    LazyLock::new(|| Mutex::new(LintConfigurationFactory::new()));

impl LintConfigurationFactory {
    /// Get a mutable reference to the global instance of the factory
    fn get_instance() -> std::sync::MutexGuard<'static, LintConfigurationFactory> {
        INSTANCE
            .lock()
            .expect("Failed to lock the global LintConfigurationFactory")
    }

    /// Create a new, empty factory
    pub(crate) fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a `LintFactory` under a given name
    pub fn register_lint_factory<T: LintFactory + 'static>(name: &str, factory: T) {
        let mut instance = LintConfigurationFactory::get_instance();
        instance
            .factories
            .insert(name.to_string(), Arc::new(factory));
    }

    /// Load a YAML configuration and produce a list of `ArchitectureLintRule`s
    pub fn from_yaml(yaml: String) -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        let config: serde_yaml::Value = serde_yaml::from_str(&yaml)?;

        let mut rules = vec![];

        // Extract the factories once, then drop `instance`
        let factories = {
            let instance = LintConfigurationFactory::get_instance();
            instance.factories.clone()
        };

        if let Some(mapping) = config.as_mapping() {
            for (rule_name, value) in mapping {
                let rule_config = value
                    .as_mapping()
                    .ok_or_else(|| anyhow!("Invalid rule format for {:?}", rule_name))?;

                let lint_type = rule_config
                    .get(serde_yaml::Value::String("type".to_string()))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing `type` field in rule {:?}", rule_name))?;

                let rule_name = rule_name
                    .as_str()
                    .expect("missing rule name on dictionary can never happen!");

                // Lookup factory in `factories` (independent of `instance`)
                let factory = factories
                    .get(lint_type)
                    .cloned()
                    .ok_or_else(|| anyhow!("Unknown lint type: {}", lint_type))?;

                let mut rule_config_map = rule_config.clone();
                rule_config_map.remove(serde_yaml::Value::String("type".to_string()));

                let config = &serde_yaml::Value::Mapping(rule_config_map);

                let mut lint_rules = factory.configure(rule_name, config)?;

                rules.append(&mut lint_rules);
            }
        }

        Ok(rules)
    }

    /// Generate a configuration file based on the current context
    pub fn generate_yaml(context: &ProjectContext) -> Result<String> {
        // Get all factories
        let factories = {
            let instance = LintConfigurationFactory::get_instance();
            instance.factories.clone()
        };

        let mut yaml_parts = Vec::new();
        yaml_parts.push("# Generated configuration file\n#\n# This file contains recommended lint rules for your project\n".to_string());

        // Generate config from each factory
        for (lint_type, factory) in factories {
            let configs = factory.generate_config(context)?;
            if !configs.is_empty() {
                yaml_parts.push(format!("\n# {}\n", lint_type));

                for (rule_name, config_yaml) in configs {
                    // Add proper indentation to the config content
                    let indented_yaml = config_yaml
                        .lines()
                        .map(|line| {
                            if line.trim().is_empty() {
                                line.to_string()
                            } else {
                                format!("  {}", line)
                            }
                        })
                        .collect::<Vec<String>>()
                        .join("\n");

                    yaml_parts.push(format!("{}:\n{}\n", rule_name, indented_yaml));
                }
            }
        }

        Ok(yaml_parts.join("\n"))
    }

    /// Generate a configuration file and write it to disk
    pub fn generate_config_file(context: &ProjectContext, path: &str) -> Result<()> {
        let yaml = Self::generate_yaml(context)?;
        let mut file = fs::File::create(path)?;
        file.write_all(yaml.as_bytes())?;
        Ok(())
    }
}

pub fn setup_lints_yaml() -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
    use std::fs;
    
    // First try the current directory
    let yaml_content = match fs::read_to_string("pup.yaml") {
        Ok(content) => content,
        Err(_e) => {
            // If that fails, try the parent directory (in case we're in a subdirectory)
            match fs::read_to_string("../pup.yaml") {
                Ok(content) => content,
                Err(_) => {
                    // If that fails too, try the original relative path
                    match fs::read_to_string("../../pup.yaml") {
                        Ok(content) => content,
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                return Ok(Vec::new());
                            }
                            return Err(anyhow::Error::from(e));
                        }
                    }
                }
            }
        }
    };

    let lint_rules =
        LintConfigurationFactory::from_yaml(yaml_content).map_err(anyhow::Error::msg)?;

    Ok(lint_rules)
}
