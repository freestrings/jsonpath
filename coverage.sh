#!/usr/bin/env bash

#
# cargo install cargo-tarpaulin
#

set -e

cargo tarpaulin --exclude-files nodejs wasm src/parser/mod.rs -v --all