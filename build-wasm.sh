#!/bin/bash

set -e

# project_root
DIR="$(pwd)"
WASM="${DIR}"/wasm
WASM_WWW="${WASM}"/www
WASM_WWW_BENCH="${WASM}"/www_bench
WASM_BROWSER_PKG="${WASM}"/browser_pkg
WASM_NODEJS_PKG="${WASM}"/nodejs_pkg
WASM_ALL_PKG="${WASM}"/all_pkg
WASM_TEST="${WASM}"/tests
DOCS="${DIR}"/docs
DOCS_BENCH="${DOCS}"/bench

__msg () {
    echo ">>>>>>>>>>$1<<<<<<<<<<"
}

__cargo_clean () {
    cd "${WASM}" && cargo clean && \
    cd "${DIR}" && cargo clean
}

echo
__msg "clean wasm"
rm -rf \
    "${WASM_NODEJS_PKG}" \
    "${WASM_BROWSER_PKG}" \
    "${WASM_ALL_PKG}" \
    "${WASM_WWW}"/node_modules \
    "${WASM_WWW_BENCH}"/node_modules \
    "${WASM_WWW}"/dist \
    "${WASM_WWW_BENCH}"/dist \
    "${WASM_TEST}"/node_modules

if [ "$1" = "all" ]; then
    __msg "clean all wasm"
    __cargo_clean
fi

__msg "npm install: wasm"
cd "${WASM_WWW}" && npm install
__msg "npm install: wasm_bench"
cd "${WASM_WWW_BENCH}" && npm install
__msg "npm install: wasm test"
cd "${WASM_TEST}" && npm install

echo
echo
__msg "wasm-pack"
cd "${WASM}" && \
    wasm-pack build --release --target=nodejs --out-dir "${WASM_NODEJS_PKG}"

cd "${WASM}" && \
    wasm-pack build --release --target=browser --out-dir "${WASM_BROWSER_PKG}"
#    && \
#    wasm-pack test --chrome --firefox --headless

__msg "wasm npm packaging"
cp -r "${WASM_BROWSER_PKG}" "${WASM_ALL_PKG}/" && \
    sed "s/require[\(]'\.\/jsonpath_wasm_bg/require\('\.\/jsonpath_wasm_nodejs/" "${WASM_NODEJS_PKG}/jsonpath_wasm.js" \
        > "${WASM_ALL_PKG}/jsonpath_wasm_main.js" && \
    sed "s/require[\(]'\.\/jsonpath_wasm/require\('\.\/jsonpath_wasm_main/" "${WASM_NODEJS_PKG}/jsonpath_wasm_bg.js" \
        > "${WASM_ALL_PKG}/jsonpath_wasm_nodejs.js" && \
    jq ".files += [\"jsonpath_wasm_nodejs.js\"]" ${WASM_ALL_PKG}/package.json \
        | jq ".main = \"jsonpath_wasm_main.js\"" \
        | jq ".keywords += [\"jsonpath\", \"json\", \"webassembly\", \"parsing\", \"rust\"]" \
        > ${WASM_ALL_PKG}/temp.json && \
    mv -v "${WASM_ALL_PKG}/temp.json" "${WASM_ALL_PKG}/package.json" && \
    cd "${WASM_ALL_PKG}" && npm link

echo
__msg "link"
cd "${WASM_WWW}" && \
    npm link jsonpath-wasm

cd "${WASM_WWW_BENCH}" && \
    npm link jsonpath-wasm

cd "${WASM_TEST}" && \
    npm link jsonpath-wasm

echo
echo
__msg "wasm test"
cd "${WASM_TEST}" && npm test

if [ "$1" = "all" ] || [ "$1" = "docs" ]; then
    echo
    __msg "docs"
    cd "${WASM_WWW}" && \
        npm run build &&
        rm -f "${DOCS}"/*.js "${DOCS}"/*.wasm "${DOCS}"/*.html && \
        cp "${WASM_WWW}"/dist/*.* "${DOCS}"/

    cd "${WASM_WWW_BENCH}" && \
        npm run build &&
        rm -f "${DOCS_BENCH}"/*.js "${DOCS_BENCH}"/*.wasm "${DOCS_BENCH}"/*.html && \
        cp "${WASM_WWW_BENCH}"/dist/*.* "${DOCS_BENCH}"/
fi

__msg "wasm done"