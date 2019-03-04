# Jsonpath

[JsonPath](https://goessner.net/articles/JsonPath/) Rust 구현

## 왜?
To enjoy Rust

## 사용법

## With Javascript (WebAssembly)


[Demo]: https://freestrings.github.io/jsonpath/

**(not yet published `jsonpath-wasm`)**

```javascript
import * as jsonpath from "jsonpath-wasm";

//
// data
//
let jsonString = "{\"a\" : 1}";

//
// reuse a compiled jsonpath
//
let template = jsonpath.compile("$.a");
// read as json string
template(jsonString)
// read as json object
template(JSON.parse(jsonString));

//
// reuse a json
//

// as json string
let reader1 = jsonpath.reader(jsonString);
reader1("$.a");

// as json object
let reader2 = jsonpath.reader(JSON.parse(jsonString));
reader2("$.a");

// read every time
jsonpath.read(JSON.parse(jsonString), "$.a");
jsonpath.read(jsonString, "$.a");
```


## On Shell

- 

## As Library

- 

## With AWS API Gateway

-

# 성능테스트

