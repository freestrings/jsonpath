#!/bin/bash

set -e
DIR="$(pwd)"

cd "${DIR}"/bench_bin && cargo build --release

ITER=100000

printf "\n\n$..book[?(@.price<30 && @.category=="fiction")] (loop ${ITER})"
printf "\n\n"

__default () {
    echo "Rust - select: " && time ./bench.sh select ${ITER}
    printf "\n"
    sleep 1
    cd "${DIR}"/javascript && echo "NodeJs - jsonpath - query: " && time ./bench.sh jsonpath ${ITER}
    printf "\n"
    sleep 1
    cd "${DIR}"/javascript && echo "NodeJs - jsonpath-rs - select:" && time ./bench.sh nativeSelect ${ITER}
}

__extra () {
    echo "Rust - selector: " && time ./bench.sh selector ${ITER}
    printf "\n"
    sleep 1
    echo "Rust - compile: " && time ./bench.sh compile ${ITER}
    printf "\n"
    sleep 1
    cd "${DIR}"/javascript && echo "NodeJs - jsonpath - query: " && time ./bench.sh jsonpath ${ITER}
    printf "\n"
    sleep 1
    cd "${DIR}"/javascript && echo "NodeJs - jsonpath-rs - selector: " && time ./bench.sh nativeSelector ${ITER}
    printf "\n"
    sleep 1
    cd "${DIR}"/javascript && echo "NodeJs - jsonpath-rs - compile: " && time ./bench.sh nativeCompile ${ITER}
}

if [ "$1" = "extra" ]; then
    __extra
else
    __default
fi