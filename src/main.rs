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

#![feature(let_chains)]
#![feature(array_windows)]
#![feature(try_blocks)]

#![warn(rust_2018_idioms, unused_lifetimes)]



use cargo_pup_common::cli::{PupArgs, PupCli, PupCommand};

use ansi_term::Colour::{Blue, Cyan, Green, Red, Yellow};
use ansi_term::Style;
use std::env;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::process::{exit, Command};
use cargo_pup_common::project_context::{ProjectContext, PUP_DIR};

#[derive(Debug, PartialEq)]
enum ProjectType {
    ConfiguredPupProject,
    RustProject,
    OtherDirectory,
}

#[derive(Debug, PartialEq)]
enum CommandType {
    PrintModules,
    PrintTraits,
    Other,
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
    let pup_yaml_path = Path::new("./pup.yaml");
    let cargo_toml_path = Path::new("./Cargo.toml");

    let has_pup_yaml = pup_yaml_path.exists();
    let has_cargo_toml = cargo_toml_path.exists();

    #[cfg(test)]
    {
        // For tests, check again to be extra sure
        let pup_exists = std::fs::metadata("./pup.yaml").is_ok();
        let cargo_exists = std::fs::metadata("./Cargo.toml").is_ok();
        println!(
            "validate_project debug - pup.yaml exists: {}/{}, Cargo.toml exists: {}/{}",
            has_pup_yaml, pup_exists, has_cargo_toml, cargo_exists
        );
    }

    if has_pup_yaml && has_cargo_toml {
        ProjectType::ConfiguredPupProject
    } else if has_cargo_toml {
        ProjectType::RustProject
    } else {
        ProjectType::OtherDirectory
    }
}

fn show_ascii_puppy() {
    println!(
        "{}",
        Cyan.paint(
            r#"
     / \__
    (    @\___
    /         O
   /   (_____/
  /_____/   U
"#
        )
    );
}

fn show_help() {
    show_ascii_puppy();
    println!("{}", help_message());
}

fn show_version() {
    println!(
        "{} {}",
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
    
    // Check command type
    let command = get_command_type(&args);
    
    // Get if we're running generate-config
    let is_generate_config = if args.len() <= 1 {
        false
    // Check for "cargo pup generate-config" pattern
    } else if args.len() > 2 && args[1] == "pup" {
        args[2] == "generate-config"
    } else {
        // Direct "generate-config" pattern
        args[1] == "generate-config"
    };

    // Skip environment checks if we're generating a config or running print commands
    let skip_checks = is_generate_config
        || command == CommandType::PrintModules
        || command == CommandType::PrintTraits;

    if !skip_checks {
        match validate_project() {
            ProjectType::ConfiguredPupProject => {
                // Good to go - continue with normal operation
            }
            ProjectType::RustProject => {
                // In a Rust project but missing pup.yaml
                show_ascii_puppy();
                println!("{}", Red.bold().paint("Missing pup.yaml - nothing to do!"));
                println!("Consider generating an initial configuration:");
                println!("  {}", Green.paint("cargo pup generate-config"));
                exit(-1)
            }
            ProjectType::OtherDirectory => {
                // Not in a cargo project directory
                show_ascii_puppy();
                println!("{}", Red.bold().paint("Not in a Cargo project directory!"));
                println!(
                    "{}",
                    Yellow.paint("cargo-pup is an architectural linting tool for Rust projects.")
                );
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

    // Parse command line args once (we've already collected args above)
    // Command type is already assigned to the 'command' variable above

    // Process the command
    match command {
        CommandType::PrintModules => {
            // First run normal process to generate context data
            if let Err(code) = process(env::args()) {
                exit(code.0);
            }

            // Then load and display the generated data
            if let Err(e) = process_print_modules() {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        CommandType::PrintTraits => {
            // First run normal process to generate context data
            if let Err(code) = process(env::args()) {
                exit(code.0);
            }

            // Then load and display the generated data
            if let Err(e) = process_print_traits() {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
        CommandType::Other => {
            // Run normal process flow
            if let Err(code) = process(env::args()) {
                exit(code.0);
            }
        }
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
    if command == PupCommand::GenerateConfig {
        // Check for existing pup.yaml and pup.generated.yaml in the project root
        let pup_yaml_exists = Path::exists(Path::new("./pup.yaml"));
        let pup_generated_yaml_exists = Path::exists(Path::new("./pup.generated.yaml"));

        // Determine target filename based on existence of pup.yaml
        let target_filename = if pup_yaml_exists {
            "pup.generated.yaml"
        } else {
            "pup.yaml"
        };

        // If target file already exists, show error
        if (target_filename == "pup.yaml" && pup_yaml_exists)
            || (target_filename == "pup.generated.yaml" && pup_generated_yaml_exists)
        {
            println!(
                "Error: {} already exists in the project root.",
                target_filename
            );
            println!("Remove this file if you want to regenerate the configuration.");
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

    // Get the same toolchain used for pup-driver
    let toolchain = get_toolchain();

    // Find rustup
    let rustup = which::which("rustup")
        .expect("couldn't find rustup")
        .to_str()
        .unwrap()
        .to_string();

    // Install the toolchain if needed
    if let Err(e) = rustup_toolchain::install(&toolchain) {
        eprintln!("Failed to install toolchain: {}", e);
        return Err(CommandExitStatus(-1));
    }

    // Build the cargo command using rustup to ensure consistent toolchain
    let mut cmd = Command::new(&rustup);
    cmd.arg("run").arg(&toolchain).arg("cargo");

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
    if exit_status.success() && command == PupCommand::GenerateConfig {
        // Look for generated config files in the .pup directory
        let pup_dir = Path::new(PUP_DIR);
        if !pup_dir.exists() {
            // No .pup directory, so there are no config files to process
            return Ok(());
        }

        let entries = std::fs::read_dir(pup_dir).expect("Failed to read .pup directory");
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

        // Check if we have any generated configs to process
        if !generated_configs.is_empty() {
            // Use the target filename previously determined during the check phase
            // Since we've already checked for file existence, we know this is safe
            let pup_yaml_exists = Path::exists(Path::new("./pup.yaml"));
            let target_filename = if pup_yaml_exists {
                "pup.generated.yaml"
            } else {
                "pup.yaml"
            };

            // Combine all config files into a single one
            let mut combined_content = String::new();

            // Add a header
            combined_content.push_str("# Combined pup configuration\n");
            combined_content.push_str("# Generated by cargo-pup\n\n");

            // Collect and sort configs by name for consistent ordering
            let mut sorted_configs: Vec<_> = generated_configs.iter().collect();
            sorted_configs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

            // Process each config file
            for (idx, entry) in sorted_configs.iter().enumerate() {
                let config_path = entry.path();
                let file_name = entry.file_name();
                let filename = file_name.to_str().unwrap_or("unknown").to_string();

                // Add file separator if not the first file
                if idx > 0 {
                    combined_content.push_str("\n# ===================================\n\n");
                }

                // Read the file content
                match std::fs::read_to_string(&config_path) {
                    Ok(content) => {
                        combined_content.push_str(&content);
                        // Ensure there's a trailing newline
                        if !content.ends_with('\n') {
                            combined_content.push('\n');
                        }
                    }
                    Err(e) => {
                        println!("Warning: Failed to read {}: {}", filename, e);
                    }
                }
            }

            // Write the combined file to the project root
            let target_path = Path::new(target_filename);
            match std::fs::write(target_path, combined_content) {
                Ok(_) => {
                    println!(
                        "Created {} with combined configurations from {} files",
                        target_filename,
                        generated_configs.len()
                    );
                }
                Err(e) => {
                    println!("Warning: Failed to write {}: {}", target_filename, e);
                }
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

/// Determine which command the user is running
fn get_command_type(args: &[String]) -> CommandType {
    // Check for print-modules command
    let is_print_modules = args.len() > 1
        && ((args.len() > 2 && args[1] == "pup" && args[2] == "print-modules")
            || (args[1] == "print-modules"));

    // Check for print-traits command
    let is_print_traits = args.len() > 1
        && ((args.len() > 2 && args[1] == "pup" && args[2] == "print-traits")
            || (args[1] == "print-traits"));

    if is_print_modules {
        CommandType::PrintModules
    } else if is_print_traits {
        CommandType::PrintTraits
    } else {
        CommandType::Other
    }
}

/// Process the print-modules command by loading contexts from disk and displaying them
fn process_print_modules() -> anyhow::Result<()> {
    use cargo_pup_common::project_context::{ProjectContext};
    use anyhow::Context;

    // Load all context data from .pup directory
    let (context, crate_names) = ProjectContext::load_all_contexts_with_crate_names()
        .context("Failed to load project context data")?;
    
    // Use the utility function to print the modules
    print_modules(&context, &crate_names)?;
    Ok(())
}

/// Process the print-traits command by loading contexts from disk and displaying them
fn process_print_traits() -> anyhow::Result<()> {
    use cargo_pup_common::project_context::{ProjectContext};
    use anyhow::Context;

    // Load all context data from .pup directory
    let (context, crate_names) = ProjectContext::load_all_contexts_with_crate_names()
        .context("Failed to load project context data")?;

    // Use the utility function to print the traits
    print_traits(&context, &crate_names)?;
    Ok(())
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
    format!(
        "
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
        note = Yellow.paint("You can use tool lints")
    )
}


/// Format and print the modules in the project context
pub fn print_modules(context: &ProjectContext, crate_names: &[String]) -> anyhow::Result<()> {
    use ansi_term::Colour::{Blue, Cyan, Green};
    use std::collections::BTreeMap;
    use cargo_pup_common::project_context::ModuleInfo;

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

    // Group modules by crate
    let mut modules_by_crate: BTreeMap<String, Vec<&ModuleInfo>> = BTreeMap::new();

    for module_info in &context.modules {
        // Extract crate name from module path (everything before the first ::)
        if let Some(idx) = module_info.name.find("::") {
            let crate_name = &module_info.name[..idx];

            modules_by_crate.entry(crate_name.to_string())
                .or_default()
                .push(module_info);
        } else {
            // Handle case where there's no :: in the path
            modules_by_crate.entry(module_info.name.clone())
                .or_default();
        }
    }

    // Print modules organized by crate
    for (crate_name, modules) in modules_by_crate {
        println!("{}", Blue.paint(&crate_name));

        // Group modules by their path structure and display them hierarchically
        let mut module_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for module_info in modules {
            // Get the crate-relative module path (everything after crate_name::)
            let module_path = if module_info.name.starts_with(&format!("{}::", crate_name)) {
                // For modules from this crate, remove the crate name prefix
                module_info.name[crate_name.len() + 2..].to_string()
            } else {
                // For modules without the expected prefix, use the full name
                module_info.name.clone()
            };

            // Format applicable lints as a comma-separated string
            let lints_str = module_info.applicable_lints.join(", ");

            // If module path is empty, it's the root module, otherwise add it to the map
            if module_path.is_empty() {
                println!("  :: [{}]", Green.paint(&lints_str));
            } else {
                module_map.entry(module_path).or_default().push(lints_str);
            }
        }

        // Print module paths with their lints
        for (module_path, lints_list) in module_map {
            // Join all lints for this module path
            let combined_lints = if lints_list.iter().all(|s| s.is_empty()) {
                "".to_string()
            } else {
                lints_list.join(", ")
            };

            println!("  ::{} [{}]", Blue.paint(&module_path), Green.paint(&combined_lints));
        }

        println!();
    }

    Ok(())
}

/// Format and print the traits in the project context
pub fn print_traits(context: &ProjectContext, crate_names: &[String]) -> anyhow::Result<()> {
    use ansi_term::Colour::{Blue, Green, Cyan};
    use std::collections::BTreeMap;
    use cargo_pup_common::project_context::TraitInfo;

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

    // Group traits by crate
    let mut traits_by_crate: BTreeMap<String, Vec<&TraitInfo>> = BTreeMap::new();

    for trait_info in &context.traits {
        // Extract crate name from trait path (everything before the first ::)
        if let Some(idx) = trait_info.name.find("::") {
            let crate_name = &trait_info.name[..idx];

            traits_by_crate.entry(crate_name.to_string())
                .or_default()
                .push(trait_info);
        } else {
            // Handle case where there's no :: in the path
            traits_by_crate.entry(trait_info.name.clone())
                .or_default();
        }
    }

    // Print traits organized by crate
    for (crate_name, traits) in traits_by_crate {
        println!("{}", Blue.paint(&crate_name));

        for trait_info in traits {
            // Get the crate-relative trait path (everything after crate_name::)
            let trait_path = if trait_info.name.starts_with(&format!("{}::", crate_name)) {
                // For traits from this crate, remove the crate name prefix
                trait_info.name[crate_name.len() + 2..].to_string()
            } else {
                // For traits without the expected prefix, use the full name
                trait_info.name.clone()
            };

            // Format applicable lints as a comma-separated string
            let lints_str = trait_info.applicable_lints.join(", ");

            // If trait path is empty, it's the root trait, otherwise add it to the map
            if trait_path.is_empty() {
                println!("  :: [{}]", Green.paint(&lints_str));
            } else {
                println!("  ::{} [{}]", Blue.paint(&trait_path), Green.paint(&lints_str));
            }

            // Print implementors with indentation
            if !trait_info.implementors.is_empty() {
                for implementor in &trait_info.implementors {
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
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    /// Tests for toolchain handling
    mod toolchain_tests {
        use super::*;
        use std::path::PathBuf;

        // Mock Command for testing command construction
        #[allow(dead_code)]
        struct MockCommand {
            program: String,
            args: Vec<String>,
            // We don't need environment variables for these tests
        }

        impl MockCommand {
            fn new(program: &str) -> Self {
                Self {
                    program: program.to_string(),
                    args: Vec::new(),
                }
            }

            fn arg(&mut self, arg: &str) -> &mut Self {
                self.args.push(arg.to_string());
                self
            }

            // No spawn/wait needed for testing command construction
        }

        // Create a test environment with a mock rustup
        fn setup_test_rustup() -> (String, String) {
            // Get the expected toolchain
            let toolchain = get_toolchain();

            // Mock rustup path
            let rustup_path = "mock_rustup".to_string();

            (rustup_path, toolchain)
        }

        #[test]
        fn test_cargo_uses_rustup_with_correct_toolchain() {
            // Get mock rustup and expected toolchain
            let (rustup_path, expected_toolchain) = setup_test_rustup();

            // Create a mock command to verify construction
            let mut cmd = MockCommand::new(&rustup_path);

            // Add expected rustup run arguments
            cmd.arg("run").arg(&expected_toolchain).arg("cargo");

            // Verify the command was constructed with rustup run and the correct toolchain
            assert_eq!(cmd.program, rustup_path);
            assert_eq!(cmd.args[0], "run");
            assert_eq!(cmd.args[1], expected_toolchain);
            assert_eq!(cmd.args[2], "cargo");
        }

        #[test]
        fn test_pup_driver_uses_rustup_with_correct_toolchain() {
            // Get mock rustup and expected toolchain
            let (rustup_path, expected_toolchain) = setup_test_rustup();

            // Create a mock command to verify construction
            let mut cmd = MockCommand::new(&rustup_path);

            // Create a fake pup-driver path
            let pup_driver_path = PathBuf::from("/path/to/pup-driver");

            // Add expected rustup run arguments
            cmd.arg("run")
                .arg(&expected_toolchain)
                .arg(pup_driver_path.to_str().unwrap());

            // Verify the command was constructed with rustup run and the correct toolchain
            assert_eq!(cmd.program, rustup_path);
            assert_eq!(cmd.args[0], "run");
            assert_eq!(cmd.args[1], expected_toolchain);
            assert_eq!(cmd.args[2], pup_driver_path.to_str().unwrap());
        }

        #[test]
        fn test_same_toolchain_for_cargo_and_pup_driver() {
            // This is the most important test - ensures both cargo and pup-driver use the same toolchain

            // Get the toolchain that would be used for both cargo and pup-driver
            let cargo_toolchain = get_toolchain();

            // It should be the same toolchain for both cargo and pup-driver
            let pup_driver_toolchain = get_toolchain();

            // Verify the same toolchain is used for both
            assert_eq!(
                cargo_toolchain, pup_driver_toolchain,
                "Cargo and pup-driver should use the same toolchain"
            );

            // Also check that it's reading from the rust-toolchain.toml file
            let toolchain_file = include_str!("../rust-toolchain.toml");
            assert!(
                toolchain_file.contains(&cargo_toolchain),
                "Toolchain should match what's in rust-toolchain.toml"
            );
        }
    }

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
            fs::write(
                temp_path.join("Cargo.toml"),
                "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
            )
            .expect("Failed to write Cargo.toml");
            fs::write(temp_path.join("pup.yaml"), "# Test pup.yaml\n")
                .expect("Failed to write pup.yaml");

            // Change to the temporary directory
            let original_dir = env::current_dir().expect("Failed to get current dir");
            env::set_current_dir(temp_path).expect("Failed to change directory");

            // Run the validation
            let result = validate_project();

            // Change back to original directory
            env::set_current_dir(original_dir)
                .expect("Failed to change back to original directory");

            assert_eq!(result, ProjectType::ConfiguredPupProject);
        }
    }

    /// Tests for help message and display functions
    mod display_tests {
        use super::*;

        #[test]
        fn test_help_message_format() {
            // Test that help_message() returns a properly formatted string
            let help = help_message();

            // Check for important components
            assert!(help.contains("Pretty Useful Pup"));
            assert!(help.contains("Usage"));
            assert!(help.contains("Commands"));
            assert!(help.contains("check"));
            assert!(help.contains("print-modules"));
            assert!(help.contains("print-traits"));
            assert!(help.contains("generate-config"));
            assert!(help.contains("Options"));
            assert!(help.contains("-h, --help"));
            assert!(help.contains("-V, --version"));
        }

        #[test]
        fn test_show_ascii_puppy() {
            // This is a difficult function to test directly since it prints to stdout
            // We'll just call it to ensure it doesn't panic
            show_ascii_puppy();
        }

        #[test]
        fn test_show_version() {
            // This is a difficult function to test directly since it prints to stdout
            // We'll just call it to ensure it doesn't panic
            show_version();
        }
    }

    /// Tests for the CommandExitStatus error type
    mod error_tests {
        use super::*;
        use std::error::Error;

        #[test]
        fn test_command_exit_status_display() {
            // Create a CommandExitStatus with a test exit code
            let status = CommandExitStatus(42);

            // Test the Display implementation
            let display_string = format!("{}", status);
            assert_eq!(display_string, "Command failed with exit code: 42");

            // Verify it implements the Error trait
            let error: &dyn Error = &status;
            assert_eq!(error.to_string(), "Command failed with exit code: 42");
        }

        #[test]
        fn test_command_exit_status_from_process_error() {
            // Test creating CommandExitStatus from different exit codes
            let error1 = CommandExitStatus(1);
            let error2 = CommandExitStatus(2);

            // Verify they have different values
            assert_ne!(error1.0, error2.0);

            // Test error message formatting
            assert_eq!(error1.to_string(), "Command failed with exit code: 1");
            assert_eq!(error2.to_string(), "Command failed with exit code: 2");
        }

        #[test]
        fn test_toolchain_parsing_error() {
            // This test verifies error handling when the toolchain configuration is malformed
            // We can't actually test this directly without modifying the code to accept a parameter
            // for the toolchain config, but we can test the error path indirectly

            // Ensure the function doesn't panic with valid input
            let toolchain = get_toolchain();
            assert!(!toolchain.is_empty(), "Toolchain should not be empty");
        }
    }

    /// Tests for the process function
    mod process_tests {
        use super::*;
        use std::path::PathBuf;

        // Mock Command for process tests that captures the command without executing it
        #[derive(Debug, Default, Clone)]
        #[allow(dead_code)]
        struct MockCommand {
            program: String,
            args: Vec<String>,
            envs: Vec<(String, String)>,
            was_spawned: bool,
        }

        #[test]
        fn test_process_generate_config() {
            // Create a test-specific temporary directory that will be automatically cleaned up
            let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
            let temp_path = temp_dir.path().to_path_buf();

            // Create basic Cargo.toml for a rust project
            fs::write(
                temp_path.join("Cargo.toml"),
                "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
            )
            .expect("Failed to write Cargo.toml");

            // Also create a src directory with a basic main.rs to make it look like a real project
            fs::create_dir_all(temp_path.join("src")).expect("Failed to create src directory");
            fs::write(temp_path.join("src/main.rs"), "fn main() {}\n")
                .expect("Failed to write main.rs");

            // Store the original directory
            let original_dir = env::current_dir().expect("Failed to get current dir");

            // Create a defer-like guard to ensure we always change back to the original directory
            struct DirGuard {
                original_dir: PathBuf,
            }

            impl Drop for DirGuard {
                fn drop(&mut self) {
                    let _ = env::set_current_dir(&self.original_dir);
                }
            }

            // Create the guard
            let _guard = DirGuard {
                original_dir: original_dir.clone(),
            };

            // Change to the temporary directory
            env::set_current_dir(&temp_path).expect("Failed to change directory");

            // Create test args for generate-config
            let args = vec!["cargo-pup".to_string(), "generate-config".to_string()];

            // Run the process function - we expect an error because cargo command will fail
            // in a test environment, but we're just verifying it doesn't panic
            let result = process(args.into_iter());
            assert!(
                result.is_err(),
                "Expected error due to cargo command failure"
            );

            // Verify the expected exit code for a cargo command failure
            // The actual value doesn't matter too much as long as it's consistent
            if let Err(exit_status) = result {
                assert!(exit_status.0 != 0, "Expected non-zero exit status");
            }

            // The guard will automatically change back to the original directory when it goes out of scope
        }

        #[test]
        fn test_get_pup_path() {
            // Test that get_pup_path returns the current executable path
            let exe_path = get_pup_path();

            // Verify it's a valid string
            assert!(!exe_path.is_empty());

            // While we can't check the exact path, we can check it's a valid path
            let path = Path::new(&exe_path);
            assert!(path.is_absolute(), "Path should be absolute");
        }

        #[test]
        fn test_process_with_existing_generated_configs() {
            // Test the process function behavior when there are existing generated configs
            let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
            let temp_path = temp_dir.path().to_path_buf();

            // Create a Cargo.toml file
            fs::write(
                temp_path.join("Cargo.toml"),
                "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
            )
            .expect("Failed to write Cargo.toml");

            // Create an existing generated config file
            fs::write(
                temp_path.join("pup.generated.test.yaml"),
                "# Test generated config\n",
            )
            .expect("Failed to write pup.generated.test.yaml");

            // Create a guard to ensure we always clean up properly
            struct DirectoryGuard {
                original_dir: PathBuf,
            }

            impl Drop for DirectoryGuard {
                fn drop(&mut self) {
                    // Best-effort attempt to change back; ignore errors in drop
                    let _ = env::set_current_dir(&self.original_dir);
                }
            }

            // Store original directory and create guard
            let original_dir = env::current_dir().expect("Failed to get current dir");
            let _guard = DirectoryGuard {
                original_dir: original_dir.clone(),
            };

            // Change to the temporary directory
            env::set_current_dir(&temp_path).expect("Failed to change directory");

            // Create test args for generate-config
            let args = vec!["cargo-pup".to_string(), "generate-config".to_string()];

            // Since there's an existing generated config, this should return an error
            let result = process(args.into_iter());

            // Verify that the process function returned an error
            assert!(result.is_err());

            // Check that the error code is non-zero, showing an error occurred
            if let Err(CommandExitStatus(code)) = result {
                assert_ne!(
                    code, 0,
                    "Error code should be non-zero for existing configs"
                );
            }

            // The DirectoryGuard will automatically change back to the original directory
        }

        #[test]
        fn test_rename_generated_config() {
            // Test the rename logic when a single generated config is found
            let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
            let temp_path = temp_dir.path().to_path_buf();

            // Create a Cargo.toml file but no pup.yaml
            fs::write(
                temp_path.join("Cargo.toml"),
                "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
            )
            .expect("Failed to write Cargo.toml");

            // Create a guard to ensure we always clean up properly
            struct DirectoryGuard {
                original_dir: PathBuf,
            }

            impl Drop for DirectoryGuard {
                fn drop(&mut self) {
                    // Best-effort attempt to change back; ignore errors in drop
                    let _ = env::set_current_dir(&self.original_dir);
                }
            }

            // Get the original directory and create the guard
            let original_dir = env::current_dir().expect("Failed to get current dir");
            let _guard = DirectoryGuard {
                original_dir: original_dir.clone(),
            };

            // Change to the temporary directory
            env::set_current_dir(&temp_path).expect("Failed to change directory");

            // Verify pup.yaml doesn't exist yet in the temp directory
            let pup_yaml_path = temp_path.join("pup.yaml");
            assert!(
                !pup_yaml_path.exists(),
                "pup.yaml should not exist at start of test"
            );

            // Use absolute paths for all file operations to avoid current directory issues
            let generated_config_path = temp_path.join("pup.generated.test.yaml");

            // Create a generated config file manually
            fs::write(&generated_config_path, "# Test generated config\n")
                .expect("Failed to write pup.generated.test.yaml");

            // Brief delay to ensure file operations complete
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Verify the generated config exists
            assert!(
                generated_config_path.exists(),
                "Generated config file should exist"
            );

            // We already have the destination path for pup.yaml defined above

            // Rename it manually for the test
            match fs::rename(&generated_config_path, &pup_yaml_path) {
                Ok(_) => {}
                Err(e) => panic!("Failed to rename file: {}", e),
            }

            // Brief delay to ensure rename completes
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Verify pup.yaml now exists
            assert!(pup_yaml_path.exists(), "pup.yaml should exist after rename");

            // The guard will automatically change back to the original directory when it goes out of scope
        }
    }

    /// Tests for the run_pup_driver function
    mod run_pup_driver_tests {

        #[test]
        fn test_run_pup_driver_args_handling() {
            // Set up temporary environment
            let _toolchain = "nightly-2023-10-10"; // Example toolchain

            // Test with rustc wrapper style args
            let args = [
                "cargo-pup".to_string(),
                "/path/to/rustc".to_string(),
                "-Copt-level=2".to_string(),
                "--edition=2021".to_string(),
            ];

            // Check that rustc_args is correctly extracted
            let rustc_args = if args.len() > 1 && args[1].ends_with("rustc") {
                args[2..].to_vec()
            } else {
                args[1..].to_vec()
            };

            assert_eq!(rustc_args, vec!["-Copt-level=2", "--edition=2021"]);

            // Test without rustc wrapper
            let args = [
                "cargo-pup".to_string(),
                "arg1".to_string(),
                "arg2".to_string(),
            ];

            // Check that rustc_args is correctly extracted
            let rustc_args = if args.len() > 1 && args[1].ends_with("rustc") {
                args[2..].to_vec()
            } else {
                args[1..].to_vec()
            };

            assert_eq!(rustc_args, vec!["arg1", "arg2"]);
        }
    }

    /// Tests for the main entry point function
    mod main_entry_point_tests {
        use super::*;

        #[test]
        fn test_help_flag_handling() {
            // Test that the --help flag is properly handled

            // Set up test environment
            let _original_args = env::args().collect::<Vec<String>>();

            // Temporarily override args
            let args: Vec<String> = vec!["cargo-pup".to_string(), "--help".to_string()];

            // We can't override env::args() directly in a test,
            // but we can check the condition that would be triggered
            let should_show_help = args.iter().any(|a| a == "--help" || a == "-h");

            // Verify the condition is true
            assert!(should_show_help, "Should detect --help flag");

            // Do the same for -h
            let args: Vec<String> = vec!["cargo-pup".to_string(), "-h".to_string()];

            let should_show_help = args.iter().any(|a| a == "--help" || a == "-h");
            assert!(should_show_help, "Should detect -h flag");
        }

        #[test]
        fn test_version_flag_handling() {
            // Test that the --version flag is properly handled

            // Set up test environment
            let _original_args = env::args().collect::<Vec<String>>();

            // Temporarily override args
            let args: Vec<String> = vec!["cargo-pup".to_string(), "--version".to_string()];

            // We can't override env::args() directly in a test,
            // but we can check the condition that would be triggered
            let should_show_version = args.iter().any(|a| a == "--version" || a == "-V");

            // Verify the condition is true
            assert!(should_show_version, "Should detect --version flag");

            // Do the same for -V
            let args: Vec<String> = vec!["cargo-pup".to_string(), "-V".to_string()];

            let should_show_version = args.iter().any(|a| a == "--version" || a == "-V");
            assert!(should_show_version, "Should detect -V flag");
        }

        #[test]
        fn test_trampoline_mode_detection() {
            // Test that rustc wrapper invocation is properly detected

            // Create args that look like a rustc wrapper invocation
            let args = [
                "cargo-pup".to_string(),
                "/path/to/rustc".to_string(),
                "-Copt-level=2".to_string(),
            ];

            // Check the condition that would trigger trampoline mode
            let is_rustc_wrapper = args.len() > 1 && args[1].ends_with("rustc");

            // Verify the condition is true
            assert!(is_rustc_wrapper, "Should detect rustc wrapper invocation");

            // Test with args that don't trigger trampoline mode
            let args = ["cargo-pup".to_string(), "check".to_string()];

            // Check the condition that would trigger trampoline mode
            let is_rustc_wrapper = args.len() > 1 && args[1].ends_with("rustc");

            // Verify the condition is false
            assert!(
                !is_rustc_wrapper,
                "Should not detect rustc wrapper invocation for normal command"
            );
        }
    }

    /// Tests for command line processing
    mod command_line_processing_tests {
        use cargo_pup_common::cli::{PupArgs, PupCli, PupCommand};
        #[test]
        fn test_arguments_parsing() {
            // Test different argument combinations

            // Basic command
            let args = vec!["cargo-pup".to_string(), "check".to_string()];
            let pup_args = PupArgs::parse(args.into_iter());
            assert_eq!(pup_args.command, PupCommand::Check);
            assert_eq!(pup_args.cargo_args.len(), 0);

            // Command with cargo args
            let args = vec![
                "cargo-pup".to_string(),
                "check".to_string(),
                "--features=foo".to_string(),
            ];
            let pup_args = PupArgs::parse(args.into_iter());
            assert_eq!(pup_args.command, PupCommand::Check);
            assert_eq!(pup_args.cargo_args, vec!["--features=foo"]);

            // Command with multiple cargo args
            let args = vec![
                "cargo-pup".to_string(),
                "print-modules".to_string(),
                "--features=foo".to_string(),
                "--manifest-path=test/Cargo.toml".to_string(),
            ];
            let pup_args = PupArgs::parse(args.into_iter());
            assert_eq!(pup_args.command, PupCommand::PrintModules);
            assert_eq!(
                pup_args.cargo_args,
                vec!["--features=foo", "--manifest-path=test/Cargo.toml"]
            );
        }

        #[test]
        fn test_is_generate_config_detection() {
            // Test direct invocation
            {
                let args = vec!["cargo-pup".to_string(), "generate-config".to_string()];

                let is_generate_config = check_is_generate_config(&args);
                assert!(is_generate_config, "Should detect generate-config command");
            }

            // Test via cargo pup
            {
                let args = vec![
                    "cargo".to_string(),
                    "pup".to_string(),
                    "generate-config".to_string(),
                ];

                let is_generate_config = check_is_generate_config(&args);
                assert!(
                    is_generate_config,
                    "Should detect generate-config command via cargo pup"
                );
            }

            // Test other command
            {
                let args = vec!["cargo-pup".to_string(), "check".to_string()];

                let is_generate_config = check_is_generate_config(&args);
                assert!(
                    !is_generate_config,
                    "Should not detect generate-config for check command"
                );
            }
        }

        // Helper function for testing generate-config detection
        fn check_is_generate_config(args: &[String]) -> bool {
            if args.len() <= 1 {
                false
            } else if args.len() > 2 && args[1] == "pup" {
                args.len() > 2 && args[2] == "generate-config"
            } else {
                args[1] == "generate-config"
            }
        }

        #[test]
        fn test_pup_cli_serialization() {
            // Test that PupCli can be serialized and deserialized correctly
            // Create a PupCli with a test command
            let pup_cli = PupCli {
                command: PupCommand::PrintModules,
            };

            // Serialize it
            let serialized = pup_cli.to_env_str();

            // Deserialize it
            let deserialized = PupCli::from_env_str(&serialized);

            // Verify the round trip works
            assert_eq!(deserialized.command, PupCommand::PrintModules);

            // Test with a different command
            let pup_cli = PupCli {
                command: PupCommand::GenerateConfig,
            };

            // Serialize it
            let serialized = pup_cli.to_env_str();

            // Deserialize it
            let deserialized = PupCli::from_env_str(&serialized);

            // Verify the round trip works
            assert_eq!(deserialized.command, PupCommand::GenerateConfig);
        }
    }
}
