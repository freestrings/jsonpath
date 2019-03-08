#!/bin/bash

set -e

# project_root/wasm
DIR="$(pwd)"

cd "${DIR}"/www && \
    rm -rf "${DIR}"/dist && \
    rm -rf "${DIR}"/node_modules && \
    npm install && \
    cd "${DIR}"

echo "-------------------- start build nodejs pkg --------------------"
echo

rm -rf "${DIR}"/wasm/nodejs_pkg && \
wasm-pack build --target=nodejs --scope nodejs --out-dir nodejs_pkg && \
cd "${DIR}"/nodejs_pkg && npm link && \
rm -rf "${DIR}"/../benches/javascript/node_modules && \
cd "${DIR}"/../benches/javascript && npm install && \
npm link @nodejs/jsonpath-wasm
echo "-------------------- build nodejs pkg done --------------------"

cd "${DIR}"

echo
echo
echo "-------------------- start build browser pkg --------------------"
echo
rm -rf "${DIR}"/wasm/browser_pkg && \
wasm-pack build --target=browser --scope browser --out-dir browser_pkg && \
cd "${DIR}"/browser_pkg && npm link && \
cd "${DIR}"/www && npm link @browser/jsonpath-wasm
echo "-------------------- build browser pkg done --------------------"

echo
echo
echo "-------------------- start build browser bench pkg --------------------"
echo
rm -rf "${DIR}"/www_bench/node_modules && \
cd "${DIR}"/www_bench && npm install && npm link @browser/jsonpath-wasm
echo "-------------------- build browser bench pkg done --------------------"