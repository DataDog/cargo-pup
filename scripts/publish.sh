#!/bin/bash

#
# Publish it! Versions should've been bumped, committed, and tagged.
# this is run. Ultimately we want to trigger this from the tag release
# in github actions.
# 
cargo workspaces publish --no-git-commit --publish-as-is
