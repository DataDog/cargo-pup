#!/bin/bash


#
# Test invocation of pup-driver
#
set -e 

cargo build

install_name_tool -add_rpath `rustc --print sysroot`/lib/ target/debug/cargo-pup
install_name_tool -add_rpath `rustc --print sysroot`/lib/ target/debug/pup-driver

target/debug/pup-driver `rustc --print sysroot`/bin/rustc - --crate-name ___ --print=file-names --crate-type bin --crate-type rlib --crate-type dylib --crate-type cdylib --crate-type staticlib --crate-type proc-macro --print=sysroot --print=split-debuginfo --print=crate-name --print=cfg
