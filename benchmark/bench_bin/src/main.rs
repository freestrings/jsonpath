extern crate jsonpath_lib as jsonpath;
extern crate serde_json;

use serde_json::Value;
use std::io::Read;

use std::env;

fn read_json(path: &str) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
}

fn main() {
    let string = read_json("../example.json");
    let json: Value = serde_json::from_str(string.as_str()).unwrap();
    let path = r#"$..book[?(@.price<30 && @.category=="fiction")]"#;

    let args: Vec<String> = env::args().collect();
    let iter = match &args[2].as_str().parse::<usize>() {
        Ok(iter) => *iter,
        _ => 100000
    };

    match &args[1].as_str() {
        &"compile" => {
            let mut template = jsonpath::compile(path);
            for _ in 1..iter {
                let _ = template(&json).unwrap();
            }
        }
        &"selector" => {
            let mut selector = jsonpath::selector(&json);
            for _ in 1..iter {
                let _ = selector(path).unwrap();
            }
        }
        &"select" => {
            let json: Value = serde_json::from_str(string.as_str()).unwrap();
            for _ in 1..iter {
                let _ = jsonpath::select(&json, path).unwrap();
            }
        }
        _ => panic!("Invalid argument")
    }
}
