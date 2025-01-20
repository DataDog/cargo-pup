#!/bin/bash

# 
# Locally installs cargo-up for the current toolchain 
# in the ~/.local/bin dir.
# This is super hacky. there must be a better way!
#

set -e 

cargo build --bin pup-driver --bin cargo-up
install_name_tool -add_rpath `rustc --print sysroot`/lib/ target/debug/cargo-up
install_name_tool -add_rpath `rustc --print sysroot`/lib/ target/debug/pup-driver

rm -f ~/.local/bin/cargo-up
rm -f ~/.local/bin/pup-driver

cp target/debug/cargo-up ~/.local/bin/
cp target/debug/pup-driver ~/.local/bin/
