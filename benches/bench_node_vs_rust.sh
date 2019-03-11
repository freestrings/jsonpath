#!/bin/bash

set -e
DIR="$(pwd)"

cd "${DIR}"/bench_bin && cargo build --release

printf "\n\n$..book[?(@.price<30 && @.category=="fiction")] (loop 100,000)"
printf "\n\n"

echo "Rust: " && time ./bench.sh
printf "\n"
cd "${DIR}"/javascript && echo "NodeJs - jsonpath module: " && time ./bench.sh jsonpath
printf "\n"
cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm module - selector: " && time ./bench.sh wasmSelector
printf "\n"
cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm module - compile: " && time ./bench.sh wasmCompile
printf "\n"
cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm module - compile-alloc: " && time ./bench.sh wasmCompileAlloc
printf "\n"
cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm module - select:" && time ./bench.sh wasmSelect
printf "\n"
cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm module - select-alloc:" && time ./bench.sh wasmSelectAlloc