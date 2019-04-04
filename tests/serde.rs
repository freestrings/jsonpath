extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;

use std::io::Read;

use serde_json::Value;
use jsonpath::ref_value::model::{RefValue, RefValueWrapper};

fn read_json(path: &str) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
}

#[test]
fn de() {
    let json_str = read_json("./benches/example.json");
    // RefValue -> Value
    let ref_value: RefValue = serde_json::from_str(json_str.as_str()).unwrap();
    let ref value_wrapper: RefValueWrapper = ref_value.into();
    let value: Value = value_wrapper.into();

    // Value
    let json: Value = serde_json::from_str(json_str.as_str()).unwrap();
    assert_eq!(value, json);
}

#[test]
fn ser() {
    let json_str = read_json("./benches/example.json");
    let ref_value: RefValue = serde_json::from_str(json_str.as_str()).unwrap();
    let ref_value_str = serde_json::to_string(&ref_value).unwrap();

    let json: Value = serde_json::from_str(json_str.as_str()).unwrap();
    let json_str = serde_json::to_string(&json).unwrap();
    assert_eq!(ref_value_str, json_str);
}