#!/bin/bash

set -e

cd ./www && \
    rm -rf dist && \
    rm -rf node_modules && \
    npm install && \
    cd .. && \
    wasm-pack build --target=$1 && \
    cd pkg && npm link && \
    cd ../www && npm link jsonpath-wasm