#!/usr/bin/env bash

# cd lua && cargo build --release && cd docker_example && ./run.sh

set -v

[ "$(docker ps -a | grep jsonpath)" ] && docker kill jsonpath

docker run -d --rm --name jsonpath \
  -v "${PWD}/../../benchmark/example.json":/etc/jsonpath/example/example.json:ro \
  -v "${PWD}/../jsonpath.lua":/etc/jsonpath/jsonpath.lua:ro \
  -v "${PWD}/testa.lua":/etc/jsonpath/testa.lua:ro \
  -v "${PWD}/init.lua":/etc/jsonpath/init.lua:ro \
  -v "${PWD}/../target/release/deps/libjsonpath_lib.so":/etc/jsonpath/libjsonpath_lib.so:ro \
  -v "${PWD}/default.conf":/etc/nginx/conf.d/default.conf \
  -p 8080:80 \
  openresty/openresty:bionic

sleep 1
curl http://localhost:8080/3
