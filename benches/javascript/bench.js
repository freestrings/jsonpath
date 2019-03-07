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

const jp = require('jsonpath');
const jpw = require('jsonpath-wasm');
const Benchmark = require('benchmark');

function compareJsonpath(path) {
    let r1 = jp.query(json, path);
    let r2 = jpw.read(json, path);

    let template = jpw.compile(path);

    var suite = new Benchmark.Suite;

    suite.add('jp', function() {
        jp.query(json, path);
    })
    .add('jpw', function() {
        template(json);
    })
    .on('cycle', function(event) {
        console.log(String(event.target));
    })
    .on('complete', function() {
        console.log('Fastest is ' + this.filter('fastest').map('name'));
        console.log('Slowest is ' + this.filter('slowest').map('name'));
    })
    .run({ 'async': true });
}

function compareEmptyFunction() {
    var suite = new Benchmark.Suite;

    suite.add('js', function() {
    })
    .add('rust', function() {
        jpw.testa();
    })
    .on('cycle', function(event) {
        console.log(String(event.target));
    })
    .on('complete', function() {
        console.log('Fastest is ' + this.filter('fastest').map('name'));
        console.log('Slowest is ' + this.filter('slowest').map('name'));
    })
    .run({});
}

function jsonpathOnly() {
    for(var i = 0; i < 100000 ; i++) {
        let _ = jp.query(json, '$..book[?(@.price<30 && @.category==\"fiction\")]');
    }
}

if(process.argv.length < 3) {
    let functions = ['', 'compareJsonpath', 'compareEmptyFunction', 'jsonpathOnly'];
    console.log("node bench.js", functions.join("\n\t|"));
    return;
}

let functionName = process.argv[2];

switch (functionName) {
    case 'compareJsonpath':
        compareJsonpath('$..book[?(@.price<30 && @.category==\"fiction\")]');
        break;
    case 'compareEmptyFunction':
        compareEmptyFunction();
        break;
    default:
        jsonpathOnly();
}