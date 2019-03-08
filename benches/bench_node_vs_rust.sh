#!/bin/bash

set -e
DIR="$(pwd)"

cd "${DIR}"/bench_bin && cargo build --release

printf "\n\n$..book[?(@.price<30 && @.category=="fiction")] (loop 100,000)"
printf "\n\n"

echo "Rust: " && time ./bench.sh
printf "\n"
cd "${DIR}"/javascript && echo "NodeJs - jsonpath module: " && time ./bench.sh jsonpathOnly
printf "\n"
cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm module: " && time ./bench.sh jsonpathWasmOnly
