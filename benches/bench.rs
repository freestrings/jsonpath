#![feature(test)]
extern crate jsonpath;
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
fn bench_a(b: &mut Bencher) {
    let string = read_json("./benches/data_array.json");
    let v: Value = serde_json::from_str(string.as_str()).unwrap();
    b.iter(move || {
        for _ in 1..1000 {
            let _ = v.clone();
        }
    });
}

#[bench]
fn bench_b(b: &mut Bencher) {
    let string = read_json("./benches/data_array.json");
    b.iter(move || {
        for _ in 1..1000 {
            let _: Value = serde_json::from_str(string.as_str()).unwrap();
        }
    });
}