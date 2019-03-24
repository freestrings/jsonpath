# jsonpath-lib

[![Build Status](https://travis-ci.org/freestrings/jsonpath.svg?branch=master)](https://travis-ci.org/freestrings/jsonpath)
![crates.io](https://img.shields.io/crates/v/jsonpath_lib.svg)

`Rust` 버전 [JsonPath](https://goessner.net/articles/JsonPath/) 구현이다. `Webassembly`와 `Javascript`에서도 역시 동일한 API 인터페이스를 제공 한다. 

It is an implementation of the `Rust` version [JsonPath] (https://goessner.net/articles/JsonPath/). `Webassembly` and` Javascript` also provide the same API interface.

- [Webassembly Demo](https://freestrings.github.io/jsonpath/)
- [Rust documentation](https://docs.rs/jsonpath_lib/0.1.6/jsonpath_lib)

## 왜?

To enjoy Rust!

## 목차

[With Javascript](#with-javascript)

- [jsonpath-wasm library](#jsonpath-wasm-library)
- [jsonpath-rs library](#jsonpath-rs-library-only-nodejs)
- [javascript - jsonpath.select(json: string|object, jsonpath: string)](#javascript---jsonpathselectjson-stringobject-jsonpath-string)
- [javascript - jsonpath.compile(jsonpath: string)](#javascript---jsonpathcompilejsonpath-string)
- [javascript - jsonpath.selector(json: string|object)](#javascript---jsonpathselectorjson-stringobject)
- [javascript - alloc_json, dealloc_json](#javascript---alloc_json-dealloc_json)
- [javascript-wasm - examples](https://github.com/freestrings/jsonpath/wiki/javascript-wasm-examples)

[With Rust (as library)](#with-rust-as-library)

- [jsonpath_lib library](#jsonpath_lib-library)
- [rust - jsonpath::select(json: &serde_json::value::Value, jsonpath: &str)](#rust---jsonpathselectjson-serde_jsonvaluevalue-jsonpath-str)
- [rust - jsonpath::select_as_str(json_str: &str, jsonpath: &str)](#rust---jsonpathselect_as_strjson-str-jsonpath-str)
- [rust - jsonpath::select_as\<T\>(json_str: &str, jsonpath: &str)](#rust---jsonpathselect_astjson-str-jsonpath-str)
- [rust - jsonpath::compile(jsonpath: &str)](#rust---jsonpathcompilejsonpath-str)
- [rust - jsonpath::selector(json: &serde_json::value::Value)](#rust---jsonpathselectorjson-serde_jsonvaluevalue)
- [rust - examples](https://github.com/freestrings/jsonpath/wiki/rust-examples)

[With AWS API Gateway](#)

[Simple time check - webassembly](https://github.com/freestrings/jsonpath/wiki/Simple-timecheck---jsonpath-wasm)

[Simple time check - native addon for NodeJs](https://github.com/freestrings/jsonpath/wiki/Simple-timecheck-jsonpath-native)

## With Javascript

### jsonpath-wasm library

*(not yet published `jsonpath-wasm`)*
```javascript
// browser
import * as jsonpath from "jsonpath-wasm";
// NodeJs
const jsonpath = require('jsonpath-wasm');
```

### jsonpath-rs library (Only NodeJS)

`jsonpath-rs` is native addon for NodeJs

*(not yet published `jsonpath-rs`)*
```javascript
const jsonpath = require('jsonpath-rs');
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

*(in `jsonpath-rs` not yet supported)*

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

## With Rust (as library)

### jsonpath_lib library

```rust
extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;
```

### rust - jsonpath::select(json: &serde_json::value::Value, jsonpath: &str)

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

### rust - jsonpath::select_as_str(json: &str, jsonpath: &str)

```rust
let ret = jsonpath::select_as_str(r#"{
    "school": { "friends": [{"id": 0}, {"id": 1}] },
    "friends": [{"id": 0}, {"id": 1}]
}"#, "$..friends[0]").unwrap();
assert_eq!(ret, r#"[{"id":0},{"id":0}]"#);
```

### rust - jsonpath::select_as\<T\>(json: &str, jsonpath: &str)

```rust
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}
let ret: Person = jsonpath::select_as(r#"
{
    "person":
        {
            "name": "Doe John",
            "age": 44,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }
}
"#, "$.person").unwrap();
let person = Person {
    name: "Doe John".to_string(),
    age: 44,
    phones: vec!["+44 1234567".to_string(), "+44 2345678".to_string()],
};
assert_eq!(person, ret);
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

let json = template(&json_obj).unwrap();
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

### rust - jsonpath::selector(&json: serde_json::value::Value)

```rust
let json_obj = json!({
    "school": {
        "friends": [{"id": 0}, {"id": 1}]
    },
    "friends": [{"id": 0},{"id": 1}]
});

let mut selector = jsonpath::selector(&json_obj);

let json = selector("$..friends[0]").unwrap();
let ret = json!([ {"id": 0}, {"id": 0} ]);
assert_eq!(json, ret);

let json = selector("$..friends[1]").unwrap();
let ret = json!([ {"id": 1}, {"id": 1} ]);
assert_eq!(json, ret);
```
