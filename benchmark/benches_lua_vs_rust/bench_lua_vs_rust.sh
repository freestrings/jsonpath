#!/bin/bash

set -e

# http://luajit.org/index.html

cargo clean && \
cargo build --release

export JSONPATH_LIB_PATH="${PWD}/../../target/release"
export LUA_PATH="${PWD}/../../lua/?.lua;"

echo
time cargo run --release -- 1000
echo
time luajit example.lua 1000
echo
time cargo run --release -- 5000
echo
time luajit example.lua 5000
echo
time cargo run --release -- 10000
echo
time luajit example.lua 10000

