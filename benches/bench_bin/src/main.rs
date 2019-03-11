extern crate jsonpath_lib as jsonpath;
extern crate serde_json;

use serde_json::Value;
use std::io::Read;

fn read_json(path: &str) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
}

fn main() {
    let string = read_json("../example.json");
    let json: Value = serde_json::from_str(string.as_str()).unwrap();
    let mut selector = jsonpath::selector(json);
    for _ in 1..100000 {
        let _ = selector(r#"$..book[?(@.price<30 && @.category=="fiction")]"#).unwrap();
    }
}
