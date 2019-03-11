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

const jp = require('jsonpath');
const jpw = require('@nodejs/jsonpath-wasm');
const iter = 100000;

function jsonpath() {
    for (var i = 0; i < iter; i++) {
        let _ = jp.query(json, '$..book[?(@.price<30 && @.category=="fiction")]');
    }
}

function wasmSelector() {
    let selector = jpw.selector(json);
    for (var i = 0; i < iter; i++) {
        let _ = selector('$..book[?(@.price<30 && @.category=="fiction")]');
    }
}

function wasmCompile() {
    let template = jpw.compile('$..book[?(@.price<30 && @.category=="fiction")]');
    for (var i = 0; i < iter; i++) {
        let _ = template(json);
    }
}

function wasmCompileAlloc() {
    let ptr = jpw.alloc_json(json);
    if (ptr == 0) {
        console.error('Invalid pointer');
        return;
    }

    try {
        let template = jpw.compile('$..book[?(@.price<30 && @.category=="fiction")]');
        for (var i = 0; i < iter; i++) {
            let _ = template(ptr);
        }
    } finally {
        jpw.dealloc_json(ptr);
    }
}

function wasmSelect() {
    for (var i = 0; i < iter; i++) {
        let _ = jpw.select(json, '$..book[?(@.price<30 && @.category=="fiction")]');
    }
}

function wasmSelectAlloc() {
    let ptr = jpw.alloc_json(json);
    if (ptr == 0) {
        console.error('Invalid pointer');
        return;
    }

    try {
        for (var i = 0; i < iter; i++) {
            let _ = jpw.select(ptr, '$..book[?(@.price<30 && @.category=="fiction")]');
        }
    } finally {
        jpw.dealloc_json(ptr);
    }
}

let functionName = process.argv[2];

switch (functionName) {
    case 'jsonpath':
        jsonpath();
        break;
    case 'wasmSelector':
        wasmSelector();
        break;
    case 'wasmCompile':
        wasmCompile();
        break;
    case 'wasmSelect':
        wasmSelect();
        break;
    case 'wasmCompileAlloc':
        wasmCompileAlloc();
        break;
    case 'wasmSelectAlloc':
        wasmSelectAlloc();
    default:
        console.error('Invalid function name');
}
