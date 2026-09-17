#!/usr/bin/env bash
# This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

set -euo pipefail

PATHS=""
[ -d "$(dirname "$0")/../../target/release" ] && PATHS="$(cd "$(dirname "$0")/../../target/release" && pwd):$PATHS"
[ -d "$(dirname "$0")/../../target/debug" ] && PATHS="$(cd "$(dirname "$0")/../../target/debug" && pwd):$PATHS"
export PATH="${PATHS}$PATH"

pushd "$(dirname "$0")/../" >/dev/null
output="$(cargo pup print-traits 2>&1 | perl -pe 's/\e\[[0-9;]*m//g')"
popd >/dev/null

expected="::trait_impl::MyTrait [trait_restrictions]"
if ! grep -Fq "$expected" <<<"$output"; then
    echo "Expected print-traits output to contain: $expected" >&2
    echo "$output" >&2
    exit 1
fi
