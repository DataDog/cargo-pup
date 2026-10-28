#!/usr/bin/env bash
# This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

NEXTEST_PINNED_VERSION="0.9.145"

if ! NEXTEST_VERSION_OUTPUT="$(cargo nextest --version 2>&1)"; then
    echo "cargo-nextest is required. Install it with:" >&2
    echo "  cargo install cargo-nextest --version ${NEXTEST_PINNED_VERSION} --locked" >&2
    exit 1
fi

if ! grep -q "$NEXTEST_PINNED_VERSION" <<<"$NEXTEST_VERSION_OUTPUT"; then
    echo "Warning: installed cargo-nextest ($NEXTEST_VERSION_OUTPUT) does not match the CI-pinned version ${NEXTEST_PINNED_VERSION}; local results may diverge from CI." >&2
fi

# Build the binaries explicitly because integration tests and test_app need
# cargo-pup and pup-driver rather than Cargo's test artefacts.
cargo build --bins

# Cargo configuration and CARGO_TARGET_DIR can move build output away from
# target/. Resolve the directory Cargo actually used so the end-to-end
# validators cannot select stale or globally installed binaries.
CARGO_TARGET_DIRECTORY="$(
    cargo metadata --format-version 1 --no-deps \
        | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])'
)"
export CARGO_PUP_BIN_DIR="${CARGO_TARGET_DIRECTORY}/debug"

# Exclude only the custom UI harness so future standard tests remain included.
cargo nextest run --workspace -E 'not binary(ui-test)'

# Nextest does not execute doctests or custom test harnesses.
cargo test --workspace --doc
cargo test --test ui-test

# test_app is a separate workspace with its own toolchain and ordinary tests.
(
    cd test_app
    cargo nextest run
)

# Force the end-to-end validation to compile with the binaries built above.
# Its first run warms this cache, and its second run compares stable output.
rm -rf test_app/.pup
test_app/scripts/validate-against-expected.sh
test_app/scripts/validate-print-traits.sh
