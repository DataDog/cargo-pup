# cargo_pup_lint_impl

Core lint implementations and rustc integration for cargo-pup architectural linting.

This crate contains the actual lint rule implementations and provides the rustc compiler integration that powers cargo-pup's architectural validation capabilities.

## Features

- **Rustc integration** - Deep integration with the Rust compiler's analysis phases
- **Module lint rules** - Enforce architectural boundaries and import restrictions
- **Function lint rules** - Validate function signatures, return types, and implementations  
- **Struct lint rules** - Assert constraints on struct definitions and trait implementations
- **Architecture lint framework** - Extensible framework for custom architectural rules

## Internal Crate

This crate is primarily intended for internal use by the cargo-pup toolchain. Most users should interact with cargo-pup through:

- The main [cargo-pup](https://crates.io/crates/cargo_pup) CLI tool
- The [cargo_pup_lint_config](https://crates.io/crates/cargo_pup_lint_config) builder API

## Architecture

This crate implements the core architectural linting logic that runs during Rust compilation, leveraging rustc's internal APIs to analyze code structure and enforce user-defined rules.

For more information and examples, visit the [cargo-pup repository](https://github.com/datadog/cargo-pup).
