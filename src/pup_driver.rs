#![feature(rustc_private)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(try_blocks)]

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;

use anyhow::Result;
use cargo_pup_common::cli::{PupCli, PupCommand};

use rustc_session::{EarlyDiagCtxt, config::ErrorOutputType};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};
use cargo_pup_lint_impl::{ArchitectureLintCollection, ArchitectureLintRunner, Mode};
use cargo_pup_lint_impl::lints::configuration_factory::LintConfigurationFactory;

pub fn main() -> Result<()> {

    let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&early_dcx);

    let mut orig_args: Vec<String> = env::args().collect();

    // Handle wrapper mode
    let wrapper_mode =
        orig_args.get(1).map(Path::new).and_then(Path::file_stem) == Some("rustc".as_ref());
    if wrapper_mode {
        orig_args.remove(1);
    }

    // Check if we're in UI testing mode by looking for the "-Zui-testing" flag
    let is_ui_testing = orig_args.iter().any(|arg| arg == "-Zui-testing");

    // Add `--sysroot` if missing
    if !orig_args.iter().any(|arg| arg == "--sysroot") {
        orig_args.extend(vec!["--sysroot".into(), find_sysroot()]);
    }

    // Default to check mode if we're in UI testing
    let mode = if is_ui_testing {
        Mode::Check
    } else {
        // Load our configuration from CLI args
        let binding = std::env::var("PUP_CLI_ARGS").unwrap_or_default();
        let cli_args = binding.as_str();
        let config = if cli_args.is_empty() {
            // Default to check mode if no CLI args
            PupCli::default()
        } else {
            PupCli::from_env_str(cli_args)
        };

        match config.command {
            PupCommand::PrintModules => Mode::PrintModules,
            PupCommand::PrintTraits => Mode::PrintTraits,
            PupCommand::Check => Mode::Check,
            PupCommand::GenerateConfig => Mode::GenerateConfig,
        }
    };

    // Log it, so we can work out what is going on
    log_invocation(&orig_args)?;

    // Parse cargo arguments from environment
    let mut cargo_args = Vec::new();
    if let Ok(args_str) = env::var("PUP_CARGO_ARGS") {
        for arg in args_str.split("__PUP_ARG_SEP__") {
            if !arg.is_empty() {
                cargo_args.push(arg.to_string());
            }
        }
    }

    // Determine the lint collection to use
    let lint_collection = if mode == Mode::GenerateConfig {
        // For generate-config mode, use an empty collection
        ArchitectureLintCollection::new(Vec::new())
    } else if is_ui_testing {
        // For UI testing, look for pup files in the same directory as the test file
        let source_file = find_source_file(&orig_args)?;
        let test_dir = source_file.parent().unwrap_or(Path::new("."));
        
        // Try loading from pup.ron
        let ron_path = test_dir.join("pup.ron");
        if ron_path.exists() {
            match LintConfigurationFactory::from_file(ron_path.to_str().unwrap().to_string()) {
                Ok(lint_rules) => {
                    ArchitectureLintCollection::new(lint_rules)
                },
                Err(e) => {
                    // In UI tests, print detailed error messages about configuration issues
                    panic!("UI TEST ERROR: Failed to parse pup.ron: {}", e);
                }
            }
        } else {
                eprintln!("UI TEST ERROR: No configuration file found in test directory");
                ArchitectureLintCollection::new(Vec::new())
            }
    } else {
        // For normal operation, load from pup.ron
        let cwd = env::current_dir()?;
        let ron_path = cwd.join("pup.ron");
        
        if ron_path.exists() {
            match LintConfigurationFactory::from_file(ron_path.to_str().unwrap().to_string()) {
                Ok(lint_rules) => {
                    ArchitectureLintCollection::new(lint_rules)
                },
                Err(e) => {
                    eprintln!("Failed to parse pup.ron: {}", e);
                    ArchitectureLintCollection::new(Vec::new())
                }
            }
        } else {
            // No configuration found
            eprintln!("No pup.ron configuration file found");
            ArchitectureLintCollection::new(Vec::new())
        }
    };

    // Prepare cli_args, either from environment or empty for UI testing
    let cli_args = if is_ui_testing {
        "".to_string()
    } else {
        std::env::var("PUP_CLI_ARGS").unwrap_or_default()
    };

    let mut runner = ArchitectureLintRunner::new(mode.clone(), cli_args, lint_collection);
    runner.set_cargo_args(cargo_args);

    rustc_driver::run_compiler(&orig_args, &mut runner);

    // Print out our lints
    if mode != Mode::GenerateConfig {
        let results_text = runner.lint_results_text();
        if !results_text.is_empty() {
            eprintln!("{0}", results_text);
        }
    }

    process::exit(0);
}

/// Find the source file from the command line arguments
fn find_source_file(args: &[String]) -> Result<PathBuf> {
    for arg in args {
        if arg.ends_with(".rs") && !arg.starts_with('-') {
            return Ok(PathBuf::from(arg));
        }
    }
    anyhow::bail!("No source file found in arguments")
}

fn find_sysroot() -> String {
    if let Ok(sysroot) = env::var("SYSROOT") {
        return sysroot;
    }

    if let Ok(rustup_home) = env::var("RUSTUP_HOME") {
        if let Ok(toolchain) = env::var("RUSTUP_TOOLCHAIN") {
            return format!("{rustup_home}/toolchains/{toolchain}");
        }
    }

    if let Ok(output) = Command::new("rustc").arg("--print").arg("sysroot").output() {
        if output.status.success() {
            return String::from_utf8(output.stdout).expect("Invalid UTF-8 in sysroot output");
        }
    }

    panic!("Could not determine sysroot.");
}

///
/// Appends the given arguments to `.pup/invocations.txt` in the current working directory
/// with a timestamp prepended.
///
fn log_invocation(orig_args: &[String]) -> std::io::Result<()> {
    let cwd = std::env::current_dir()?;
    let log_path = cwd.join(".pup/invocations.txt");

    // Ensure the directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Get current timestamp as seconds since UNIX epoch
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let args_str = orig_args.join(" ");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    writeln!(file, "[{}] {}", timestamp, args_str)?;

    Ok(())
}
