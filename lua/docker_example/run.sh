#!/usr/bin/env bash

# cd lua/docker_example && ./run.sh

set -v

docker kill jsonpath

docker run -d --rm --name jsonpath \
  -v "${PWD}/../../benchmark/example.json":/etc/jsonpath/example/example.json:ro \
  -v "${PWD}/../jsonpath.lua":/etc/jsonpath/jsonpath.lua:ro \
  -v "${PWD}/testa.lua":/etc/jsonpath/testa.lua:ro \
  -v "${PWD}/init.lua":/etc/jsonpath/init.lua:ro \
  -v "${PWD}/../target/release/deps/libjsonpath_lib.so":/etc/jsonpath/libjsonpath_lib.so:ro \
  -v "${PWD}/default.conf":/etc/nginx/conf.d/default.conf \
  -p 8080:80 \
  openresty/openresty:bionic

paths=(
    "$.store.book[*].author"
    "$..author"
    "$.store.*"
    "$.store..price"
    "$..book[2]"
    "$..book[-2]"
    "$..book[0,1]"
    "$..book[:2]"
    "$..book[1:2]"
    "$..book[-2:]"
    "$..book[2:]"
    "$..book[?(@.isbn)]"
    "$.store.book[?(@.price == 10)]"
    "$..*"
    "$..book[ ?( (@.price < 13 || $.store.bicycle.price < @.price) && @.price <=10 ) ]"
    "$.store.book[?( (@.price < 10 || @.price > 10) && @.price > 10 )]"
)

encoded_paths=()

for i in "${!paths[@]}"
do
  encoded_paths+=("$(node --eval "console.log(encodeURIComponent('${paths[$i]}'))")")
  curl http://localhost:8080/1?path="${encoded_paths[$i]}"
done

for i in "${!encoded_paths[@]}"
do
  ab -n 1000 -c 10 http://localhost:8080/1?path="${encoded_paths[$i]}" > ab_results/"${paths[$i]}".txt
done

