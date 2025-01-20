#!/bin/bash

# 
# Locally installs cargo-gsr for the current toolchain 
# in the ~/.local/bin dir.
# This is super hacky. there must be a better way!
#

set -e 

cargo build --bin gsr-driver --bin cargo-gsr
install_name_tool -add_rpath `rustc --print sysroot`/lib/ target/debug/cargo-gsr
install_name_tool -add_rpath `rustc --print sysroot`/lib/ target/debug/gsr-driver

rm -f ~/.local/bin/cargo-gsr
rm -f ~/.local/bin/gsr-driver

cp target/debug/cargo-gsr ~/.local/bin/
cp target/debug/gsr-driver ~/.local/bin/
