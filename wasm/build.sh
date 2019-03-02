#!/bin/bash

set -e

cd ./www && \
    rm -rf dist && \
    rm -rf node_modules && \
    npm install && \
    cd .. && \
    wasm-pack build --target=$1 --out-dir=www/node_modules/rs-jsonpath

