use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use crate::lints::ArchitectureLintRule;
use anyhow::{anyhow, Result};

pub trait LintFactory: Send + Sync {
    ///
    /// Registers itself with the ConfigurationFactory
    ///
    fn register()
    where
        Self: Sized;

    ////
    /// Create a list of `ArchitectureLintRule`s from a YAML configuration
    ///
    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>>;
}

pub struct LintConfigurationFactory {
    factories: HashMap<String, Box<dyn LintFactory>>,
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
    fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a `LintFactory` under a given name
    pub fn register_lint_factory<T: LintFactory + 'static>(name: &str, factory: T) {
        let mut instance = LintConfigurationFactory::get_instance();
        instance
            .factories
            .insert(name.to_string(), Box::new(factory));
    }

    /// Load a YAML configuration and produce a list of `ArchitectureLintRule`s
    pub fn from_yaml(yaml: &str) -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        let instance = LintConfigurationFactory::get_instance();

        let config: serde_yaml::Value = serde_yaml::from_str(yaml)?;

        let mut rules = vec![];

        if let Some(mapping) = config.as_mapping() {
            for (rule_name, value) in mapping {
                let rule_config = value
                    .as_mapping()
                    .ok_or_else(|| anyhow!("Invalid rule format for {:?}", rule_name))?;

                // Extract the `type` field
                let lint_type = rule_config
                    .get(serde_yaml::Value::String("type".to_string()))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing `type` field in rule {:?}", rule_name))?;

                // Extract the rule name
                let rule_name = rule_name
                    .as_str()
                    .expect("missing rule name on dictionary can never happen!");

                // Find the factory
                let factory = instance
                    .factories
                    .get(lint_type)
                    .ok_or_else(|| anyhow!("Unknown lint type: {}", lint_type))?;

                // Pass the entire rule configuration (minus `type`) to the factory
                let mut rule_config_map = rule_config.clone();
                rule_config_map.remove(serde_yaml::Value::String("type".to_string()));

                let mut lint_rules =
                    factory.configure(rule_name, &serde_yaml::Value::Mapping(rule_config_map))?;
                rules.append(&mut lint_rules);
            }
        }

        Ok(rules)
    }

    // Serialize the current configuration to YAML
    // pub fn to_yaml(&self) -> Result<String> {
    //     let mut mapping = Mapping::new();

    //     for (name, factory) in &self.factories {
    //         let yaml_value = factory.to_yaml()?;
    //         mapping.insert(Value::String(name.clone()), yaml_value);
    //     }

    //     serde_yaml::to_string(&Value::Mapping(mapping))
    //         .map_err(|e| anyhow!("Failed to serialize to YAML: {}", e))
    // }
}
