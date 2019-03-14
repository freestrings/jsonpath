#!/bin/bash

set -e
DIR="$(pwd)"

cd "${DIR}"/bench_bin && cargo build --release

ITER=100000

printf "\n\n$..book[?(@.price<30 && @.category=="fiction")] (loop ${ITER})"
printf "\n\n"

#echo "Rust - compile: " && time ./bench.sh compile ${ITER}
#printf "\n"
#sleep 1
#echo "Rust - selector: " && time ./bench.sh selector ${ITER}
#printf "\n"
#sleep 1
echo "Rust - select: " && time ./bench.sh select ${ITER}
printf "\n"
sleep 1
cd "${DIR}"/javascript && echo "NodeJs - jsonpath: " && time ./bench.sh jsonpath ${ITER}
printf "\n"
sleep 1
#cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm - selector: " && time ./bench.sh wasmSelector ${ITER}
#printf "\n"
#sleep 1
#cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm - compile: " && time ./bench.sh wasmCompile ${ITER}
#printf "\n"
#sleep 1
#cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm - compile-alloc: " && time ./bench.sh wasmCompileAlloc ${ITER}
#printf "\n"
#sleep 1
cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm - select:" && time ./bench.sh wasmSelect ${ITER}
printf "\n"
sleep 1
#cd "${DIR}"/javascript && echo "NodeJs - jsonpath-wasm - select-alloc:" && time ./bench.sh wasmSelectAlloc ${ITER}
#printf "\n"
#sleep 1
#cd "${DIR}"/javascript && echo "NodeJs - jsonpath-rs - compile:" && time ./bench.sh nativeCompile ${ITER}
#printf "\n"
#sleep 1
#cd "${DIR}"/javascript && echo "NodeJs - jsonpath-rs - selector:" && time ./bench.sh nativeSelector ${ITER}
#printf "\n"
#sleep 1
cd "${DIR}"/javascript && echo "NodeJs - jsonpath-rs - select:" && time ./bench.sh nativeSelect ${ITER}