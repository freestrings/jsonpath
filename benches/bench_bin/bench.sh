#!/bin/bash
set -e

if [ -d "target/release" ]; then
    ./target/release/bench_bin $1 $2
else
    echo "빌드먼저"
fi