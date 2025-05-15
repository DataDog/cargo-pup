# Integration Tests for pup.ron Configuration

This directory contains integration tests that verify the functionality of the new RON-based linting framework for cargo-pup.

## Test Overview

### pup_ron_test.rs

This test demonstrates how to programmatically build lint configurations using the `cargo_pup_lint_config` crate. It:

1. Creates a new `LintBuilder` instance
2. Adds several lint rules that approximate the ones defined in the YAML configuration
3. Writes the configuration to a RON file in the `.pup` directory
4. Reads the configuration back and verifies it was properly serialized

The test shows how the new API can be used to construct the same type of configuration that was previously defined in YAML format.

## Running the Tests

To run these integration tests:

```
cd test_app
cargo test
```

## Relationship to pup.yaml

This test is part of the migration from the YAML-based configuration to the new RON-based configuration format. The tests verify that we can programmatically create configurations that match what was previously defined in the YAML format.

The RON format provides better type checking and a more Rust-native configuration format, while the builder API makes it more convenient to construct these configurations in code. 