# jsonpath-lib

[![Build Status](https://travis-ci.org/freestrings/jsonpath.svg?branch=master)](https://travis-ci.org/freestrings/jsonpath)

`Rust` 버전 [JsonPath](https://goessner.net/articles/JsonPath/) 구현이다. Rust 구현과 동일한 기능을 `Webassembly` 로 제공하는 것 또한 목적이다.

The `Rust` version is a [JsonPath](https://goessner.net/articles/JsonPath/) implementation. It is also aimed to provide the same functionality as `Webassembly` in Rust implementation.

## 왜?

To enjoy Rust!

## 목차

[With Javascript (Webassembly)](#with-javascript-webassembly)

- [jsonpath-wasm library](#jsonpath-wasm-library)
- [javascript - jsonpath.read(json: string|object, jsonpath: string)](#javascript---jsonpathreadjson-stringobject-jsonpath-string)
- [javascript - jsonpath.compile(jsonpath: string)](#javascript---jsonpathcompilejsonpath-string)
- [javascript - jsonpath.reader(json: string|object)](#javascript---jsonpathreaderjson-stringobject)
- [javascript - examples](#javascript---examples)

[With Rust (as library)](#with-rust-as-library)

- [jsonpath_lib library](#jsonpath_lib-library)
- [rust - jsonpath::read(json: serde_json::value::Value, jsonpath: &str)](#rust---jsonpathreadjson-serde_jsonvaluevalue-jsonpath-str)
- [rust - jsonpath::compile(jsonpath: &str)](#rust---jsonpathcompilejsonpath-str)
- [rust - jsonpath::reader(json: serde_json::value::Value)](#rust---jsonpathreaderjson-serde_jsonvaluevalue)
- [rust - examples](#rust---examples)

[With AWS API Gateway](#with-aws-api-gateway)

[Benchmark](#benchmark)

## With Javascript (WebAssembly)

### jsonpath-wasm library

*(not yet published `jsonpath-wasm`)*
```javascript
// browser
import * as jsonpath from "jsonpath-wasm";
// nodejs
let jsonpath = require('jsonpath-wasm');
```

### javascript - jsonpath.read(json: string|object, jsonpath: string)

```javascript
let jsonObj = {
   "school": {
       "friends": [{"id": 0}, {"id": 1}]
   },
   "friends": [{"id": 0}, {"id": 1}]
};
let ret = [{"id": 0}, {"id": 0}];

let a = jsonpath.read(JSON.stringify(jsonObj), "$..friends[0]");
let b = jsonpath.read(jsonObj, "$..friends[0]");
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

### javascript - jsonpath.reader(json: string|object)

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
let reader = jsonpath.reader(jsonObj);
console.log(JSON.stringify(reader("$..friends[0]")) == ret1);
console.log(JSON.stringify(reader("$..friends[1]")) == ret2);

// 2. read as json string
let reader2 = jsonpath.reader(JSON.stringify(jsonObj));
console.log(JSON.stringify(reader2("$..friends[0]")) == ret1);
console.log(JSON.stringify(reader2("$..friends[1]")) == ret2);
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

### rust - jsonpath::read(json: serde_json::value::Value, jsonpath: &str)

```rust
let json_obj = json!({
    "school": {
        "friends": [{"id": 0}, {"id": 1}]
    },
    "friends": [{"id": 0}, {"id": 1}]
});
let json = jsonpath::read(json_obj, "$..friends[0]").unwrap();
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

### rust - jsonpath::reader(json: serde_json::value::Value)

```rust
let json_obj = json!({
    "school": {
        "friends": [{"id": 0}, {"id": 1}]
    },
    "friends": [{"id": 0},{"id": 1}]
});

let mut reader = jsonpath::reader(json_obj);

let json = reader("$..friends[0]").unwrap();
let ret = json!([ {"id": 0}, {"id": 0} ]);
assert_eq!(json, ret);

let json = reader("$..friends[1]").unwrap();
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

let mut reader = jsonpath::reader(json_obj);

```

#### $.store.book[*].author
```rust
let json = reader("$.store.book[*].author").unwrap();
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
let json = reader("$..author").unwrap();
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
let json = reader("$.store.*").unwrap();
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
let json = reader("$.store..price").unwrap();
let ret = json!([8.95, 12.99, 8.99, 22.99, 19.95]);
assert_eq!(ret, json);
```

#### $..book[2]
```rust
let json = reader("$..book[2]").unwrap();
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
let json = reader("$..book[-2]").unwrap();
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
let json = reader("$..book[0,1]").unwrap();
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
let json = reader("$..book[:2]").unwrap();
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
let json = reader("$..book[2:]").unwrap();
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
let json = reader("$..book[?(@.isbn)]").unwrap();
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
let json = reader("$.store.book[?(@.price < 10)]").unwrap();
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
let json = reader(r#"$..book[
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

## Benchmark

