#!/bin/bash
set -e

if [ -d "target/release" ]; then
    ./target/release/bench_bin
else
    echo "빌드먼저"
fi