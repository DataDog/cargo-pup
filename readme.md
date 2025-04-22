# `cargo pup`

**Pretty Useful Pup** (_pup_) lets you write assertions about your Rust project's architecture, letting you continuously
validate consistency both locally and in your CI pipelines. As projects grow and new contributors come on board inconsistency
begins to creep in, increasing the cognitive load for everyone working on the system.

Inspired by [ArchUnit](https://www.archunit.org/) and [ArchUnitNet](https://github.com/TNG/ArchUnitNET), it also 
introduces an exciting, fresh naming convention for architectural linting tools.

Check out the [Examples](#examples) to see what you can do!

## Usage

> [!NOTE]
> Long term, this should work as one of those classic `curl https://sh.cargopup.sh | sh` deployments. For now while we're private,
> this will have to do.

**Pretty Useful Pup** is installed as a [cargo](TODO) subcommand. This simply means that it needs to be in your `$PATH`, or optimally, in your `~/.cargo/bin` directory.

First up, make sure to install [rustup](https://rustup.rs/) to manage your local rust installs and provide the tooling required for Pretty Useful Pup, if you haven't already.

Next, run [install.sh](https://github.com/DataDog/cargo-pup/raw/refs/heads/main/scripts/install.sh). While this repository is private, you'll have to
download this manually!

If you want to make changes to the repository you can also `git clone` the whole thing, then run `install.sh` from within the clone to build and install
the local state.

## Examples

Cargo-pup uses cargo-pup to enforce it's own architecture.

### Enforce naming of structs implementing a trait

For particular traits you may want the implementors to follow a consistent naming scheme; here we ensure that all of cargo-pup's lint processors are named the same way, and marked `private`:

```yaml
architecture_lint_rules:
  type: trait_impl
  source_name: "lints::architecture_lint_rule::ArchitectureLintRule"
  name_must_match: ".*LintProcessor$"
  enforce_visibility: "Private"
  severity: Error

```

### Constrain module usage:

We can ensure that particular modules are not used in certain places. This is a useful strategy to enforce layering and avoiding mixing concerns - for instance, if we have a REST API, we probably don't want to use Database clients directly, prefering to go through an intermediate layer:

```yaml
deny_std_collections:
  type: module_usage
  name: "test_me_namespace_rule_new"
  modules:
    - "^test_app::public_rest_api$"
  rules:
    - type: Deny
      denied_modules:
        - "sqlx::*"
        - "diesel::*"
      severity: Warn

    # We can also block wildcard imports
    - type: DenyWildcard
      severity: Warn
```

### Enforce empty mod.rs files

This ensures that a `mod.rs` can contain only references to other modules and re-exports:

```yaml
empty_mods:
  type: empty_mod
  modules:
    - "lints"
    - "utils"
  severity: Warn
```

### Constraint language items allowed

Sometimes we may want to ensure that particular modules can only contain certain items, for instance, here we want to ensure our helpers module can only contain basic functions, enums, and so on:

```yaml
helpers_no_structs_or_traits:
  type: item_type
  modules:
    - "^pup_driver::lints::helpers$"
  denied_items:
    - struct  
    - trait 
  severity: Error
```

## Testing

### UI Tests

Cargo Pup includes UI tests to validate lint behavior. These tests follow the pattern used by Clippy and other Rust compiler components.

To run the UI tests:

```bash
cargo test --test ui-test
```

If you make changes to the lints that affect the expected output, you can update the .stderr files with:

```bash
BLESS=1 cargo test --test ui-test
```

#### How UI Tests Work

UI tests consist of:
1. `.rs` files containing code that triggers (or doesn't trigger) lints 
2. `.stderr` files containing the expected compiler output/diagnostics

Tests use special comments:
- `//@` comments configure test behavior
- `//~` comments mark expected diagnostic locations
- `//@ pup-config: |` comments define lint configurations for the test

You can find examples in the `tests/ui/function_length/` directory.

## Pretty Useful Pup Tenets

* **Not [clippy](https://github.com/rust-lang/rust-clippy)** - pup isn't interested in code style and common-mistake style linting. We already have a great tool for this!
* **Simple to use** - pup should be easy to drop onto a developer's desktop or into a CI pipeline and work seamlessly as a `cargo` extension
* **Simple to configure** - in the spirit of similar static analysis tools, pup reads from `pup.yaml` dropped into the root of a project
* **Easy to integrate** - TODO - reference that standard for exporting linting syntax. 
