#!/usr/bin/env bash

#
# cargo install cargo-tarpaulin
#

set -e

cargo tarpaulin --exclude-files \
  nodejs \
  wasm \
  src/parser/mod.rs \
  src/select/mod.rs \
  src/select/cmp.rs \
  src/select/expr_term.rs \
  src/select/value_walker.rs \
  -v --all