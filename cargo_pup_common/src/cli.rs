// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use serde::{Deserialize, Serialize};

// PUP Commands
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PupCommand {
    PrintModules,
    PrintTraits,
    Check,
    GenerateConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PupCli {
    pub command: PupCommand,
    pub config_path: Option<String>,
}

impl Default for PupCli {
    fn default() -> Self {
        Self {
            command: PupCommand::Check,
            config_path: None,
        }
    }
}

#[allow(dead_code)]
impl PupCli {
    pub fn to_env_str(&self) -> String {
        serde_json::to_string(self).expect("Failed serializing CLI args")
    }

    pub fn from_env_str(env_str: &str) -> PupCli {
        serde_json::from_str(env_str).expect("Failed deserializing CLI args")
    }
}

#[allow(dead_code)]
pub struct PupArgs {
    pub command: PupCommand,
    pub config_path: Option<String>,
    pub cargo_args: Vec<String>,
}

impl PupArgs {
    #[allow(dead_code)]
    pub fn parse<I>(args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let mut command = PupCommand::Check; // Default command
        let mut config_path = None;

        // Convert args to a vector for easier processing
        let args: Vec<String> = args.collect();

        // Find the start of actual arguments after program name and possibly 'cargo pup'
        let mut start_idx = 1;

        // If invoked as 'cargo pup', skip those
        if args.get(1) == Some(&"pup".to_string()) {
            start_idx = 2;
        }

        // Check if there are any arguments left
        if args.len() > start_idx {
            // Check if the next arg is a command
            match args[start_idx].as_str() {
                "print-modules" => {
                    command = PupCommand::PrintModules;
                    start_idx += 1;
                }
                "print-traits" => {
                    command = PupCommand::PrintTraits;
                    start_idx += 1;
                }
                "check" => {
                    command = PupCommand::Check;
                    start_idx += 1;
                }
                "generate-config" => {
                    command = PupCommand::GenerateConfig;
                    start_idx += 1;
                }
                _ => { /* Not a command, use default and keep this arg */ }
            }
        }

        // Look for --pup-config argument
        let mut filtered_cargo_args = Vec::new();
        let mut i = start_idx;
        while i < args.len() {
            if args[i] == "--pup-config" {
                // Check if there's a value after --pup-config
                if i + 1 < args.len() {
                    config_path = Some(args[i + 1].clone());
                    i += 2; // Skip both the flag and its value
                } else {
                    // Missing value for --pup-config
                    eprintln!("Warning: --pup-config flag requires a path argument");
                    i += 1;
                }
            } else {
                // Not a special flag, add to cargo args
                filtered_cargo_args.push(args[i].clone());
                i += 1;
            }
        }

        Self {
            command,
            config_path,
            cargo_args: filtered_cargo_args,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> PupArgs {
        let args = args.iter().map(|s| s.to_string());
        PupArgs::parse(args)
    }

    #[test]
    fn test_basic_command_parsing() {
        // Test default command
        let args = parse_args(&["cargo-pup"]);
        assert_eq!(args.command, PupCommand::Check);
        assert!(args.cargo_args.is_empty());

        // Test print-modules command
        let args = parse_args(&["cargo-pup", "print-modules"]);
        assert_eq!(args.command, PupCommand::PrintModules);
        assert!(args.cargo_args.is_empty());

        // Test print-traits command
        let args = parse_args(&["cargo-pup", "print-traits"]);
        assert_eq!(args.command, PupCommand::PrintTraits);
        assert!(args.cargo_args.is_empty());

        // Test check command
        let args = parse_args(&["cargo-pup", "check"]);
        assert_eq!(args.command, PupCommand::Check);
        assert!(args.cargo_args.is_empty());

        // Test generate-config command
        let args = parse_args(&["cargo-pup", "generate-config"]);
        assert_eq!(args.command, PupCommand::GenerateConfig);
        assert!(args.cargo_args.is_empty());
    }

    #[test]
    fn test_command_with_cargo_args() {
        // Test with cargo features flag
        let args = parse_args(&["cargo-pup", "print-modules", "--features=foo"]);
        assert_eq!(args.command, PupCommand::PrintModules);
        assert_eq!(args.cargo_args, vec!["--features=foo"]);

        // Test with multiple cargo args
        let args = parse_args(&[
            "cargo-pup",
            "check",
            "--features=foo",
            "--manifest-path=Cargo.toml",
        ]);
        assert_eq!(args.command, PupCommand::Check);
        assert_eq!(
            args.cargo_args,
            vec!["--features=foo", "--manifest-path=Cargo.toml"]
        );
    }

    #[test]
    fn test_with_cargo_pup_invocation() {
        // When invoked via cargo pup
        let args = parse_args(&["cargo", "pup", "print-modules"]);
        assert_eq!(args.command, PupCommand::PrintModules);
        assert!(args.cargo_args.is_empty());

        // With cargo args
        let args = parse_args(&["cargo", "pup", "print-modules", "--features=foo"]);
        assert_eq!(args.command, PupCommand::PrintModules);
        assert_eq!(args.cargo_args, vec!["--features=foo"]);
    }

    #[test]
    fn test_pup_config_argument() {
        // Test with --pup-config argument
        let args = parse_args(&["cargo-pup", "check", "--pup-config", "/tmp/pup.ron"]);
        assert_eq!(args.command, PupCommand::Check);
        assert_eq!(args.config_path, Some("/tmp/pup.ron".to_string()));
        assert!(args.cargo_args.is_empty());

        // Test with cargo args along with --pup-config
        let args = parse_args(&[
            "cargo-pup",
            "check",
            "--pup-config",
            "/tmp/pup.ron",
            "--features=foo",
        ]);
        assert_eq!(args.command, PupCommand::Check);
        assert_eq!(args.config_path, Some("/tmp/pup.ron".to_string()));
        assert_eq!(args.cargo_args, vec!["--features=foo"]);

        // Test via cargo pup
        let args = parse_args(&["cargo", "pup", "check", "--pup-config", "/tmp/pup.ron"]);
        assert_eq!(args.command, PupCommand::Check);
        assert_eq!(args.config_path, Some("/tmp/pup.ron".to_string()));
        assert!(args.cargo_args.is_empty());
    }

    #[test]
    fn test_unknown_args() {
        // When invoked with unknown args, should use default command
        let args = parse_args(&["cargo-pup", "--features=foo"]);
        assert_eq!(args.command, PupCommand::Check); // Default command
        assert_eq!(args.cargo_args, vec!["--features=foo"]);

        // With unknown subcommand, should use default command
        let args = parse_args(&["cargo-pup", "unknown-command", "--features=foo"]);
        assert_eq!(args.command, PupCommand::Check); // Default command
        assert_eq!(args.cargo_args, vec!["unknown-command", "--features=foo"]);
    }

    #[test]
    fn test_real_world_examples() {
        // Our specific troublesome case
        let args = parse_args(&["cargo", "pup", "print-modules", "--features=test-feature"]);
        assert_eq!(args.command, PupCommand::PrintModules);
        assert_eq!(args.cargo_args, vec!["--features=test-feature"]);
    }
}
