#![feature(rustc_private)]
#![feature(let_chains)]
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use anyhow::Result;

use crate::lints::{register_all_lints, ArchitectureLintCollection, ArchitectureLintRule};
use rustc_driver::RunCompiler;
use rustc_session::{config::ErrorOutputType, EarlyDiagCtxt};
use std::{env, path::Path, process, process::Command};
use utils::configuration_factory::LintConfigurationFactory;

struct DefaultCallbacks;
impl rustc_driver::Callbacks for DefaultCallbacks {}

mod example;
mod lints;
mod utils;

pub fn main() -> Result<()> {
    register_all_lints();

    let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&early_dcx);

    let mut orig_args: Vec<String> = env::args().collect();

    // Handle `--help` and `--version` early
    if orig_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        process::exit(0);
    }

    if orig_args
        .iter()
        .any(|arg| arg == "--version" || arg == "-V")
    {
        print_version();
        process::exit(0);
    }

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

    // Suppress rust's own lint output
    orig_args.extend(vec!["-A".into(), "warnings".into()]);

    // Forward all arguments to RunCompiler, including `"-"`
    let mut callbacks = ArchitectureLintCollection::new(setup_lints_yaml()?);
    RunCompiler::new(&orig_args, &mut callbacks).run();

    // Print out our lints
    eprintln!();
    eprintln!("{0}", callbacks.to_string());

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

fn print_help() {
    println!(
        "Pretty Useful Pup: Checks your architecture against your architecture lint file.

Usage:
    pup-driver [OPTIONS] INPUT

Options:
    -h, --help        Print this message
    -V, --version     Print version info and exit
    --rustc           Pass all arguments directly to rustc
    --sysroot PATH    Specify the sysroot directory

Example:
    pup-driver --sysroot /path/to/sysroot -- my_crate.rs
"
    );
}

fn print_version() {
    println!("Golden Span Retriever Driver version 0.1.0");
}

fn setup_lints_yaml() -> Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
    use std::fs;

    // Attempt to load configuration from `pup.yaml`
    let yaml_content = fs::read_to_string("pup.yaml")?;
    let lint_rules =
        LintConfigurationFactory::from_yaml(&yaml_content).map_err(anyhow::Error::msg)?;

    Ok(lint_rules)
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

    use crate::lints::register_all_lints;

    ///
    /// This project should have its own loadable pup.yaml
    ///
    #[test]
    pub fn load_own_configuration() {
        register_all_lints();

        LintConfigurationFactory::from_yaml(include_str!("../pup.yaml")).unwrap();
    }
}
