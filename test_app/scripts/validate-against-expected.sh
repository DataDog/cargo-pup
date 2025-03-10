#!/bin/bash
export PATH="$(cd "$(dirname "$0")/../../target/release" && pwd):$(cd "$(dirname "$0")/../../target/debug" && pwd):$PATH"
pushd "$(dirname "$0")/../"
cargo pup 2>&1 | grep -v 'Finished' | diff - expected_output
popd