#!/bin/bash

# Set strict error handling
set -euo pipefail

# Define binary names and paths
BINARIES=("pup-driver" "cargo-pup")
TARGET_DIR="target/debug"
TOOLCHAIN_LIB_DIR=$(rustc --print sysroot)/lib

# Build the binaries
echo "Building binaries..."
cargo build --all-targets

# Locate and copy librustc_driver.dylib
echo "Locating and copying librustc_driver.dylib..."
LIBRUSTC_DRIVER=$(find "$TOOLCHAIN_LIB_DIR" -name 'librustc_driver*.dylib' | head -n 1)

if [[ -z "$LIBRUSTC_DRIVER" ]]; then
  echo "Error: librustc_driver.dylib not found in $TOOLCHAIN_LIB_DIR"
  exit 1
fi

cp "$LIBRUSTC_DRIVER" "$TARGET_DIR"
echo "Copied $LIBRUSTC_DRIVER to $TARGET_DIR"

# Update RPATH for each binary
for BINARY in "${BINARIES[@]}"; do
  BINARY_PATH="$TARGET_DIR/$BINARY"
  if [[ -f "$BINARY_PATH" ]]; then
    echo "Updating RPATH for $BINARY_PATH..."
    install_name_tool -add_rpath "@executable_path" "$BINARY_PATH"
  else
    echo "Warning: $BINARY_PATH does not exist, skipping RPATH update"
  fi
done

echo "Script completed successfully."

