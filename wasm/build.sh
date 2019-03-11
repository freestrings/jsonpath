#!/bin/bash

set -e

# project_root/wasm
DIR="$(pwd)"

cd "${DIR}"/www && \
    rm -rf "${DIR}"/www/dist && \
    rm -rf "${DIR}"/www/node_modules && \
    rm -rf "${DIR}"/www_bench/dist && \
    rm -rf "${DIR}"/www_bench/node_modules && \
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
echo
echo
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
echo
echo
echo
echo
echo "-------------------- start build browser bench pkg --------------------"
echo
rm -rf "${DIR}"/www_bench/node_modules && \
cd "${DIR}"/www_bench && npm install && npm link @browser/jsonpath-wasm
echo "-------------------- build browser bench pkg done --------------------"

echo
echo
echo
echo
echo
echo
echo "-------------------- start build docs --------------------"
cd "${DIR}"/www && \
    npm run build && \
    rm -f "${DIR}"/../docs/*.js && rm -f "${DIR}"/../docs/*.wasm && rm -f "${DIR}"/../docs/*.html && \
    cp "${DIR}"/www/dist/*.* "${DIR}"/../docs/
echo "-------------------- build docs done --------------------"

echo
echo
echo
echo
echo
echo
echo "-------------------- start build docs bench --------------------"
cd "${DIR}"/www_bench && \
    npm run build && \
    rm -f "${DIR}"/../docs/bench/*.js && rm -f "${DIR}"/../docs/bench/*.wasm && rm -f "${DIR}"/../docs/bench/*.html && \
    cp "${DIR}"/www_bench/dist/*.* "${DIR}"/../docs/bench/
echo "-------------------- build docs bench done --------------------"