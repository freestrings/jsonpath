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
use jsonpath::ref_value::model::RefValueWrapper;

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

fn _as_value(b: &mut Bencher, index: usize) {
    let json = get_json();
    b.iter(move || {
        for _ in 1..100 {
            let _ = jsonpath::select(&json, get_path(index));
        }
    });
}

fn _as_ref_value(b: &mut Bencher, index: usize) {
    let ref json = get_json();
    let rv: RefValueWrapper = json.into();
    b.iter(move || {
        for _ in 1..100 {
            let mut selector = jsonpath::Selector::new();
            let _ = selector.path(get_path(index));
            let _ = selector.value_from_ref_value(rv.clone());
            let _ = selector.select_as_value();
        }
    });
}

macro_rules! example_val {
    ($name:ident, $i:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) { _as_value(b, $i); }
    };
}

macro_rules! example_val_ref {
    ($name:ident, $i:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) { _as_ref_value(b, $i); }
    };
}

example_val!(example_val_0, 0);
example_val!(example_val_1, 1);
example_val!(example_val_2, 2);
example_val!(example_val_3, 3);
example_val!(example_val_4, 4);
example_val!(example_val_5, 5);
example_val!(example_val_6, 6);
example_val!(example_val_7, 7);
example_val!(example_val_8, 8);
example_val!(example_val_9, 9);
example_val!(example_val_10, 10);
example_val!(example_val_11, 11);
example_val!(example_val_12, 12);
example_val!(example_val_13, 13);
example_val!(example_val_14, 14);

example_val_ref!(example_val_ref_0, 0);
example_val_ref!(example_val_ref_1, 1);
example_val_ref!(example_val_ref_2, 2);
example_val_ref!(example_val_ref_3, 3);
example_val_ref!(example_val_ref_4, 4);
example_val_ref!(example_val_ref_5, 5);
example_val_ref!(example_val_ref_6, 6);
example_val_ref!(example_val_ref_7, 7);
example_val_ref!(example_val_ref_8, 8);
example_val_ref!(example_val_ref_9, 9);
example_val_ref!(example_val_ref_10, 10);
example_val_ref!(example_val_ref_11, 11);
example_val_ref!(example_val_ref_12, 12);
example_val_ref!(example_val_ref_13, 13);
example_val_ref!(example_val_ref_14, 14);