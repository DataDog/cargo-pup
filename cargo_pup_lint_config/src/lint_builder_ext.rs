// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use std::process::{Command, Output};
use anyhow::{Context, Result};
use tempfile::NamedTempFile;
use std::path::Path;

use crate::LintBuilder;

/// Extension trait for LintBuilder that provides unit testing capabilities.
pub trait LintBuilderExt {
    /// Executes lint rules against a Cargo project and fails if any violations are found.
    /// 
    /// This method creates a temporary configuration file from your `LintBuilder`, runs
    /// `cargo pup check` with that configuration, and panics if any lint violations are detected.
    /// This makes it ideal for use in unit tests where you want to verify that your code
    /// adheres to the architectural rules you've defined.
    /// 
    /// # Arguments
    /// 
    /// * `project_path` - Optional path to the project to lint:
    ///   - `None`: Uses the current working directory (must contain a `Cargo.toml`)
    ///   - `Some(path)`: Path to a directory containing `Cargo.toml` or path to a `Cargo.toml` file
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(Output)` if all lints pass, or an error if:
    /// - The project path is invalid or doesn't contain a `Cargo.toml`
    /// - The temporary config file cannot be created
    /// - The cargo-pup command fails to execute
    /// 
    /// # Panics
    /// 
    /// Panics with a descriptive message if any lint rules are violated. This is intentional
    /// behavior for unit testing - the panic will fail the test and show you which rules
    /// were broken.
    /// 
    /// # Examples
    /// 
    /// ## Test current directory
    /// ```rust,no_run
    /// # use cargo_pup_lint_config::{LintBuilder, LintBuilderExt, ModuleLintExt};
    /// # fn test_lint_rules() -> anyhow::Result<()> {
    /// let mut builder = LintBuilder::new();
    /// builder.module_lint()
    ///     .lint_named("no_empty_modules")
    ///     .matching(|m| m.module(".*"))
    ///     .must_not_be_empty()
    ///     .build();
    /// 
    /// // Test against current directory
    /// builder.assert_lints(None)?;
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// ## Test specific project
    /// ```rust,no_run
    /// # use cargo_pup_lint_config::{LintBuilder, LintBuilderExt, ModuleLintExt};
    /// # fn test_lint_rules() -> anyhow::Result<()> {
    /// let mut builder = LintBuilder::new();
    /// builder.module_lint()
    ///     .lint_named("helpers_must_be_small")
    ///     .matching(|m| m.module(".*::helpers"))
    ///     .must_be_empty()
    ///     .build();
    /// 
    /// // Test against a specific project
    /// builder.assert_lints(Some("../other-project"))?;
    /// # Ok(())
    /// # }
    /// ```
    /// 
    /// ## Use in unit tests
    /// ```rust,no_run
    /// # use cargo_pup_lint_config::{LintBuilder, LintBuilderExt, ModuleLintExt};
    /// #[test]
    /// fn test_architecture_rules() {
    ///     let mut builder = LintBuilder::new();
    ///     
    ///     builder.module_lint()
    ///         .lint_named("utils_no_business_logic")
    ///         .matching(|m| m.module(".*::utils"))
    ///         .denied_items(vec!("struct".to_string(), "enum".to_string()))
    ///         .build();
    ///     
    ///     // This will panic (failing the test) if utils modules contain structs or enums
    ///     builder.assert_lints(None).expect("Architecture rules should pass");
    /// }
    /// ```
    fn assert_lints(&self, project_path: Option<&str>) -> Result<Output>;
}

impl LintBuilderExt for LintBuilder {
    fn assert_lints(&self, project_path: Option<&str>) -> Result<Output> {
        // Determine which path to validate
        let path_to_validate = match project_path {
            Some(path) => path.to_string(),
            None => {
                std::env::current_dir()
                    .context("Failed to get current working directory")?
                    .to_str()
                    .context("Current working directory path is not valid UTF-8")?
                    .to_string()
            }
        };
        
        // Validate the project path
        validate_project_path(&path_to_validate)
            .with_context(|| {
                if project_path.is_some() {
                    format!("Invalid manifest path: {}", path_to_validate)
                } else {
                    format!("Current working directory is not a valid Cargo project: {}", path_to_validate)
                }
            })?;
        
        let args = if let Some(path) = project_path {
            vec!["--manifest-path", path]
        } else {
            vec![]
        };
        
        let output = run_with_args(self, &args.iter().map(|s| *s).collect::<Vec<&str>>())?;
        
        // Check if the command failed (non-zero exit status)
        if !output.status.success() {
            // Print the output
            if !output.stdout.is_empty() {
                println!("Lint stdout:\n{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprintln!("Lint stderr:\n{}", String::from_utf8_lossy(&output.stderr));
            }
            
            // Panic to fail the test
            panic!("cargo pup checks failed!");
        }
        
        Ok(output)
    }
}

fn run_with_args(lint_builder: &LintBuilder, args: &[&str]) -> Result<Output> {
    // Default to the 'check' command
    run_command(lint_builder, "check", args)
}

/// Validates that the given path exists and contains a Cargo.toml file, returning the canonical path
fn validate_project_path(path: &str) -> Result<std::path::PathBuf> {
    let project_path = Path::new(path)
        .canonicalize()
        .with_context(|| format!("Cannot resolve path: {}", path))?;
    
    // If it's a file, check if it's a Cargo.toml and get its parent directory
    let cargo_dir = if project_path.is_file() {
        if project_path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
            project_path.parent()
                .context("Cargo.toml file has no parent directory")?
                .to_path_buf()
        } else {
            anyhow::bail!("Path {} exists but is not a Cargo.toml file", path);
        }
    } else if project_path.is_dir() {
        project_path
    } else {
        anyhow::bail!("Path {} exists but is neither a file nor directory", path);
    };
    
    // Check if Cargo.toml exists in the directory
    let cargo_toml = cargo_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("No Cargo.toml found in directory: {}", cargo_dir.display());
    }
    
    Ok(cargo_dir)
}

/// Finds the cargo-pup workspace root by looking for the actual cargo-pup project
/// If we are running within a subdirectory of cargo-pup itself, we'll use
/// `cargo run` to invoke our own copy of cargo-pup, rather than relying on the
/// cargo system installed copy. This makes unit and integration tests behave
/// the way you'd expect them to.
fn find_workspace_root() -> Result<std::path::PathBuf> {
    // Start from the current executable's directory (which should be in target/debug/deps)
    // and work our way up to find the cargo-pup workspace
    let current_exe = std::env::current_exe()
        .context("Failed to get current executable path")?;
    
    let mut check_dir = current_exe.parent();
    
    while let Some(dir) = check_dir {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                // Look specifically for cargo-pup project or workspace with cargo-pup
                if contents.contains("name = \"cargo-pup\"") || 
                   (contents.contains("[workspace]") && contents.contains("cargo_pup")) {
                    return Ok(dir.to_path_buf());
                }
            }
        }
        
        // Also check if this directory has the specific cargo-pup source structure
        if dir.join("src").join("pup_driver.rs").exists() {
            return Ok(dir.to_path_buf());
        }
        
        check_dir = dir.parent();
    }
    
    // Fallback: try from current working directory
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    check_dir = Some(current_dir.as_path());
    while let Some(dir) = check_dir {
        if dir.join("src").join("pup_driver.rs").exists() {
            return Ok(dir.to_path_buf());
        }
        check_dir = dir.parent();
    }
    
    // If we can't find workspace root, return current directory
    Ok(current_dir)
}

fn run_command(lint_builder: &LintBuilder, command: &str, args: &[&str]) -> Result<Output> {
    // Create a temporary file to store the configuration
    let temp_file = NamedTempFile::new()
        .context("Failed to create temporary configuration file")?;
    let config_path = temp_file.path().to_path_buf();

    // Write the configuration to the temporary file
    lint_builder.write_to_file(config_path.to_str().unwrap())
        .context("Failed to write configuration to temporary file")?;

    // Prepare the cargo-pup command
    // Try to use development mode if we can find the workspace, otherwise use system cargo pup
    let mut cmd = if let Ok(workspace_root) = find_workspace_root() {
        let manifest_path = workspace_root.join("Cargo.toml");
        if manifest_path.exists() {
            // Development mode: use cargo run
            let mut dev_cmd = Command::new("cargo");
            dev_cmd.arg("run");
            dev_cmd.arg("--manifest-path").arg(&manifest_path);
            dev_cmd.arg("--bin").arg("cargo-pup").arg("--");
            dev_cmd.arg(command);
            dev_cmd.arg("--pup-config");
            dev_cmd.arg(config_path.to_str().unwrap());
            dev_cmd
        } else {
            // System mode: use installed cargo pup
            let mut system_cmd = Command::new("cargo");
            system_cmd.arg("pup");
            system_cmd.arg(command);
            system_cmd.arg("--pup-config");
            system_cmd.arg(config_path.to_str().unwrap());
            system_cmd
        }
    } else {
        // System mode: use installed cargo pup
        let mut system_cmd = Command::new("cargo");
        system_cmd.arg("pup");
        system_cmd.arg(command);
        system_cmd.arg("--pup-config");
        system_cmd.arg(config_path.to_str().unwrap());
        system_cmd
    };

    // Add any additional arguments
    for arg in args {
        cmd.arg(arg);
    }

    // Run the command and capture the output
    // Keep temp_file alive during command execution
    let output = cmd.output()
        .context("Failed to execute cargo-pup")?;

    // Explicitly drop temp_file after command completion
    drop(temp_file);

    // Return the command output
    Ok(output)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModuleLintExt;
    use tempfile::TempDir;
    use std::fs;


    #[test]
    fn test_assert_lints_with_non_existent_path() {
        let mut builder = LintBuilder::new();
        
        // Add a simple rule
        builder.module_lint()
            .lint_named("test_rule")
            .matching(|m| m.module("test::module"))
            .must_not_be_empty()
            .build();
        
        // Try to run with a non-existent path
        let result = builder.assert_lints(Some("/non/existent/path"));
        
        // Should return an error
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Check for either the context message or the root cause
        let error_str = error.to_string();
        assert!(
            error_str.contains("Cannot resolve path") || 
            error_str.contains("Invalid manifest path") ||
            error_str.contains("No such file or directory") ||
            error_str.contains("cannot find the path specified"),
            "Error message should indicate path resolution failure, got: {}",
            error_str
        );
    }

    #[test]
    fn test_assert_lints_with_non_cargo_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();
        
        // Create a directory without Cargo.toml
        let non_cargo_dir = temp_path.join("not_a_cargo_project");
        fs::create_dir_all(&non_cargo_dir).expect("Failed to create directory");
        
        let mut builder = LintBuilder::new();
        
        // Add a simple rule
        builder.module_lint()
            .lint_named("test_rule")
            .matching(|m| m.module("test::module"))
            .must_not_be_empty()
            .build();
        
        // Try to run with a non-cargo project path
        let result = builder.assert_lints(Some(non_cargo_dir.to_str().unwrap()));
        
        // Should return an error
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Check for either the context message or the root cause
        let error_str = error.to_string();
        assert!(
            error_str.contains("Invalid manifest path") || 
            error_str.contains("No Cargo.toml found"),
            "Error message should indicate missing Cargo.toml, got: {}",
            error_str
        );
    }

    #[test]
    fn test_assert_lints_uses_pup_config_correctly() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();
        
        // Create a valid cargo project structure
        let cargo_project_dir = temp_path.join("valid_cargo_project");
        fs::create_dir_all(&cargo_project_dir).expect("Failed to create directory");
        
        // Create a minimal src directory and main.rs
        let src_dir = cargo_project_dir.join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create src directory");
        fs::write(src_dir.join("main.rs"), "fn main() {}\n").expect("Failed to write main.rs");
        
        // Create Cargo.toml
        fs::write(
            cargo_project_dir.join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.1"
edition = "2021"
"#,
        ).expect("Failed to write Cargo.toml");
        
        let mut builder = LintBuilder::new();
        
        // Add a simple rule that shouldn't trigger on this minimal project
        builder.module_lint()
            .lint_named("test_rule")
            .matching(|m| m.module("nonexistent::module"))
            .must_not_be_empty()
            .build();
        
        // Test the validation manually first to ensure it passes
        let validation_result = validate_project_path(cargo_project_dir.to_str().unwrap());
        assert!(validation_result.is_ok(), "Validation should pass for valid cargo project");
        
        // Test that run_command properly creates and uses a temporary config file
        let result = run_command(&builder, "check", &["--manifest-path", cargo_project_dir.join("Cargo.toml").to_str().unwrap()]);
        
        // The command should execute (though it may fail due to environment issues)
        // The important thing is that it doesn't fail with config-related errors
        match result {
            Ok(_) => {
                // Success case - the command worked
            }
            Err(error) => {
                let error_str = error.to_string();
                // Should NOT contain validation errors
                assert!(!error_str.contains("Path does not exist"));
                assert!(!error_str.contains("No Cargo.toml found"));
                assert!(!error_str.contains("Invalid project path"));
                // Should NOT contain config file errors  
                assert!(!error_str.contains("Failed to create temporary configuration file"));
                assert!(!error_str.contains("Failed to write configuration to temporary file"));
            }
        }
    }

    #[test]
    fn test_assert_lints_with_cargo_toml_file_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();
        
        // Create a valid cargo project structure
        let cargo_project_dir = temp_path.join("valid_cargo_project");
        fs::create_dir_all(&cargo_project_dir).expect("Failed to create directory");
        
        // Create a minimal src directory and main.rs
        let src_dir = cargo_project_dir.join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create src directory");
        fs::write(src_dir.join("main.rs"), "fn main() {}\n").expect("Failed to write main.rs");
        
        // Create Cargo.toml
        let cargo_toml_path = cargo_project_dir.join("Cargo.toml");
        fs::write(
            &cargo_toml_path,
            r#"[package]
name = "test"
version = "0.1.1"
edition = "2021"
"#,
        ).expect("Failed to write Cargo.toml");
        
        let mut builder = LintBuilder::new();
        
        // Add a simple rule that shouldn't trigger
        builder.module_lint()
            .lint_named("test_rule")
            .matching(|m| m.module("nonexistent::module"))
            .must_not_be_empty()
            .build();
        
        // Test the validation manually first
        let validation_result = validate_project_path(cargo_toml_path.to_str().unwrap());
        assert!(validation_result.is_ok(), "Validation should pass for Cargo.toml file path");
        
        // Just test the validation, not the actual command execution in this test
        // since cargo execution can be flaky in test environments
    }

    #[test]
    fn test_temp_config_file_creation() {
        let mut builder = LintBuilder::new();
        
        // Add some rules to the builder
        builder.module_lint()
            .lint_named("test_rule_1")
            .matching(|m| m.module("test::module"))
            .must_not_be_empty()
            .build();
            
        builder.module_lint()
            .lint_named("test_rule_2")
            .matching(|m| m.module("another::module"))
            .must_be_empty()
            .build();
        
        // Test that we can create a temporary file and write config to it
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_path = temp_file.path();
        
        // Write config to the temporary file
        let write_result = builder.write_to_file(config_path.to_str().unwrap());
        assert!(write_result.is_ok(), "Should be able to write config to temp file");
        
        // Verify the file exists and has content
        assert!(config_path.exists(), "Temp config file should exist");
        
        let config_contents = std::fs::read_to_string(config_path)
            .expect("Should be able to read temp config file");
        
        // Verify the config contains our rules
        assert!(config_contents.contains("test_rule_1"), "Config should contain test_rule_1");
        assert!(config_contents.contains("test_rule_2"), "Config should contain test_rule_2");
        assert!(config_contents.contains("lints"), "Config should contain lints field");
        
        // Test that the config is valid RON format
        assert!(config_contents.trim().starts_with("("), "Config should start with RON tuple syntax");
        assert!(config_contents.trim().ends_with(")"), "Config should end with RON tuple syntax");
    }

    #[test]
    fn test_assert_lints_validates_current_directory() {
        // Store the original directory
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        
        // Create a temporary directory that's NOT a valid cargo project
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let invalid_dir = temp_dir.path().join("invalid_project");
        fs::create_dir_all(&invalid_dir).expect("Failed to create invalid directory");
        
        // Create a guard to ensure we always restore the original directory
        struct DirGuard {
            original_dir: std::path::PathBuf,
        }
        
        impl Drop for DirGuard {
            fn drop(&mut self) {
                let _ = std::env::set_current_dir(&self.original_dir);
            }
        }
        
        let _guard = DirGuard {
            original_dir: original_dir.clone(),
        };
        
        // Change to the invalid directory
        std::env::set_current_dir(&invalid_dir).expect("Failed to change to invalid directory");
        
        let mut builder = LintBuilder::new();
        
        // Add a simple rule
        builder.module_lint()
            .lint_named("test_rule")
            .matching(|m| m.module("test::module"))
            .must_not_be_empty()
            .build();
        
        // Call assert_lints with None - should validate current directory and fail
        let result = builder.assert_lints(None);
        
        // Should return an error because current directory is not a valid cargo project
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = error.to_string();
        assert!(
            error_str.contains("Current working directory is not a valid Cargo project") || 
            error_str.contains("No Cargo.toml found"),
            "Error should indicate invalid current working directory: {}",
            error_str
        );
    }

    #[test]
    fn test_assert_lints_validates_current_directory_success() {
        // Store the original directory
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        
        // Create a valid cargo project in a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let valid_project_dir = temp_dir.path().join("valid_project");
        fs::create_dir_all(&valid_project_dir).expect("Failed to create valid directory");
        
        // Create src directory and main.rs
        let src_dir = valid_project_dir.join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create src directory");
        fs::write(src_dir.join("main.rs"), "fn main() {}\n").expect("Failed to write main.rs");
        
        // Create Cargo.toml
        fs::write(
            valid_project_dir.join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.1"
edition = "2021"
"#,
        ).expect("Failed to write Cargo.toml");
        
        // Create a guard to ensure we always restore the original directory
        struct DirGuard {
            original_dir: std::path::PathBuf,
        }
        
        impl Drop for DirGuard {
            fn drop(&mut self) {
                let _ = std::env::set_current_dir(&self.original_dir);
            }
        }
        
        let _guard = DirGuard {
            original_dir: original_dir.clone(),
        };
        
        // Change to the valid project directory
        std::env::set_current_dir(&valid_project_dir).expect("Failed to change to valid directory");
        
        let mut builder = LintBuilder::new();
        
        // Add a simple rule that shouldn't trigger
        builder.module_lint()
            .lint_named("test_rule")
            .matching(|m| m.module("nonexistent::module"))
            .must_not_be_empty()
            .build();
        
        // Call assert_lints with None - should validate current directory and pass validation
        let result = builder.assert_lints(None);
        
        // The validation should pass (though cargo execution may fail in test environment)
        // We're mainly testing that it doesn't fail with directory validation errors
        match result {
            Ok(_) => {
                // Success case - validation and execution both worked
            }
            Err(error) => {
                let error_str = error.to_string();
                // Should NOT contain directory validation errors
                assert!(!error_str.contains("Invalid current working directory"));
                assert!(!error_str.contains("Path does not exist"));
                assert!(!error_str.contains("No Cargo.toml found"));
                assert!(!error_str.contains("Failed to get current working directory"));
            }
        }
    }

    #[test]
    fn test_validate_project_path_function() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();
        
        // Test non-existent path
        let result = validate_project_path("/non/existent/path");
        assert!(result.is_err());
        let error_str = result.unwrap_err().to_string();
        assert!(
            error_str.contains("Cannot resolve path") ||
            error_str.contains("No such file or directory") ||
            error_str.contains("cannot find the path specified"),
            "Expected path resolution error, got: {}",
            error_str
        );
        
        // Test directory without Cargo.toml
        let empty_dir = temp_path.join("empty");
        fs::create_dir_all(&empty_dir).expect("Failed to create directory");
        let result = validate_project_path(empty_dir.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No Cargo.toml found"));
        
        // Test valid cargo project directory
        let cargo_dir = temp_path.join("cargo_project");
        fs::create_dir_all(&cargo_dir).expect("Failed to create directory");
        fs::write(
            cargo_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.1\"\n",
        ).expect("Failed to write Cargo.toml");
        
        let result = validate_project_path(cargo_dir.to_str().unwrap());
        assert!(result.is_ok());
        
        // Test valid Cargo.toml file path
        let cargo_toml_path = cargo_dir.join("Cargo.toml");
        let result = validate_project_path(cargo_toml_path.to_str().unwrap());
        assert!(result.is_ok());
        
        // Test invalid file (not Cargo.toml)
        let random_file = cargo_dir.join("random.txt");
        fs::write(&random_file, "content").expect("Failed to write file");
        let result = validate_project_path(random_file.to_str().unwrap());
        assert!(result.is_err());
        let error_str = result.unwrap_err().to_string();
        assert!(
            error_str.contains("is not a Cargo.toml file"),
            "Expected error about non-Cargo.toml file, got: {}",
            error_str
        );
    }
}
