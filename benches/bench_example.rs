#![feature(test)]

extern crate bencher;
extern crate indexmap;
extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate test;

use std::io::Read;

use serde_json::Value;

use self::test::Bencher;

fn read_json(path: &str) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
}

fn get_string() -> String {
    read_json("./benches/example.json")
}

fn get_json() -> Value {
    let string = get_string();
    serde_json::from_str(string.as_str()).unwrap()
}

fn get_path(i: usize) -> &'static str {
    let paths = vec![
        "$.store.book[*].author",   //0
        "$..author",    //1
        "$.store.*",    //2
        "$.store..price",   //3
        "$..book[2]",   //4
        "$..book[-2]",  //5
        "$..book[0,1]", //6
        "$..book[:2]",  //7
        "$..book[1:2]", //8
        "$..book[-2:]", //9
        "$..book[2:]",  //10
        "$..book[?(@.isbn)]",   //11
        "$.store.book[?(@.price < 10)]",    //12
        "$..*", //13
        "$..book[ ?( (@.price < 13 || $.store.bicycle.price < @.price) && @.price <=10 ) ]" //14
    ];
    paths[i]
}

macro_rules! example {
    ($name:ident, $i:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let json = get_json();
            b.iter(move || {
                for _ in 1..100 {
                    let _ = jsonpath::select(&json, get_path($i));
                }
            });
        }
    };
}

example!(example0, 0);
example!(example1, 1);
example!(example2, 2);
example!(example3, 3);
example!(example4, 4);
example!(example5, 5);
example!(example6, 6);
example!(example7, 7);
example!(example8, 8);
example!(example9, 9);
example!(example10, 10);
example!(example11, 11);
example!(example12, 12);
example!(example13, 13);
example!(example14, 14);
