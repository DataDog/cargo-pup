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

use std::env;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::process::{Command, exit};

/// Simple error type that wraps a command exit code
#[derive(Debug)]
struct CommandExitStatus(i32);

impl fmt::Display for CommandExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command failed with exit code: {}", self.0)
    }
}

impl Error for CommandExitStatus {}

fn show_help() {
    println!("{}", help_message());
}

fn show_version() {
    println!("cargo-pup version 0.1.0");
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

    // Check if we have a `pup.yaml` in the directory we're in
    if !Path::exists(Path::new("./pup.yaml")) {
        println!("Missing pup.yaml - nothing to do!");
        exit(-1)
    }

    // Parse command and arguments
    // Normal invocation - process args and run cargo
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
pub fn help_message() -> &'static str {
    "
Pretty Useful Pup: Checks your architecture against your architecture lint file.

Usage:
    cargo pup [COMMAND] [OPTIONS] [--] [CARGO_ARGS...]

Commands:
    check         Run architectural lints (default)
    print-modules Print all modules and applicable lints
    print-traits  Print all traits

Options:
    -h, --help             Print this message
    -V, --version          Print version info and exit

Any additional arguments will be passed directly to cargo:
    --features=FEATURES    Cargo features to enable
    --manifest-path=PATH   Path to Cargo.toml

You can use tool lints to allow or deny lints from your code, e.g.:
    #[allow(pup::some_lint)]
"
}
