#!/bin/bash
PATHS=""
[ -d "$(dirname "$0")/../../target/release" ] && PATHS="$(cd "$(dirname "$0")/../../target/release" && pwd):$PATHS"
[ -d "$(dirname "$0")/../../target/debug" ] && PATHS="$(cd "$(dirname "$0")/../../target/debug" && pwd):$PATHS"
export PATH="${PATHS}$PATH"

pushd "$(dirname "$0")/../" > /dev/null

# Run it once, so we don't see any tooling installs in the output
cargo pup 2>&1 > /dev/null

# Now run it for real and check
cargo pup 2>&1 | grep -v 'Finished' | grep -vE '^\s*Checking test_app' | diff - expected_output
if [ $? -ne 0 ]; then
  popd > /dev/null
  exit -1
fi
popd > /dev/null
