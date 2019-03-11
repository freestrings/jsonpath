# jsonpath-lib

[![Build Status](https://travis-ci.org/freestrings/jsonpath.svg?branch=master)](https://travis-ci.org/freestrings/jsonpath)
![crates.io](https://img.shields.io/crates/v/jsonpath_lib.svg)

`Rust` 버전 [JsonPath](https://goessner.net/articles/JsonPath/) 구현이다. Rust 구현과 동일한 기능을 `Webassembly` 로 제공하는 것도 목표.

The `Rust` version is a [JsonPath](https://goessner.net/articles/JsonPath/) implementation. It is also aimed to provide the same functionality as `Webassembly` in Rust implementation.

## 왜?

To enjoy Rust!

## 목차

[With Javascript (Webassembly)](#with-javascript-webassembly)

- [jsonpath-wasm library](#jsonpath-wasm-library)
- [javascript - jsonpath.select(json: string|object, jsonpath: string)](#javascript---jsonpathselectjson-stringobject-jsonpath-string)
- [javascript - jsonpath.compile(jsonpath: string)](#javascript---jsonpathcompilejsonpath-string)
- [javascript - jsonpath.selector(json: string|object)](#javascript---jsonpathselectorjson-stringobject)
- [javascript - alloc_json, dealloc_json](#javascript---alloc_json-dealloc_json)
- [javascript - examples](#javascript---examples)

[With Rust (as library)](#with-rust-as-library)

- [jsonpath_lib library](#jsonpath_lib-library)
- [rust - jsonpath::select(json: serde_json::value::Value, jsonpath: &str)](#rust---jsonpathselectjson-serde_jsonvaluevalue-jsonpath-str)
- [rust - jsonpath::compile(jsonpath: &str)](#rust---jsonpathcompilejsonpath-str)
- [rust - jsonpath::selector(json: serde_json::value::Value)](#rust---jsonpathselectorjson-serde_jsonvaluevalue)
- [rust - examples](#rust---examples)

[With AWS API Gateway](#with-aws-api-gateway)

[Simple time check](#simple-time-check-with-dchesterjsonpath)

## With Javascript (WebAssembly)

### jsonpath-wasm library

*(not yet published `jsonpath-wasm`)*
```javascript
// browser
import * as jsonpath from "jsonpath-wasm";
// nodejs
let jsonpath = require('jsonpath-wasm');
```

### javascript - jsonpath.select(json: string|object, jsonpath: string)

```javascript
let jsonObj = {
   "school": {
       "friends": [{"id": 0}, {"id": 1}]
   },
   "friends": [{"id": 0}, {"id": 1}]
};
let ret = [{"id": 0}, {"id": 0}];

let a = jsonpath.select(JSON.stringify(jsonObj), "$..friends[0]");
let b = jsonpath.select(jsonObj, "$..friends[0]");
console.log(
    JSON.stringify(ret) == JSON.stringify(a),
    JSON.stringify(a) == JSON.stringify(b)
);
```

### javascript - jsonpath.compile(jsonpath: string)

```javascript
let template = jsonpath.compile("$..friends[0]");

let jsonObj = {
    "school": {
        "friends": [ {"id": 0}, {"id": 1} ]
    },
    "friends": [ {"id": 0}, {"id": 1} ]
};

let ret = JSON.stringify([ {"id": 0}, {"id": 0} ]);

// 1. read as json object
console.log(JSON.stringify(template(jsonObj)) == ret);
// 2. read as json string
console.log(JSON.stringify(template(JSON.stringify(jsonObj))) == ret);

let jsonObj2 = {
    "school": {
        "friends": [ 
            {"name": "Millicent Norman"}, 
            {"name": "Vincent Cannon"} 
        ]
    },
    "friends": [ {"id": 0}, {"id": 1} ]
};

let ret2 = JSON.stringify([ {"id": 0}, {"name": "Millicent Norman"} ]);

// 1. read as json object
console.log(JSON.stringify(template(jsonObj2)) == ret2);
// 2. read as json string
console.log(JSON.stringify(template(JSON.stringify(jsonObj2))) == ret2);
```

### javascript - jsonpath.selector(json: string|object)

```javascript
let jsonObj = {
    "school": {
        "friends": [{"id": 0}, {"id": 1}]
    },
    "friends": [{"id": 0},{"id": 1}]
};

let ret1 = JSON.stringify([ {"id": 0}, {"id": 0} ]);
let ret2 = JSON.stringify([ {"id": 1}, {"id": 1} ]);

// 1. read as json object
let selector = jsonpath.selector(jsonObj);
console.log(JSON.stringify(selector("$..friends[0]")) == ret1);
console.log(JSON.stringify(selector("$..friends[1]")) == ret2);

// 2. read as json string
let selector = jsonpath.selector(JSON.stringify(jsonObj));
console.log(JSON.stringify(selector("$..friends[0]")) == ret1);
console.log(JSON.stringify(selector("$..friends[1]")) == ret2);
```

### javascript - alloc_json, dealloc_json

wasm-bindgen은 Javascript와 Webassembly 간 값을 주고받을 때 JSON 객체는 String으로 변환되기 때문에, 반복해서 사용되는 JSON 객체를 Webassembly 영역에 생성해 두면 성능에 도움이 된다.

Since wasm-bindgen converts JSON objects to String when exchanging values between Javascript and Webassembly, it is helpful to create repeated Json objects in Webassembly area.

```javascript

let jsonObj = {
    "school": {
        "friends": [{"id": 0}, {"id": 1}]
    },
    "friends": [{"id": 0},{"id": 1}]
};

let path = '$..friends[0]';
let template = jsonpath.compile(path);
let selector = jsonpath.selector(jsonObj);

let ptr = jsonpath.alloc_json(jsonObj);
if(ptr == 0) console.error('invalid ptr'); // `0` is invalid pointer
let selector2 = jsonpath.selector(ptr);

let ret1 = selector(path)
let ret2 = selector2(path)
let ret3 = template(jsonObj);
let ret4 = template(ptr);
let ret5 = jsonpath.select(jsonObj, path);
let ret6 = jsonpath.select(ptr, path);

console.log(
    JSON.stringify(ret1) == JSON.stringify(ret2),// true
    JSON.stringify(ret1) == JSON.stringify(ret3),// true
    JSON.stringify(ret1) == JSON.stringify(ret4),// true
    JSON.stringify(ret1) == JSON.stringify(ret5),// true
    JSON.stringify(ret1) == JSON.stringify(ret6));// true

jsonpath.dealloc_json(ptr);

```

### javascript - examples

**Demo**: https://freestrings.github.io/jsonpath/

json 데이터 *(참고 사이트: https://github.com/json-path/JsonPath)*

```javascript
{
    "store": {
        "book": [
            {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
            },
            {
                "category": "fiction",
                "author": "Evelyn Waugh",
                "title": "Sword of Honour",
                "price": 12.99
            },
            {
                "category": "fiction",
                "author": "Herman Melville",
                "title": "Moby Dick",
                "isbn": "0-553-21311-3",
                "price": 8.99
            },
            {
                "category": "fiction",
                "author": "J. R. R. Tolkien",
                "title": "The Lord of the Rings",
                "isbn": "0-395-19395-8",
                "price": 22.99
            }
        ],
        "bicycle": {
            "color": "red",
            "price": 19.95
        }
    },
    "expensive": 10
}
```


| JsonPath (click link to try)| Result |
| :------- | :----- |
| <a href="https://freestrings.github.io/jsonpath/?path=$.store.book[*].author" target="_blank">$.store.book[*].author</a>| The authors of all books     |
| <a href="https://freestrings.github.io/jsonpath/?path=$..author" target="_blank">$..author</a>                   | All authors                         |
| <a href="https://freestrings.github.io/jsonpath/?path=$.store.*" target="_blank">$.store.*</a>                  | All things, both books and bicycles  |
| <a href="https://freestrings.github.io/jsonpath/?path=$.store..price" target="_blank">$.store..price</a>             | The price of everything         |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[2]" target="_blank">$..book[2]</a>                 | The third book                      |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[2]" target="_blank">$..book[-2]</a>                 | The second to last book            |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[0,1]" target="_blank">$..book[0,1]</a>               | The first two books               |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[:2]" target="_blank">$..book[:2]</a>                | All books from index 0 (inclusive) until index 2 (exclusive) |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[1:2]" target="_blank">$..book[1:2]</a>                | All books from index 1 (inclusive) until index 2 (exclusive) |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[-2:]" target="_blank">$..book[-2:]</a>                | Last two books                   |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[2:]" target="_blank">$..book[2:]</a>                | Book number two from tail          |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[?(@.isbn)]" target="_blank">$..book[?(@.isbn)]</a>          | All books with an ISBN number         |
| <a href="https://freestrings.github.io/jsonpath/?path=$.store.book[?(@.price < 10)]" target="_blank">$.store.book[?(@.price < 10)]</a> | All books in store cheaper than 10  |
| <a href="https://freestrings.github.io/jsonpath/?path=$..*" target="_blank">$..*</a>                        | Give me every thing
| <a href="https://freestrings.github.io/jsonpath/?path=%24..book%5B%3F((%40.price%20%3D%3D%2012.99%20%7C%7C%20%24.store.bicycle.price%20%3C%20%40.price)%20%7C%7C%20%40.category%20%3D%3D%20%22reference%22)%5D" target="_blank">$..book[ ?(<br>(@.price == 12.99 &#124; &#124; $.store.bicycle.price < @.price)<br> &#124;&#124; @.category == "reference"<br>)]</a> | Complex filter


## With Rust (as library)

### jsonpath_lib library

```rust
extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;
```

### rust - jsonpath::select(json: serde_json::value::Value, jsonpath: &str)

```rust
let json_obj = json!({
    "school": {
        "friends": [{"id": 0}, {"id": 1}]
    },
    "friends": [{"id": 0}, {"id": 1}]
});
let json = jsonpath::select(json_obj, "$..friends[0]").unwrap();
let ret = json!([ {"id": 0}, {"id": 0} ]);
assert_eq!(json, ret)
```

### rust - jsonpath::compile(jsonpath: &str)

```rust
let mut template = jsonpath::compile("$..friends[0]");

let json_obj = json!({
    "school": {
        "friends": [ {"id": 0}, {"id": 1} ]
    },
    "friends": [ {"id": 0}, {"id": 1} ]
});

let json = template(json_obj).unwrap();
let ret = json!([ {"id": 0}, {"id": 0} ]);
assert_eq!(json, ret);

let json_obj = json!({
    "school": {
        "friends": [ {"name": "Millicent Norman"}, {"name": "Vincent Cannon"} ]
    },
    "friends": [ {"id": 0}, {"id": 1} ]
});

let json = template(json_obj).unwrap();
let ret = json!([ {"id": 0}, {"name": "Millicent Norman"} ]);
assert_eq!(json, ret);
```

### rust - jsonpath::selector(json: serde_json::value::Value)

```rust
let json_obj = json!({
    "school": {
        "friends": [{"id": 0}, {"id": 1}]
    },
    "friends": [{"id": 0},{"id": 1}]
});

let mut selector = jsonpath::selector(json_obj);

let json = selector("$..friends[0]").unwrap();
let ret = json!([ {"id": 0}, {"id": 0} ]);
assert_eq!(json, ret);

let json = selector("$..friends[1]").unwrap();
let ret = json!([ {"id": 1}, {"id": 1} ]);
assert_eq!(json, ret);
```

### rust - examples

```rust
let json_obj = json!({
    "store": {
        "book": [
            {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
            },
            {
                "category": "fiction",
                "author": "Evelyn Waugh",
                "title": "Sword of Honour",
                "price": 12.99
            },
            {
                "category": "fiction",
                "author": "Herman Melville",
                "title": "Moby Dick",
                "isbn": "0-553-21311-3",
                "price": 8.99
            },
            {
                "category": "fiction",
                "author": "J. R. R. Tolkien",
                "title": "The Lord of the Rings",
                "isbn": "0-395-19395-8",
                "price": 22.99
            }
        ],
        "bicycle": {
            "color": "red",
            "price": 19.95
        }
    },
    "expensive": 10
});

let mut selector = jsonpath::selector(json_obj);

```

#### $.store.book[*].author
```rust
let json = selector("$.store.book[*].author").unwrap();
let ret = json!([
  "Nigel Rees",
  "Evelyn Waugh",
  "Herman Melville",
  "J. R. R. Tolkien"
]);
assert_eq!(json, ret);
```

#### $..author
```rust
let json = selector("$..author").unwrap();
let ret = json!([
  "Nigel Rees",
  "Evelyn Waugh",
  "Herman Melville",
  "J. R. R. Tolkien"
]);
assert_eq!(json, ret);
```

#### $.store.*
```rust
let json = selector("$.store.*").unwrap();
let ret = json!([
    [
        {
          "category": "reference",
          "author": "Nigel Rees",
          "title": "Sayings of the Century",
          "price": 8.95
        },
        {
          "category": "fiction",
          "author": "Evelyn Waugh",
          "title": "Sword of Honour",
          "price": 12.99
        },
        {
          "category": "fiction",
          "author": "Herman Melville",
          "title": "Moby Dick",
          "isbn": "0-553-21311-3",
          "price": 8.99
        },
        {
          "category": "fiction",
          "author": "J. R. R. Tolkien",
          "title": "The Lord of the Rings",
          "isbn": "0-395-19395-8",
          "price": 22.99
        }
    ],
    {
        "color": "red",
        "price": 19.95
    }
]);
assert_eq!(ret, json);
```

#### $.store..price
```rust
let json = selector("$.store..price").unwrap();
let ret = json!([8.95, 12.99, 8.99, 22.99, 19.95]);
assert_eq!(ret, json);
```

#### $..book[2]
```rust
let json = selector("$..book[2]").unwrap();
let ret = json!([{
    "category" : "fiction",
    "author" : "Herman Melville",
    "title" : "Moby Dick",
    "isbn" : "0-553-21311-3",
    "price" : 8.99
}]);
assert_eq!(ret, json);
```

#### $..book[-2]
```rust
let json = selector("$..book[-2]").unwrap();
let ret = json!([{
    "category" : "fiction",
    "author" : "Herman Melville",
    "title" : "Moby Dick",
    "isbn" : "0-553-21311-3",
    "price" : 8.99
 }]);
assert_eq!(ret, json);
```

#### $..book[0,1]
```rust
let json = selector("$..book[0,1]").unwrap();
let ret = json!([
  {
    "category": "reference",
    "author": "Nigel Rees",
    "title": "Sayings of the Century",
    "price": 8.95
  },
  {
    "category": "fiction",
    "author": "Evelyn Waugh",
    "title": "Sword of Honour",
    "price": 12.99
  }
]);
assert_eq!(ret, json);
```

#### $..book[:2]
```rust
let json = selector("$..book[:2]").unwrap();
let ret = json!([
  {
    "category": "reference",
    "author": "Nigel Rees",
    "title": "Sayings of the Century",
    "price": 8.95
  },
  {
    "category": "fiction",
    "author": "Evelyn Waugh",
    "title": "Sword of Honour",
    "price": 12.99
  }
]);
assert_eq!(ret, json);
```

#### $..book[2:]
```rust
let json = selector("$..book[2:]").unwrap();
let ret = json!([
  {
    "category": "fiction",
    "author": "Herman Melville",
    "title": "Moby Dick",
    "isbn": "0-553-21311-3",
    "price": 8.99
  },
  {
    "category": "fiction",
    "author": "J. R. R. Tolkien",
    "title": "The Lord of the Rings",
    "isbn": "0-395-19395-8",
    "price": 22.99
  }
]);
assert_eq!(ret, json);
```

#### $..book[?(@.isbn)]
```rust
let json = selector("$..book[?(@.isbn)]").unwrap();
let ret = json!([
  {
    "category": "fiction",
    "author": "Herman Melville",
    "title": "Moby Dick",
    "isbn": "0-553-21311-3",
    "price": 8.99
  },
  {
    "category": "fiction",
    "author": "J. R. R. Tolkien",
    "title": "The Lord of the Rings",
    "isbn": "0-395-19395-8",
    "price": 22.99
  }
]);
assert_eq!(ret, json);
```

#### $.store.book[?(@.price < 10)]
```rust
let json = selector("$.store.book[?(@.price < 10)]").unwrap();
let ret = json!([
  {
    "category": "reference",
    "author": "Nigel Rees",
    "title": "Sayings of the Century",
    "price": 8.95
  },
  {
    "category": "fiction",
    "author": "Herman Melville",
    "title": "Moby Dick",
    "isbn": "0-553-21311-3",
    "price": 8.99
  }
]);
assert_eq!(ret, json);
```

#### $..book[?((@.price == 12.99 || $.store.bicycle.price < @.price) || @.category == "reference")]
```rust
let json = selector(r#"$..book[
                    ?(
                        (@.price == 12.99 || $.store.bicycle.price < @.price) 
                        || @.category == "reference"
                     )]"#).unwrap();
let ret = json!([
  {
    "category": "fiction",
    "author": "Evelyn Waugh",
    "title": "Sword of Honour",
    "price": 12.99
  },
  {
    "category": "fiction",
    "author": "J. R. R. Tolkien",
    "title": "The Lord of the Rings",
    "isbn": "0-395-19395-8",
    "price": 22.99
  },
  {
    "category": "reference",
    "author": "Nigel Rees",
    "title": "Sayings of the Century",
    "price": 8.95
  }
]);
assert_eq!(ret, json);
```

## With AWS API Gateway

-

## Simple time check with [dchester/jsonpath](https://github.com/dchester/jsonpath)

`jsonpath` is dchester/jsonpath `jsonpath-wasm` is freestrings/jsonpath's compiled to webassembly

`jsonpath-wasm` is slow performance on Chrome browser and in NodeJS. not yet usable. :)

### Browser [Bench Demo](https://freestrings.github.io/jsonpath/bench)

```
'$..book[?(@.price<30 && @.category=="fiction")]' (loop 2,000)
```

#### Chrome: 72.0

> Something to wrong in chrome

```
jsonpath, 166
jsonpath-wasm- selector, 256
jsonpath-wasm- compile, 1168
jsonpath-wasm- compile-alloc, 645
jsonpath-wasm- select, 3224
jsonpath-wasm- select-alloc, 1427
```

#### Firefox: 65.0

> jsonpath-wasm is faster than jsonpath

```
jsonpath, 125
jsonpath-wasm- selector, 101
jsonpath-wasm- compile, 169
jsonpath-wasm- compile-alloc, 78
jsonpath-wasm- select, 186
jsonpath-wasm- select-alloc, 93
```

### NodeJs

* NodeJS: 11.0

> Rust > jsonpath > jsonpath-wasm


```bash
cd benches && ./bench_node_vs_rust.sh
$..book[?(@.price<30 && @.category==fiction)] (loop 100,000)

Rust: 

real	0m0.862s
user	0m0.862s
sys	0m0.000s

NodeJs - jsonpath module: 

real	0m3.667s
user	0m4.139s
sys	0m0.045s

NodeJs - jsonpath-wasm module - selector: 

real	0m5.331s
user	0m5.494s
sys	0m0.093s

NodeJs - jsonpath-wasm module - compile: 

real	0m8.665s
user	0m8.809s
sys	0m0.197s

NodeJs - jsonpath-wasm module - compile-alloc: 

real	0m4.014s
user	0m4.173s
sys	0m0.088s

NodeJs - jsonpath-wasm module - select:

real	0m9.843s
user	0m9.897s
sys	0m0.244s

NodeJs - jsonpath-wasm module - select-alloc:

real	0m5.212s
user	0m5.339s
sys	0m0.096s

```