# cargo_pup_lint_config

Configuration and rule builder utilities for cargo-pup architectural linting.

This crate provides the programmatic builder interface for defining architectural lint rules. It offers type-safe, IDE-friendly APIs for creating complex architectural assertions about your Rust codebase.

## Features

- **Type-safe builder API** - Define lint rules with full IDE support and compile-time validation
- **Module-level linting** - Enforce rules about module structure, imports, and naming
- **Function-level linting** - Validate function signatures, return types, and implementations
- **Struct-level linting** - Assert constraints on struct definitions, visibility, and trait implementations
- **Integration testing support** - Run architectural assertions directly in your test suite

## Usage

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
cargo_pup_lint_config = "0.1.0"
```

## Example

```rust
use cargo_pup_lint_config::{LintBuilder, LintBuilderExt, ModuleLintExt, Severity};

#[test]
fn test_api_layer_isolation() {
    let mut builder = LintBuilder::new();
    
    builder.module_lint()
        .lint_named("api_no_direct_db_access")
        .matching(|m| m.module(".*::api::.*"))
        .with_severity(Severity::Error)
        .restrict_imports(None, Some(vec![".*::database::*".to_string()]))
        .build();
    
    builder.assert_lints(None).expect("API isolation rules should pass");
}
```

For more examples and documentation, visit the [cargo-pup repository](https://github.com/datadog/cargo-pup).
