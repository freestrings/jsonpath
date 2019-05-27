#![feature(test)]
extern crate bencher;
extern crate indexmap;
extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate test;

use std::io::Read;

use serde::Serialize;
use serde::Deserialize;
use serde_json::Value;

use self::test::Bencher;
use jsonpath::ref_value::model::{RefValue, RefValueWrapper};
use jsonpath::ref_value::ser::RefValueSerializer;

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

fn get_path() -> &'static str {
    r#"$..book[?(@.price<30 && @.category=="fiction")]"#
}

#[bench]
fn bench_selector(b: &mut Bencher) {
    let json = get_json();
    let mut selector = jsonpath::selector(&json);
    b.iter(move || {
        for _ in 1..100 {
            let _ = selector(get_path()).unwrap();
        }
    });
}

#[bench]
fn bench_selector_as(b: &mut Bencher) {
    let json = get_json();
    let mut selector = jsonpath::selector_as::<Value>(&json);
    b.iter(move || {
        for _ in 1..100 {
            let _ = selector(get_path()).unwrap();
        }
    });
}

#[bench]
fn bench_select_val(b: &mut Bencher) {
    let json = get_json();
    b.iter(move || {
        for _ in 1..100 {
            let _ = jsonpath::select(&json, get_path()).unwrap();
        }
    });
}

#[bench]
fn bench_select_as_str(b: &mut Bencher) {
    let json = get_string();
    b.iter(move || {
        for _ in 1..100 {
            let _ = jsonpath::select_as_str(&json, get_path()).unwrap();
        }
    });
}

#[bench]
fn bench_compile(b: &mut Bencher) {
    let json = get_json();
    let mut template = jsonpath::compile(get_path());
    b.iter(move || {
        for _ in 1..100 {
            let _ = template(&json).unwrap();
        }
    });
}

#[bench]
fn bench_select_as(b: &mut Bencher) {
    let json = get_string();

    #[derive(Deserialize, PartialEq, Debug)]
    struct Book {
        category: String,
        author: String,
        title: String,
        price: f64,
    }

    b.iter(move || {
        for _ in 1..100 {
            let _: Book = jsonpath::select_as(&json, r#"$..book[?(@.price<30 && @.category=="fiction")][0]"#).unwrap();
        }
    });
}

#[bench]
fn refval_de(b: &mut Bencher) {
    let json = get_json();
    b.iter(move || {
        for _ in 1..100 {
            let _ = RefValue::deserialize(&json).unwrap();
        }
    });
}

#[bench]
fn refval_se(b: &mut Bencher) {
    let json = get_json();
    b.iter(move || {
        for _ in 1..100 {
            let _ = &json.serialize(RefValueSerializer).unwrap();
        }
    });
}

#[bench]
fn refval_refcopy(b: &mut Bencher) {
    use std::ops::Deref;

    let json = get_json();
    let ref_json: RefValue = json.serialize(RefValueSerializer).unwrap();
    let store = ref_json.get("store".to_string()).unwrap();
    let book = store.get("book".to_string()).unwrap();

    b.iter(move || {
        for _ in 1..100 {
            if let RefValue::Array(vec) = book.deref() {
                let _: Vec<RefValueWrapper> = vec.iter().map(|v| v.clone()).collect();
            }
        }
    });
}

#[bench]
fn refval_copy(b: &mut Bencher) {

    let json = get_json();
    let store = json.get("store".to_string()).unwrap();
    let book = store.get("book".to_string()).unwrap();

    b.iter(move || {
        for _ in 1..100 {
            if let Value::Array(vec) = book {
                let _: Vec<Value> = vec.iter().map(|v| v.clone()).collect();
            }
        }
    });
}

#[bench]
fn value_clone(b: &mut Bencher) {
    let json = get_json();
    b.iter(move || {
        for _ in 1..100 {
            let _ = json.clone();
        }
    });
}