#!/bin/bash
#
# 
# This is a simple script to download and install
# cargo-pup on a local machine. In the future it'll
# be nice and curl-able and use the github release machinery,
# for now you should just download it by itself then run it.
#

set -e

if [ ! -d ~/.local/bin ] ; then
  echo "You need to create an XDG bin directory at ~/.local/bin and put it in your shell path!"
  exit 1
fi

# Where's our script?
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script_parent_dir="$(dirname "$script_dir")"

# If we're not in a checked-out copy of the repository, clone it to a temporary directory
if ! (cd "$script_parent_dir" && git remote -v | grep -q 'DataDog/cargo-pup'); then
  echo "No working copy of cargo-pup found; cloning"
  tmpdir=$(mktemp -d)
  pushd $tmpdir
  git clone git@github.com:DataDog/cargo-pup.git
  cd cargo-pup
else
  echo "Using enclosing working copy of cargo-pup"
  pushd $script_parent_dir
fi

cargo build --release --all
cp target/release/cargo-pup ~/.local/bin/
cp target/release/pup-driver ~/.local/bin/
popd

if [[ -n "${tmpdir+x}" ]]; then
  rm -rf $tmpdir
fi
