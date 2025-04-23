//! Integration tests for toolchain handling
//! 
//! These tests verify that toolchain handling is consistent between
//! the cargo command and the pup-driver command.
//!
//! The most important function we're testing is that cargo is executed 
//! with the same toolchain as pup-driver, which is what ensures we don't
//! get a version mismatch between the toolchains.

use std::process::Command;
use std::env;
use std::path::PathBuf;

// Helper function to get the path to the cargo-pup binary
fn get_cargo_pup_path() -> PathBuf {
    // First try finding it in the path - useful for development
    if let Ok(path) = which::which("cargo-pup") {
        return path;
    }
    
    // Otherwise build it from the target directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    
    let target_dir = env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| format!("{}/target", manifest_dir));
    
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    
    // On Windows add the .exe extension
    let executable_name = if cfg!(windows) {
        "cargo-pup.exe"
    } else {
        "cargo-pup"
    };
    
    PathBuf::from(target_dir)
        .join(profile)
        .join(executable_name)
}

#[test]
fn test_consistent_toolchain_handling() {
    // Skip this test if it's running in CI or if we can't find rustup
    if env::var("CI").is_ok() || which::which("rustup").is_err() {
        println!("Skipping toolchain integration test in CI environment");
        return;
    }
    
    // This test captures command invocations and verifies they use rustup consistently
    // We use RUST_LOG=trace to capture detailed logs
    
    // Get path to cargo-pup
    let cargo_pup_path = get_cargo_pup_path();
    
    // Create a temporary directory for our test
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    
    // Create a basic Cargo.toml and pup.yaml
    std::fs::write(
        temp_path.join("Cargo.toml"),
        r#"
            [package]
            name = "test_app"
            version = "0.1.0"
            edition = "2024"
        "#,
    ).expect("Failed to write Cargo.toml");
    
    std::fs::write(
        temp_path.join("pup.yaml"),
        r#"
            test_rule:
              type: function_length
              namespace: "test_app"
              max_lines: 5
              severity: Warn
        "#,
    ).expect("Failed to write pup.yaml");
    
    // Create a basic Rust source file
    std::fs::create_dir_all(temp_path.join("src")).expect("Failed to create src dir");
    std::fs::write(
        temp_path.join("src/main.rs"),
        r#"
            fn main() {
                println!("Hello, world!");
            }
        "#,
    ).expect("Failed to write main.rs");
    
    // Run cargo-pup with tracing to capture invocations
    let output = Command::new(&cargo_pup_path)
        .current_dir(temp_path)
        .env("RUST_LOG", "trace")
        .args(["check"])
        .output()
        .expect("Failed to run cargo-pup");
    
    // Check if the command succeeded
    assert!(output.status.success(), "cargo-pup failed to run: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    // Get the output and check for rustup invocations
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // For actual validation, we can't directly inspect the commands
    // but we can check for common error patterns that would indicate a toolchain mismatch
    
    // Check for known error patterns that would indicate a toolchain mismatch
    let error_patterns = [
        "incompatible version of rustc",
        "compiled by an incompatible version",
        "please recompile that crate using this compiler",
    ];
    
    for pattern in error_patterns {
        assert!(!stderr.contains(pattern), 
            "Found error pattern indicating toolchain mismatch: {}", pattern);
    }
    
    // Clean up
    temp_dir.close().expect("Failed to clean up temp directory");
}