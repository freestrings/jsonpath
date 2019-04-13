const jsonpath = require('jsonpath-rs');

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

let selector = new jsonpath.Selector();
selector.path('$..friends[0]');
selector.value(jsonObj);

let selectToObj = selector.selectTo();
let selectToString = selector.selectToStr();

console.log(
    JSON.stringify(ret) == JSON.stringify(selectToObj),
    JSON.stringify(ret) == selectToString
);
