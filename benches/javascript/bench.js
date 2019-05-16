let json = {
    'store': {
        'book': [
            {
                'category': 'reference',
                'author': 'Nigel Rees',
                'title': 'Sayings of the Century',
                'price': 8.95,
            },
            {
                'category': 'fiction',
                'author': 'Evelyn Waugh',
                'title': 'Sword of Honour',
                'price': 12.99,
            },
            {
                'category': 'fiction',
                'author': 'Herman Melville',
                'title': 'Moby Dick',
                'isbn': '0-553-21311-3',
                'price': 8.99,
            },
            {
                'category': 'fiction',
                'author': 'J. R. R. Tolkien',
                'title': 'The Lord of the Rings',
                'isbn': '0-395-19395-8',
                'price': 22.99,
            },
        ],
        'bicycle': {
            'color': 'red',
            'price': 19.95,
        },
    },
    'expensive': 10,
};
let jsonStr = JSON.stringify(json);

function getJson() {
    return JSON.parse(jsonStr);
}
const path = '$..book[?(@.price<30 && @.category=="fiction")]';
const jp = require('jsonpath');
const jpw = require('jsonpath-wasm');
const jpwRs = require('jsonpath-rs');

function jsonpath() {
    for (var i = 0; i < iter; i++) {
        let _ = jp.query(getJson(), path);
    }
}

function nativeCompile() {
    let template = jpwRs.compile(path);
    for (var i = 0; i < iter; i++) {
        let _ = template(JSON.stringify(json));
    }
}

function nativeSelector() {
    let selector = jpwRs.selector(getJson());
    for (var i = 0; i < iter; i++) {
        let _ = selector(path);
    }
}

function nativeSelect() {
    for (var i = 0; i < iter; i++) {
        let _ = jpwRs.select(JSON.stringify(json), path);
    }
}

function nativeSelectorClassMap() {
    let selector = new jpwRs.Selector();
    for (var i = 0; i < iter; i++) {
        let _ = selector.path(path).value(jsonStr).map((v) => v).get();
    }
}

function wasmSelector() {
    let selector = jpw.selector(getJson());
    for (var i = 0; i < iter; i++) {
        let _ = selector(path);
    }
}

function wasmCompile() {
    let template = jpw.compile(path);
    for (var i = 0; i < iter; i++) {
        let _ = template(getJson());
    }
}

function wasmCompileAlloc() {
    let ptr = jpw.allocJson(getJson());
    if (ptr == 0) {
        console.error('Invalid pointer');
        return;
    }

    try {
        let template = jpw.compile(path);
        for (var i = 0; i < iter; i++) {
            let _ = template(ptr);
        }
    } finally {
        jpw.deallocJson(ptr);
    }
}

function wasmSelect() {
    for (var i = 0; i < iter; i++) {
        let _ = jpw.select(getJson(), path);
    }
}

function wasmSelectAlloc() {
    let ptr = jpw.allocJson(getJson());
    if (ptr == 0) {
        console.error('Invalid pointer');
        return;
    }

    try {
        for (var i = 0; i < iter; i++) {
            let _ = jpw.select(ptr, path);
        }
    } finally {
        jpw.deallocJson(ptr);
    }
}

function wasmSelectorClass() {
    let selector = new jpw.Selector();
    for (var i = 0; i < iter; i++) {
        selector.path(path);
        selector.value(jsonStr);
        let _ = selector.selectToStr();
    }
}

function wasmSelectorClassMap() {
    let selector = new jpw.Selector();
    for (var i = 0; i < iter; i++) {
        selector.path(path);
        selector.value(jsonStr);
        let _1 = selector.map((v) => v);
        let _2 = selector.get();
    }
}

const functionName = process.argv[2];
const iter = parseInt(process.argv[3], 10);
eval(functionName + "()");