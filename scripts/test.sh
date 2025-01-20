#!/bin/bash


#
# Test invocation of gsr-driver
#
set -e 

cargo build

install_name_tool -add_rpath `rustc --print sysroot`/lib/ target/debug/cargo-gsr
install_name_tool -add_rpath `rustc --print sysroot`/lib/ target/debug/gsr-driver

target/debug/gsr-driver `rustc --print sysroot`/bin/rustc - --crate-name ___ --print=file-names --crate-type bin --crate-type rlib --crate-type dylib --crate-type cdylib --crate-type staticlib --crate-type proc-macro --print=sysroot --print=split-debuginfo --print=crate-name --print=cfg
