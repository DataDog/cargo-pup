//!
//! cargo-pup
//! This is the entry point for our cargo extension, and what is ultimately run
//! when you type `cargo pup` on the command line. To run, it must be present in the
//! user's path. That's it!
//!
//!  # Running pup-driver
//!
//!   pup-driver itself links against librustc_dev, the rust compiler. The rust compiler's usage
//!   as a library is _only_ available in nightly, and can only be (easily, at least) linked dynamically.
//!   This means that in order to run our code, the user must have the same nightly toolchain that we
//!   were built against. Fortunately we need to trampoline through here, first (see below), and in this
//!   entry point, we have no dependency on librustc yet. So we can, in the process of trampolining,
//!   use rustup to ensure that the toolchain we need for pup-driver is installed.
//!
//!  # Proxying Rustc
//!
//! There is a bit of **magic** in here, which is derived from the way both clippy and
//! charon work. Ultimately what we need from cargo is the rust compilation command line -
//! this includes things like module paths, versions, configuration flags - everything you
//! need to actually be able to invoke the compiler. We then pass this along to pup-driver which
//! serves as our "rustc proxy", with our analysis code hooked in.
//!
//! To get this, we need to tell cargo to use us as a proxy for rustc commands. This is achieved by
//! setting RUSTC_WORKSPACE_WRAPPER, and pointing it back at ourselves - cargo-pup. We use then use
//! an environment variable PUP_TRAMPOLINE_MODE to work out if we're in the _first_ invocation of
//! `cargo pup` - what the user has typed onto the command line - or if we're the trampolined version,
//! where cargo has called back through us, with the compiler command line. This isn't explicitly necessary,
//! but it makes the logic a bit easier to follow.
//!
//! So, the overall execution flow looks like:
//!
//!   ## Initial execution
//!
//!   1. User types `cargo pup`.
//!   2. Cargo runs `cargo-pup` from the user's path.
//!      At this point, cargo has no "intent" apart from invoking us. E.g., it's not doing a 'build' or
//!      any other explicit goal. It's only task is to run cargo-pup.
//!   3. cargo-pup starts, and sees that it is _not_ in trampoline mode (PUP_TRAMPOLINE_MODE not set)
//!   4. cargo-pup ensures that the rustup toolchain needed to invoke pup-driver is installed,
//!      and installs it if it is not.
//!   4. cargo-pup forks a `cargo check`, whilst setting PUP_TRAMPOLINE_MODE=true and
//!      RUSTC_WORKSPACE_WRAPPER to point back to itself - e.g., the cargo-pup executable path.
//!
//!   ## Trampoline execution
//!
//!   5. Cargo runs again, and sees that it needs to run rustc, and that it needs to do so with a
//!      RUSTC_WORKSPACE_WRAPPER. Rather than invoking `rustc` directly, it invokes `cargo-pup` again,
//!      passing all the arguments it needs to pass to `rustc` normally. PUP_TRAMPOLINE_MODE is propagated
//!      in the environment.
//!   6. cargo-pup starts up, and notes it is in trampoline mode. It takes all of the rustc arguments it
//!      has been given, and forks pup-driver, passing them along.
//!
//!   ## Compilation
//!
//!   7. pup-driver runs, effectively wrapping up the rustc compilation process with our static analysis
//!
//!

#![feature(rustc_private)]
#![warn(rust_2018_idioms, unused_lifetimes)]

mod cli;

use cli::{PupArgs, PupCli};

use ansi_term::Colour::{Blue, Green, Red, Yellow, Cyan};
use ansi_term::Style;
use std::env;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::process::{Command, exit};

#[derive(Debug, PartialEq)]
enum ProjectType {
    ConfiguredPupProject,
    RustProject,
    OtherDirectory,
}

/// Simple error type that wraps a command exit code
#[derive(Debug)]
struct CommandExitStatus(i32);

impl fmt::Display for CommandExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command failed with exit code: {}", self.0)
    }
}

impl Error for CommandExitStatus {}

/// Validates the current directory to determine the project type
fn validate_project() -> ProjectType {
    let has_pup_yaml = Path::new("./pup.yaml").exists();
    let has_cargo_toml = Path::new("./Cargo.toml").exists();
    
    if has_pup_yaml && has_cargo_toml {
        ProjectType::ConfiguredPupProject
    } else if has_cargo_toml {
        ProjectType::RustProject
    } else {
        ProjectType::OtherDirectory
    }
}

fn show_ascii_puppy() {
    println!("{}", Cyan.paint(r#"
     / \__
    (    @\___
    /         O
   /   (_____/
  /_____/   U
"#));
}

fn show_help() {
    show_ascii_puppy();
    println!("{}", help_message());
}

fn show_version() {
    println!("{} {}", 
        Style::new().bold().paint("cargo-pup version"), 
        Green.paint(env!("CARGO_PKG_VERSION"))
    );
}

pub fn main() {
    // Handle help and version flags
    if env::args().any(|a| a == "--help" || a == "-h") {
        show_help();
        return;
    }

    if env::args().any(|a| a == "--version" || a == "-V") {
        show_version();
        return;
    }

    // Are we being invoked as a rustc wrapper?
    if env::args().len() > 1 && env::args().nth(1).is_some_and(|a| a.ends_with("rustc")) {
        // Special case: Handle rustc version query
        if env::args().len() > 2 && env::args().nth(2).is_some_and(|a| a == "-vV") {
            // Just run rustc with -vV to get its version
            let status = Command::new(env::args().nth(1).unwrap())
                .arg("-vV")
                .status()
                .expect("failed to run rustc");
            exit(status.code().unwrap_or(-1));
        }

        // Get the toolchain and run pup-driver with the args we've been given
        let toolchain = get_toolchain();
        if let Err(err) = run_pup_driver(&toolchain) {
            exit(err.0);
        }
        return;
    }

    // Parse command and arguments
    // Normal invocation - process args and run cargo
    let args: Vec<String> = env::args().collect();
    
    // Check if we're running generate-config
    let is_generate_config = args.len() > 1 && 
        ((args.len() > 2 && args[1] == "pup" && args[2] == "generate-config") || 
         (args[1] == "generate-config"));

    // Skip environment checks if we're generating a config
    if !is_generate_config {
        match validate_project() {
            ProjectType::ConfiguredPupProject => {
                // Good to go - continue with normal operation
            },
            ProjectType::RustProject => {
                // In a Rust project but missing pup.yaml
                show_ascii_puppy();
                println!("{}", Red.bold().paint("Missing pup.yaml - nothing to do!"));
                println!("Consider generating an initial configuration:");
                println!("  {}", Green.paint("cargo pup generate-config"));
                exit(-1)
            },
            ProjectType::OtherDirectory => {
                // Not in a cargo project directory
                show_ascii_puppy();
                println!("{}", Red.bold().paint("Not in a Cargo project directory!"));
                println!("{}", Yellow.paint("cargo-pup is an architectural linting tool for Rust projects."));
                println!("It needs to be run from a directory containing a Cargo.toml file.");
                println!("\nTo use cargo-pup:");
                println!("  1. Navigate to a Rust project directory");
                println!("  2. Run {}", Green.paint("cargo pup generate-config"));
                println!("  3. Edit the generated pup.yaml file");
                println!("  4. Run {}", Green.paint("cargo pup"));
                exit(-1)
            }
        }
    }

    let process_result = process(env::args());

    if let Err(code) = process_result {
        exit(code.0);
    }
}

fn process<I>(args: I) -> Result<(), CommandExitStatus>
where
    I: Iterator<Item = String>,
{
    // Parse arguments to get pup command and cargo args
    let pup_args = PupArgs::parse(args);

    // Store command for later use
    let command = pup_args.command.clone();

    // Check if we're generating config and the file already exists
    if command == cli::PupCommand::GenerateConfig {
        // Check for any existing generated config files
        let entries = std::fs::read_dir(".").expect("Failed to read current directory");
        let existing_configs: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                if let Some(name) = entry.file_name().to_str() {
                    name.starts_with("pup.generated.") && name.ends_with(".yaml")
                } else {
                    false
                }
            })
            .collect();
        
        if !existing_configs.is_empty() {
            println!("Error: Generated config files already exist:");
            for entry in existing_configs {
                println!("  - {}", entry.file_name().to_str().unwrap());
            }
            println!("Remove these files if you want to regenerate the configuration.");
            return Err(CommandExitStatus(1));
        }
    }

    // Create configuration to pass through to pup-driver
    let pup_cli = PupCli {
        command: pup_args.command,
    };

    // Convert args to string for environment
    let cli_args = pup_cli.to_env_str();

    // Format cargo args for environment
    let cargo_args_str = pup_args.cargo_args.join("__PUP_ARG_SEP__");

    // Build the cargo command
    let mut cmd = Command::new("cargo");

    // Set up environment variables
    cmd.env("RUSTC_WORKSPACE_WRAPPER", get_pup_path())
        .env("PUP_CLI_ARGS", cli_args)
        .env("PUP_CARGO_ARGS", cargo_args_str)
        .arg("check")
        .arg("--target-dir")
        .arg(".pup");

    // Add cargo args
    cmd.args(&pup_args.cargo_args);

    // Run cargo with our wrapper
    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    // If we just ran generate-config and it succeeded, check for generated files
    if exit_status.success() && command == cli::PupCommand::GenerateConfig {
        // Look for generated config files
        let entries = std::fs::read_dir(".").expect("Failed to read current directory");
        let generated_configs: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                if let Some(name) = entry.file_name().to_str() {
                    name.starts_with("pup.generated.") && name.ends_with(".yaml")
                } else {
                    false
                }
            })
            .collect();
        
        // If there's exactly one generated file and pup.yaml doesn't exist, rename it
        if generated_configs.len() == 1 && !Path::exists(Path::new("./pup.yaml")) {
            let generated_path = generated_configs[0].path();
            if let Err(e) = std::fs::rename(&generated_path, "pup.yaml") {
                println!("Warning: Failed to rename generated config to pup.yaml: {}", e);
            } else {
                println!("Created pup.yaml from {}", generated_path.file_name().unwrap().to_string_lossy());
            }
        }
    }

    if exit_status.success() {
        Ok(())
    } else {
        Err(CommandExitStatus(exit_status.code().unwrap_or(-1)))
    }
}

fn get_pup_path() -> String {
    env::current_exe()
        .expect("current executable path invalid")
        .to_str()
        .unwrap()
        .to_string()
}

fn run_pup_driver(toolchain: &str) -> Result<(), CommandExitStatus> {
    let args: Vec<String> = env::args().collect();

    // Check if we're processing a rustc wrapper call
    let rustc_args = if args.len() > 1 && args[1].ends_with("rustc") {
        args[2..].to_vec()
    } else {
        args[1..].to_vec()
    };

    // Build path to pup-driver
    let mut pup_driver_path = env::current_exe()
        .expect("current executable path invalid")
        .with_file_name("pup-driver");
    if cfg!(windows) {
        pup_driver_path.set_extension("exe");
    }

    // Find rustup
    let rustup = which::which("rustup")
        .expect("couldn't find rustup")
        .to_str()
        .unwrap()
        .to_string();

    // Install the toolchain if needed
    if let Err(e) = rustup_toolchain::install(toolchain) {
        eprintln!("Failed to install toolchain: {}", e);
        return Err(CommandExitStatus(-1));
    }

    // Compose our arguments for rustup run
    let mut final_args = vec![
        "run".to_string(),
        toolchain.to_string(),
        pup_driver_path.to_str().unwrap().to_string(),
    ];

    // Add all rustc arguments
    final_args.extend(rustc_args);

    // Run pup-driver through rustup
    let mut cmd = Command::new(rustup);
    cmd.args(&final_args);

    let exit_status = cmd
        .spawn()
        .expect("could not run pup-driver")
        .wait()
        .expect("failed to wait for pup-driver?");

    if exit_status.success() {
        Ok(())
    } else {
        Err(CommandExitStatus(exit_status.code().unwrap_or(-1)))
    }
}

fn get_toolchain() -> String {
    // We want to run with the same toolchain we were built with. This deals
    // with the dynamic-linking-against-librustc_driver piece, but _will_ add that toolchain
    // to the user's local rustup installs.
    let toolchain_config = include_str!("../rust-toolchain.toml");
    let toml = toml::from_str::<toml::Value>(toolchain_config).unwrap();

    // Work out which toolchain we want to run against
    toml.get("toolchain")
        .unwrap()
        .get("channel")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

#[must_use]
pub fn help_message() -> String {
    format!("
{title}: Checks your architecture against your architecture lint file.

{usage_label}:
    cargo pup [COMMAND] [OPTIONS] [--] [CARGO_ARGS...]

{commands_label}:
    {check}            Run architectural lints (default)
    {print_modules}    Print all modules and applicable lints
    {print_traits}     Print all traits
    {generate_config}  Generates an initial pup.yaml for your project.

{options_label}:
    -h, --help             Print this message
    -V, --version          Print version info and exit

Any additional arguments will be passed directly to cargo:
    --features=FEATURES    Cargo features to enable
    --manifest-path=PATH   Path to Cargo.toml

{note} to allow or deny lints from your code, e.g.:
    #[allow(pup::some_lint)]
",
        title = Style::new().bold().paint("Pretty Useful Pup"),
        usage_label = Blue.bold().paint("Usage"),
        commands_label = Blue.bold().paint("Commands"),
        check = Green.paint("check"),
        print_modules = Green.paint("print-modules"),
        print_traits = Green.paint("print-traits"),
        generate_config = Green.paint("generate-config"),
        options_label = Blue.bold().paint("Options"),
        note = Yellow.paint("You can use tool lints"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;
    
    /// Tests for validate_project function
    mod validate_project_tests {
        use super::*;
        
        fn setup_test_directory() -> TempDir {
            TempDir::new().expect("Failed to create temp directory")
        }
        
        #[test]
        fn test_configured_pup_project() {
            let temp_dir = setup_test_directory();
            let temp_path = temp_dir.path();
            
            // Create Cargo.toml and pup.yaml files
            fs::write(temp_path.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n")
                .expect("Failed to write Cargo.toml");
            fs::write(temp_path.join("pup.yaml"), "# Test pup.yaml\n")
                .expect("Failed to write pup.yaml");
            
            // Change to the temporary directory
            let original_dir = env::current_dir().expect("Failed to get current dir");
            env::set_current_dir(&temp_path).expect("Failed to change directory");
            
            // Run the validation
            let result = validate_project();
            
            // Change back to original directory
            env::set_current_dir(original_dir).expect("Failed to change back to original directory");
            
            assert_eq!(result, ProjectType::ConfiguredPupProject);
        }
        
        #[test]
        fn test_rust_project_without_pup() {
            let temp_dir = setup_test_directory();
            let temp_path = temp_dir.path();
            
            // Create only Cargo.toml file
            fs::write(temp_path.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n")
                .expect("Failed to write Cargo.toml");
            
            // Change to the temporary directory
            let original_dir = env::current_dir().expect("Failed to get current dir");
            env::set_current_dir(&temp_path).expect("Failed to change directory");
            
            // Run the validation
            let result = validate_project();
            
            // Change back to original directory
            env::set_current_dir(original_dir).expect("Failed to change back to original directory");
            
            assert_eq!(result, ProjectType::RustProject);
        }
        
        #[test]
        fn test_other_directory() {
            let temp_dir = setup_test_directory();
            let temp_path = temp_dir.path();
            
            // Empty directory - no files
            
            // Change to the temporary directory
            let original_dir = env::current_dir().expect("Failed to get current dir");
            env::set_current_dir(&temp_path).expect("Failed to change directory");
            
            // Run the validation
            let result = validate_project();
            
            // Change back to original directory
            env::set_current_dir(original_dir).expect("Failed to change back to original directory");
            
            assert_eq!(result, ProjectType::OtherDirectory);
        }
    }
}
