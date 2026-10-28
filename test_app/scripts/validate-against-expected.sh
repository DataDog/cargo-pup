#!/bin/bash
# This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

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

pushd "$(dirname "$0")/../" > /dev/null

# Run it once, so we don't see any tooling installs in the output
cargo pup > /dev/null 2>&1

# Now run it for real and check
cargo pup 2>&1 | grep -v 'Finished' | grep -vE '^\s*Checking .* \(\/.*\)' | grep -vE '^\s*Compiling .* \(\/.*\)' | diff - expected_output
if [ $? -ne 0 ]; then
  popd > /dev/null
  exit -1
fi
popd > /dev/null
