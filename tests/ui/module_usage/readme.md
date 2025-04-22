# Module Usage Lint UI Tests

This directory contains UI tests for the `module_usage` lint, which enforces constraints on module imports.

## Test Files

1. `module_usage_test.rs` - Tests for two rule types:
   - `DenyWildcard`: Prohibits wildcard imports (`use x::y::*`)
   - `Deny`: Prohibits specific modules from being imported

2. `allow_only_test.rs` - Tests for the third rule type:
   - `AllowOnly`: Only allows a specific set of modules to be imported

## Configuration Files

- `pup.yaml` - Main configuration used by the standard test (contains DenyWildcard and Deny rules)
- `allow_only.yaml` - Configuration for the AllowOnly rule test

## Running the Tests

To run the tests, use:

```bash
# Run all module_usage tests
TESTNAME=module_usage cargo test --test ui-test

# Run a specific test
TESTNAME=module_usage/module_usage_test.rs cargo test --test ui-test
TESTNAME=module_usage/allow_only_test.rs cargo test --test ui-test 
```

When modifying the test for the `AllowOnly` rule, remember to copy the configuration:

```bash
cp tests/ui/module_usage/allow_only.yaml tests/ui/module_usage/pup.yaml
```