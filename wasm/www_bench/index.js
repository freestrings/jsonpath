import * as jpw from "@browser/jsonpath-wasm";
import * as jp from "jsonpath/jsonpath.js";

function run(message, iter, cb) {
    let d = Date.now();
    for (let i = 0; i < iter; i++) {
        cb();
    }
    msg([message, Date.now() - d].join(", "));
}

function msg(msg) {
    console.log(msg);
    let div = document.createElement("div");
    div.innerText = msg;
    document.body.appendChild(div);
}

let json = {
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
};

setTimeout(function() {
    let path = '$..book[?(@.price<30 && @.category=="fiction")]';
    let template = jpw.compile(path);
    let reader = jpw.reader(json);
    run('jsonpath', 1000, function() { jp.query(json, path) });
    run('jsonpath-wasm- reader', 1000, function() { reader(path) });
    run('jsonpath-wasm- compile', 1000, function() { template(json) });
    run('jsonpath-wasm- read', 1000, function() { jpw.read(json, path) });
}, 0);
