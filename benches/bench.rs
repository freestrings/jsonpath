#![feature(test)]
extern crate jsonpath_lib as jsonpath;
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

#[bench]
fn bench_selector(b: &mut Bencher) {
    let string = read_json("./benches/example.json");
    let path = r#"$..book[?(@.price<30 && @.category=="fiction")]"#;
    let json: Value = serde_json::from_str(string.as_str()).unwrap();
    let mut selector = jsonpath::selector(&json);
    b.iter(move || {
        for _ in 1..1000 {
            let _ = selector(path).unwrap();
        }
    });
}

#[bench]
fn bench_select(b: &mut Bencher) {
    let string = read_json("./benches/example.json");
    let path = r#"$..book[?(@.price<30 && @.category=="fiction")]"#;
    let json: Value = serde_json::from_str(string.as_str()).unwrap();
    b.iter(move || {
        for _ in 1..1000 {
            let _ = jsonpath::select(&json, path).unwrap();
        }
    });
}

#[bench]
fn bench_compile(b: &mut Bencher) {
    let string = read_json("./benches/example.json");
    let path = r#"$..book[?(@.price<30 && @.category=="fiction")]"#;
    let json: Value = serde_json::from_str(string.as_str()).unwrap();
    let mut template = jsonpath::compile(path);
    b.iter(move || {
        for _ in 1..1000 {
            let _ = template(&json).unwrap();
        }
    });
}