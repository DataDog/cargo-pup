#!/bin/bash

#
# Run before a publish to bump versions. We should
# also create a new git commit for the release, and then
# create a release from this commit from the GitHub UI.
# 
cargo workspaces version --no-git-commit minor -y
