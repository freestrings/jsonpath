# jsonpath-rs

[![Build Status](https://travis-ci.org/freestrings/jsonpath.svg?branch=master)](https://travis-ci.org/freestrings/jsonpath)

It is [JsonPath](https://goessner.net/articles/JsonPath/) implementation. The core implementation is written in Rust.

## Notice

Pre-built 바이너리는 제공하진 않고 소스를 컴파일해서 설치한다. 만약 Rust가 설치되지 않았다면 자동으로 설치된다.

Build from source instead of using pre-built binary, and if Rust is not installed, the latest version is automatically installed.

> Not yet tested in Windows

## APIs

* [jsonpath.Selector](#jsonpathselector)
* [jsonpath.select(json: string|object, jsonpath: string)](#json-stringobject-jsonpath-string)
* [jsonpath.compile(jsonpath: string)](#compilejsonpath-string)
* [jsonpath.selector(json: string|object)](#selectorjson-stringobject)
* [Simple time check](https://github.com/freestrings/jsonpath/wiki/Simple-timecheck-jsonpath-native)
* [Other Examples](https://github.com/freestrings/jsonpath/wiki/Javascript-examples)

### jsonpath.Selector

```javascript
let jsonObj = {
    "school": {
        "friends": [
            {"name": "친구1", "age": 20},
            {"name": "친구2", "age": 20}
        ]
    },
    "friends": [
        {"name": "친구3", "age": 30},
        {"name": "친구4"}
    ]
};

let selector = new jsonpath.Selector().value(jsonObj);

{
    let jsonObj = selector.path('$..[?(@.age >= 30)]').selectTo();
    let resultObj = [{"name": "친구3", "age": 30}];
    console.log(JSON.stringify(jsonObj) === JSON.stringify(resultObj));
}

{
    let jsonObj = selector.path('$..[?(@.age == 20)]').selectTo();
    let resultObj = [{"name": "친구1", "age": 20}, {"name": "친구2", "age": 20}];
    console.log(JSON.stringify(jsonObj) === JSON.stringify(resultObj));
}

{
    let jsonObj = selector.value({"friends": [ {"name": "친구5", "age": 20} ]}).selectTo();
    let resultObj = [{"name": "친구5", "age": 20}];
    console.log(JSON.stringify(jsonObj) === JSON.stringify(resultObj));
}
```

### jsonpath.select(json: string|object, jsonpath: string)

```javascript
let jsonObj = {
    "school": {
        "friends": [
            {"name": "친구1", "age": 20},
            {"name": "친구2", "age": 20}
        ]
    },
    "friends": [
        {"name": "친구3", "age": 30},
        {"name": "친구4"}
    ]
};

let ret = [
    {"name": "친구3", "age": 30},
    {"name": "친구1", "age": 20}
];


let selectAsString = jsonpath.select(JSON.stringify(jsonObj), '$..friends[0]');
let selectAsObj = jsonpath.select(jsonObj, '$..friends[0]');

console.log(
    JSON.stringify(ret) == JSON.stringify(selectAsString),
    JSON.stringify(ret) == JSON.stringify(selectAsObj)
);

// => true, true
```

### jsonpath.compile(jsonpath: string)

```javascript
let template = jsonpath.compile('$..friends[0]');

let jsonObj = {
    "school": {
        "friends": [
            {"name": "친구1", "age": 20},
            {"name": "친구2", "age": 20}
        ]
    },
    "friends": [
        {"name": "친구3", "age": 30},
        {"name": "친구4"}
    ]
};

let ret = [
    {"name": "친구3", "age": 30},
    {"name": "친구1", "age": 20}
];

let selectAsString = template(JSON.stringify(jsonObj));
let selectAsObj = template(jsonObj);

console.log(
    JSON.stringify(ret) == JSON.stringify(selectAsString),
    JSON.stringify(ret) == JSON.stringify(selectAsObj)
);

// => true, true

let jsonObj2 = {
    "school": {
        "friends": [
            {"name": "Millicent Norman"},
            {"name": "Vincent Cannon"}
        ]
    },
    "friends": [ {"age": 30}, {"age": 40} ]
};

let ret2 = [
    {"age": 30},
    {"name": "Millicent Norman"}
];

let selectAsString2 = template(JSON.stringify(jsonObj2));
let selectAsObj2 = template(jsonObj2);

console.log(
        JSON.stringify(ret2) == JSON.stringify(selectAsString2),
        JSON.stringify(ret2) == JSON.stringify(selectAsObj2)
);

// => true, true
```

### jsonpath.selector(json: string|object)

```javascript
let jsonObj = {
    "school": {
        "friends": [
            {"name": "친구1", "age": 20},
            {"name": "친구2", "age": 20}
        ]
    },
    "friends": [
        {"name": "친구3", "age": 30},
        {"name": "친구4"}
    ]
};

let ret1 = [
    {"name": "친구3", "age": 30},
    {"name": "친구1", "age": 20}
];

let ret2 = [
    {"name": "친구4"},
    {"name": "친구2", "age": 20}
];

let selector = jsonpath.selector(jsonObj);
// or as json string 
// let selector = jsonpath.selector(JSON.stringify(jsonObj));

let select1 = selector('$..friends[0]');
let select2 = selector('$..friends[1]');

console.log(
    JSON.stringify(ret1) == JSON.stringify(select1),
    JSON.stringify(ret2) == JSON.stringify(select2)
);

// => true, true
```
