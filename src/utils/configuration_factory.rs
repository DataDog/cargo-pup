use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use crate::lints::ArchitectureLintRule;
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
}

pub fn setup_lints_yaml() -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
    use std::fs;

    // Attempt to load configuration from `pup.yaml`
    let yaml_content = fs::read_to_string("pup.yaml")?.to_string();
    let lint_rules =
        LintConfigurationFactory::from_yaml(yaml_content).map_err(anyhow::Error::msg)?;

    Ok(lint_rules)
}
