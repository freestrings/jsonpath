# jsonpath-rs

[![Build Status](https://travis-ci.org/freestrings/jsonpath.svg?branch=master)](https://travis-ci.org/freestrings/jsonpath)

It is native-addon of [jsonpath_lib](https://github.com/freestrings/jsonpath) that is [JsonPath](https://goessner.net/articles/JsonPath/) engine written in Rust.

## Notice

Pre-built 바이너리는 제공하진 않고 소스를 컴파일해서 설치한다. 만약 Rust가 설치되지 않았다면 자동으로 설치된다.

Build from source instead of using pre-built binary, and if Rust is not installed, the latest version is automatically installed.

> Not yet tested in Windows.

> Supported node version is under v12.0.

## APIs

<details><summary><b>npm package</b></summary>

```javascript
const jsonpath = require('jsonpath-rs');
```

</details>

<details><summary><b>Javascript - jsonpath.Selector class</b></summary>

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

let selector = new jsonpath.Selector()
    .path('$..friends[0]')
    .value(jsonObj);

let retObj = selector.select();

console.log(JSON.stringify(ret) == JSON.stringify(retObj));

// => true
```

</details>

<details><summary><b>Javascript - jsonpath.SelectorMut class</b></summary>

빌더 패턴 제약은 `Selector class`와 동일하다.

```javascript
let jsonObj = {
    'school': {
        'friends': [
            {'name': '친구1', 'age': 20},
            {'name': '친구2', 'age': 20},
        ],
    },
    'friends': [
        {'name': '친구3', 'age': 30},
        {'name': '친구4'},
    ],
};

let selector = new jsonpath.SelectorMut();
selector.path('$..[?(@.age == 20)]');

{
    selector.value(jsonObj);
    selector.deleteValue();

    let resultObj = {
        'school': {'friends': [null, null]},
        'friends': [
            {'name': '친구3', 'age': 30},
            {'name': '친구4'},
        ],
    };
    console.log(JSON.stringify(selector.take()) !== JSON.stringify(resultObj));
    
    // => true
}

{
    selector.value(jsonObj);
    selector.replaceWith((v) => {
        v.age = v.age * 2;
        return v;
    });

    let resultObj = {
        'school': {
            'friends': [
                {'name': '친구1', 'age': 40},
                {'name': '친구2', 'age': 40},
            ],
        },
        'friends': [
            {'name': '친구3', 'age': 30},
            {'name': '친구4'},
        ],
    };
    console.log(JSON.stringify(selector.take()) !== JSON.stringify(resultObj));
    
    // => true
}
```

</details>

<details><summary><b>Javascript - jsonpath.select(json: string|object, jsonpath: string)</b></summary>

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

</details>

<details><summary><b>Javascript - jsonpath.compile(jsonpath: string)</b></summary>

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
    
</details>

<details><summary><b>Javascript - jsonpath.selector(json: string|object)</b></summary>

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

</details>

<details><summary><b>Javascript - jsonpath.deleteValue(json: string|object, path: string)</b></summary>

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

let _1 = jsonpath.deleteValue(jsonObj, '$..friends[0]');
let result = jsonpath.deleteValue(_1, '$..friends[1]');

console.log(JSON.stringify(result) !== JSON.stringify({
    "school": { "friends": [null, null]},
    "friends": [null, null]
}));

// => true

```

</details>

<details><summary><b>Javascript - jsonpath.replaceWith(json: string|object, path: string, fun: function(json: object) => json: object</b></summary>

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

let result = jsonpath.replaceWith(jsonObj, '$..friends[0]', (v) => {
    v.age = v.age * 2;
    return v;
});

console.log(JSON.stringify(result) === JSON.stringify({
    "school": {
        "friends": [
            {"name": "친구1", "age": 40},
            {"name": "친구2", "age": 20}
        ]
    },
    "friends": [
        {"name": "친구3", "age": 60},
        {"name": "친구4"}
    ]
}));

// => true

```

</details>

[Javascript - Other Examples](https://github.com/freestrings/jsonpath/wiki/Javascript-examples)