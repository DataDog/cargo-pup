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

use clap::Parser;
use cli::PupCli;

use std::env;
use std::path::Path;
use std::process::{Command, exit};
use std::fmt;
use std::error::Error;

/// Simple error type that wraps a command exit code
#[derive(Debug)]
struct CommandExitStatus(i32);

impl fmt::Display for CommandExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command failed with exit code: {}", self.0)
    }
}

impl Error for CommandExitStatus {}

#[allow(dead_code)]
fn show_help() {
    println!("{}", help_message());
}

#[allow(dead_code)]
fn show_version() {
    println!("Golden Span Retriever version 0.1.0");
}

pub fn main() {
    // Check if we have a `pup.yaml` in the directory we're in
    if !Path::exists(Path::new("./pup.yaml")) {
        println!("Missing pup.yaml - nothing to do!");
        exit(-1)
    }

    let toolchain = get_toolchain();

    // Are we first-iteration, or, are we being called back by cargo?
    // If we're first iteration, we redirect cargo back to us.
    if env::var("PUP_TRAMPOLINE_MODE").is_ok() {
        // We're in the trampoline - we need to run cargo-pup
        // with the arguments we've got to do the build.
        if let Err(err) = run_pup_cmd(&toolchain) {
            exit(err.0);
        }
    } else {
        // We've not trampolined yet - we're in the initial invocation.
        // We need to trampoline through using `cargo build` with us as
        // the workspace wrapper.
        //
        // But first, let's check the command line arguments to see if we can
        // short circuit this.
        let cmd = PupCli::parse_from(env::args().skip(1));
        let cli_args = cmd.to_env_str();
        if let Err(err) = run_trampoline(&toolchain, &cli_args) {
            exit(err.0);
        }
    }
}

///
/// Generates our trampoline command. This trampolines straight
/// back to this executable, cargo-pup.
///
fn generate_trampoline_cmd(toolchain: &str, cli_args: &str) -> Command {
    // we want to invoke cargo via rutup
    let mut cmd = Command::new("rustup");
    let terminal_width = termize::dimensions().map_or(0, |(w, _)| w);

    // Construct a path back to ourselves
    let path = env::current_exe().expect("current executable path invalid");

    // But, we'll use RUSTC_WORKSPACE_WRAPPER, so that when the nested cargo runs, it kicks
    // the invocation back to us
    cmd.env("RUSTC_WORKSPACE_WRAPPER", path.to_str().unwrap())
        .env("PUP_TRAMPOLINE_MODE", "true")
        .env("PUP_TERMINAL_WIDTH", terminal_width.to_string())
        .env("PUP_CLI_ARGS", cli_args)
        .arg("run")
        .arg(toolchain)
        .arg("cargo")
        .arg("check")
        .arg("--target-dir")
        .arg(".pup");

    cmd
}

///
/// Trampolines back through cargo-pup using us as RUSTC_WORKSPACE_WRAPPER. This'll return to us with
/// the `rustc` invocation that cargo wants, which we can than wrap up and pass off to pup-driver.
///
fn run_trampoline(toolchain: &str, cli_args: &str) -> Result<(), CommandExitStatus> {
    let mut cmd = generate_trampoline_cmd(toolchain, cli_args);

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

///
/// The second time we come through, when we are being invoekd as the wrapper,
/// we call off with all of our arguments to pup-driver, using rustup to wrap
/// the invocation.
///
fn generate_pup_cmd(args: env::Args, toolchain: &String) -> anyhow::Result<Command> {
    // First, construct the executable path to pup-driver.
    let mut pup_driver_path = env::current_exe()
        .expect("current executable path invalid")
        .with_file_name("pup-driver");
    if cfg!(windows) {
        pup_driver_path.set_extension("exe");
    }

    // Locate rustup
    let which_rustup = which::which("rustup").unwrap();
    let rustup = which_rustup.to_str().unwrap();

    rustup_toolchain::install(toolchain)?;

    // Compose our arguments
    let mut final_args: Vec<String> = vec![
        "run".into(),
        toolchain.into(),
        pup_driver_path.to_str().unwrap().into(),
    ];
    for arg in args.skip(1) {
        let arg = arg.to_string();
        final_args.push(arg);
    }

    let mut cmd = Command::new(rustup);
    cmd.args(final_args);

    Ok(cmd)
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

///
/// Runs pup-driver. This is the bit that does the actual work of starting
/// the rustc compilation with all of the args we've been given by cargo.
///
/// This is launched once we've trampolined back through ourselves.
///
fn run_pup_cmd(toolchain: &String) -> Result<(), CommandExitStatus> {
    match generate_pup_cmd(env::args(), toolchain) {
        Ok(mut cmd) => {
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
        Err(_) => Err(CommandExitStatus(-1)),
    }
}

#[must_use]
pub fn help_message() -> &'static str {
    "
Pretty Useful Pup: Checks your architecture against your architecture lint file.

Usage:
    cargo pup [OPTIONS] [--] [ARGS...]

Options:
    -h, --help             Print this message
    -V, --version          Print version info and exit

You can use tool lints to allow or deny lints from your code, e.g.:
    #[allow(pup::some_lint)]
"
}
