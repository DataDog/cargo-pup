#!/bin/bash

#
# Run before a publish to bump versions. We should
# also create a new git commit for the release, and then
# create a release from this commit from the GitHub UI.
# 
set -e

# Bump versions in Cargo.toml files
cargo workspaces version --no-git-commit --exact --force '*' patch -y

# Get the new version from the root Cargo.toml
NEW_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Update version references in README files
sed -i '' "s/cargo_pup_lint_config = \"[^\"]*\"/cargo_pup_lint_config = \"$NEW_VERSION\"/g" README.md
sed -i '' "s/cargo_pup_lint_config = \"[^\"]*\"/cargo_pup_lint_config = \"$NEW_VERSION\"/g" cargo_pup_lint_config/README.md

echo "Updated versions to $NEW_VERSION"
