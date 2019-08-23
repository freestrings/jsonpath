#!/usr/bin/env bash

# cd lua/docker_example && ./run.sh

set -v

docker run -d --rm --name jsonpath \
  -v "${PWD}/../../benchmark/example.json":/etc/jsonpath/example/example.json:ro \
  -v "${PWD}/../jsonpath.lua":/etc/jsonpath/jsonpath.lua:ro \
  -v "${PWD}/../target/release/deps/libjsonpath_lib.so":/etc/jsonpath/libjsonpath_lib.so:ro \
  -v "${PWD}/default.conf":/etc/nginx/conf.d/default.conf \
  -p 8080:80 \
  openresty/openresty:bionic

docker exec -it jsonpath bash -c "curl localhost"