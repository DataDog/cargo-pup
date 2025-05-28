use std::process::{Command, Output};
use anyhow::{Context, Result};
use tempfile::NamedTempFile;
use std::path::Path;

use crate::LintBuilder;

/// Extension trait for LintBuilder that adds runtime linting capabilities
pub trait LintBuilderExt {
    /// Writes the current builder configuration to a temporary file and runs cargo-pup against it
    /// 
    /// # Arguments
    /// 
    /// * `project_path` - Optional path to the project to run lints on
    fn assert_lints(&self, project_path: Option<&str>) -> Result<Output>;

}

impl LintBuilderExt for LintBuilder {
    fn assert_lints(&self, project_path: Option<&str>) -> Result<Output> {
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

fn run_command(lint_builder: &LintBuilder, command: &str, args: &[&str]) -> Result<Output> {
    // Create a temporary file to store the configuration
    let temp_file = NamedTempFile::new()
        .context("Failed to create temporary configuration file")?;
    let config_path = temp_file.path().to_path_buf();

    // Write the configuration to the temporary file
    lint_builder.write_to_file(config_path.to_str().unwrap())
        .context("Failed to write configuration to temporary file")?;

    // Prepare the cargo-pup command
    let mut cmd = Command::new("cargo");
    cmd.arg("pup");
    cmd.arg(command);
    cmd.arg("--pup-config");
    cmd.arg(config_path.to_str().unwrap());

    // Add any additional arguments
    for arg in args {
        cmd.arg(arg);
    }

    // Run the command and capture the output
    let output = cmd.output()
        .context("Failed to execute cargo-pup")?;

    // Return the command output
    Ok(output)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_lint::ModuleLintBuilder;
    use crate::ModuleLintExt;

    // This test is just a compile-time check to ensure the trait is implemented correctly
    #[test]
    fn test_lint_builder_ext_compiles() {
        let mut builder = LintBuilder::new();
        
        // Add a simple rule
       builder.module_lint()
            .lint_named("test_rule")
            .matching(|m| m.module("test::module"))
            .must_not_be_empty()
            .build();
        
        // These should compile, even if we don't run them
        let _result = builder.assert_lints(None);
        let _result_with_path = builder.assert_lints(Some("../path/to/project"));
    }
}