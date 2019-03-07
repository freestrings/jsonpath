#!/bin/bash
set -e

cd ../../wasm && ./build.sh nodejs
cd ../benches/javascript && npm link jsonpath-wasm