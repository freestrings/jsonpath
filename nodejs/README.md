# jsonpath-rs

[![Build Status](https://travis-ci.org/freestrings/jsonpath.svg?branch=master)](https://travis-ci.org/freestrings/jsonpath)
![npm](https://img.shields.io/npm/dt/jsonpath-rs.svg?label=%60jsonpath-rs%60%20npm%20downloads)

It is [JsonPath](https://goessner.net/articles/JsonPath/) implementation. The core implementation is written in Rust-lang.

## Install

```bash
# package.json
"dependencies": {
    "node-pre-gyp": "0.12.0",
    "jsonpath-rs": "0.1"
}
```

## 목차

* [jsonpath.select(json: string|object, jsonpath: string)](#json-stringobject-jsonpath-string)
* [jsonpath.compile(jsonpath: string)](#compilejsonpath-string)
* [jsonpath.selector(json: string|object)](#selectorjson-stringobject)
* [Simple time check](https://github.com/freestrings/jsonpath/wiki/Simple-timecheck-jsonpath-native)
* [Other Examples](https://github.com/freestrings/jsonpath/wiki/Javascript-examples)

### jsonpath.select(json: string|object, jsonpath: string)

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

### jsonpath.compile(jsonpath: string)

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

### jsonpath.selector(json: string|object)

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
