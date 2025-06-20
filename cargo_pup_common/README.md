# cargo_pup_common

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/DataDog/cargo-pup/main/docs/pup_dark.png">
  <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/DataDog/cargo-pup/main/docs/pup_light.png">
  <img alt="cargo_pup logo" src="https://raw.githubusercontent.com/DataDog/cargo-pup/main/docs/pup_light.png" width="250">
</picture>

Common utilities and shared components for the [cargo-pup architectural linting](https://github.com/datadog/cargo-pup) tool.

This crate provides foundational types, CLI utilities, and project context management that are shared across the cargo-pup ecosystem.

## Features

- Project context serialization and management
- CLI argument parsing utilities
- Shared types and utilities used by other cargo-pup components

## Usage

This crate is primarily intended for internal use by other cargo-pup components. If you're looking to use cargo-pup in your project, see the main [cargo_pup](https://crates.io/crates/cargo_pup) crate.

For more information and examples, visit the [cargo-pup repository](https://github.com/datadog/cargo-pup).
