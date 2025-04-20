# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands
- Build: `cargo build`
- Run: `cargo run`
- Check: `cargo check`
- Test all: `cargo test`
- Test single: `cargo test test_name`
- Validate test app: `cd test_app && scripts/validate-against-expected.sh`
- Generate config: `cargo run -- generate-config`

## Project Structure
- **cargo-pup**: Main entry point (src/main.rs) - cargo subcommand executed when users run `cargo pup`
- **pup-driver**: Rustc driver (src/pup_driver.rs) - interacts with rustc compiler API to apply lints

## Invocation Flow
1. User runs `cargo pup` (which executes cargo-pup)
2. cargo-pup parses arguments and sets up environment
3. cargo-pup launches `cargo check` with RUSTC_WORKSPACE_WRAPPER pointing to itself
4. cargo calls back into cargo-pup with rustc args ("trampoline mode")
5. cargo-pup executes pup-driver with rustc args
6. pup-driver loads lint configs, analyzes code using rustc API, and reports results

## Code Style
- Use nightly Rust (see rust-toolchain.toml for specific version)
- Format code with `rustfmt`
- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use anyhow for error wrapping, use custom error types where appropriate
- Modules should be structured similar to rustc/clippy
- Keep functions under 30 lines where possible
- Use proper trait implementations with clear contracts
- Write unit tests for new linting rules
- Use descriptive variable names and add documentation for public APIs