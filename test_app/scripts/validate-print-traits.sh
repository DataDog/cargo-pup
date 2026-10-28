#!/usr/bin/env bash
# This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

set -euo pipefail

if [[ -n "${CARGO_PUP_BIN_DIR:-}" ]]; then
    if [[ ! -x "${CARGO_PUP_BIN_DIR}/cargo-pup" || ! -x "${CARGO_PUP_BIN_DIR}/pup-driver" ]]; then
        echo "cargo-pup binaries not found in configured directory: ${CARGO_PUP_BIN_DIR}" >&2
        exit 1
    fi
    export PATH="${CARGO_PUP_BIN_DIR}:$PATH"
else
    PATHS=""
    [ -d "$(dirname "$0")/../../target/release" ] && PATHS="$(cd "$(dirname "$0")/../../target/release" && pwd):$PATHS"
    [ -d "$(dirname "$0")/../../target/debug" ] && PATHS="$(cd "$(dirname "$0")/../../target/debug" && pwd):$PATHS"
    export PATH="${PATHS}$PATH"
fi

pushd "$(dirname "$0")/../" >/dev/null
if ! output="$(cargo pup print-traits 2>&1)"; then
    echo "cargo pup print-traits failed:" >&2
    echo "$output" >&2
    exit 1
fi
popd >/dev/null

output="$(sed $'s/\e\[[0-9;]*m//g' <<<"$output")"

expected_lines=(
    "::trait_impl::MyTrait [trait_restrictions]"
    "::configured_struct_rules::RequiredTrait [negated_trait_matcher_check, required_trait_rule_check]"
    "::configured_struct_rules::GenericRequiredTrait [generic_trait_matcher_check, required_generic_trait_rule_check]"
)

for expected in "${expected_lines[@]}"; do
    if ! grep -Fq "$expected" <<<"$output"; then
        echo "Expected print-traits output to contain: $expected" >&2
        echo "$output" >&2
        exit 1
    fi
done
