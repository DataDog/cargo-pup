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
use cli::{PupCli, PupCliCommands};
use lints::{ArchitectureLintRunner, Mode};

use crate::lints::{ArchitectureLintCollection, register_all_lints};
use rustc_session::{EarlyDiagCtxt, config::ErrorOutputType};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};
use utils::configuration_factory::setup_lints_yaml;

mod cli;
mod lints;
mod utils;

pub fn main() -> Result<()> {
    register_all_lints();

    let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&early_dcx);

    let mut orig_args: Vec<String> = env::args().collect();

    // Handle wrapper mode
    let wrapper_mode =
        orig_args.get(1).map(Path::new).and_then(Path::file_stem) == Some("rustc".as_ref());
    if wrapper_mode {
        orig_args.remove(1);
    }

    // Add `--sysroot` if missing
    if !orig_args.iter().any(|arg| arg == "--sysroot") {
        orig_args.extend(vec!["--sysroot".into(), find_sysroot()]);
    }

    // Load our configuration
    let binding = std::env::var("PUP_CLI_ARGS").expect("Missing PUP_CLI_ARGS");
    let cli_args = binding.as_str();
    let config = PupCli::from_env_str(cli_args);

    let mode = match config.command.unwrap_or(PupCliCommands::Check) {
        PupCliCommands::PrintModules => Mode::PrintModules,
        PupCliCommands::PrintTraits => Mode::PrintTraits,
        PupCliCommands::Check => Mode::Check,
    };

    // Suppress rust's own lint output
    // orig_args.extend(vec!["-A".into(), "warnings".into()]);

    // Log it, so we can work out what is going on
    log_invocation(&orig_args)?;

    // Forward all arguments to RunCompiler, including `"-"`
    let lint_rules = setup_lints_yaml()?;
    let lint_collection = ArchitectureLintCollection::new(lint_rules);
    let mut runner = ArchitectureLintRunner::new(mode, cli_args.into(), lint_collection);

    rustc_driver::run_compiler(&orig_args, &mut runner);

    // Print out our lints
    let results_text = runner.lint_results_text();
    if !results_text.is_empty() {
        eprintln!("{0}", runner.lint_results_text());
    }

    process::exit(0);
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    pub fn test_yaml_loading() -> Result<()> {
        register_all_lints();

        let lints = setup_lints_yaml()?;
        assert_eq!(lints.len(), 5);
        Ok(())
    }

    use crate::{
        lints::register_all_lints,
        utils::configuration_factory::{LintConfigurationFactory, setup_lints_yaml},
    };

    ///
    /// This project should have its own loadable pup.yaml
    ///
    #[test]
    pub fn load_own_configuration() {
        register_all_lints();

        LintConfigurationFactory::from_yaml(include_str!("../pup.yaml").to_string()).unwrap();
    }
}
