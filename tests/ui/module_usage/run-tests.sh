#!/bin/bash
# Script to run both module_usage tests with their respective configurations

set -e  # Exit on error

echo "Running module_usage_test.rs (DenyWildcard and Deny rules)..."
cp pup.module_usage.yaml pup.yaml
TESTNAME=module_usage/module_usage_test.rs cargo test --test ui-test

echo "Running allow_only_test.rs (AllowOnly rule)..."
cp pup.allow_only.yaml pup.yaml
TESTNAME=module_usage/allow_only_test.rs cargo test --test ui-test

echo "All tests passed!"