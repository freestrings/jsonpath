# jsonpath-wasm

[![Build Status](https://travis-ci.org/freestrings/jsonpath.svg?branch=master)](https://travis-ci.org/freestrings/jsonpath)

It is Webassembly version of [jsonpath_lib](https://github.com/freestrings/jsonpath) that is [JsonPath](https://goessner.net/articles/JsonPath/) engine written in Rust. 

## APIs

* [jsonpath.Selector](#jsonpathselector)
* [jsonpath.select(json: string|object, jsonpath: string)](#jsonpathselectjson-stringobject-jsonpath-string)
* [jsonpath.compile(jsonpath: string)](#jsonpathcompilejsonpath-string)
* [jsonpath.selector(json: string|object)](#jsonpathselectorjson-stringobject)
* [Other Examples](https://github.com/freestrings/jsonpath/wiki/Javascript-examples)

### jsonpath.Selector

> Selector's selectTo function is deprecated since 0.1.3

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
    let jsonObj = selector.path('$..[?(@.age >= 30)]').selectAs();
    let resultObj = [{"name": "친구3", "age": 30}];
    console.log(JSON.stringify(jsonObj) === JSON.stringify(resultObj));
}

{
    let jsonObj = selector.path('$..[?(@.age == 20)]').selectAs();
    let resultObj = [{"name": "친구1", "age": 20}, {"name": "친구2", "age": 20}];
    console.log(JSON.stringify(jsonObj) === JSON.stringify(resultObj));
}

{
    let jsonObj = selector.value({"friends": [ {"name": "친구5", "age": 20} ]}).selectAs();
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
