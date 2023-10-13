extern crate env_logger;
extern crate jsonpath_lib as jsonpath;
extern crate serde_json;

use std::io::Read;
use std::io::Write;

use log::LevelFilter;
use serde_json::Value;

use self::jsonpath::{JsonSelector, PathParser};

#[allow(dead_code)]
pub fn setup() {
    let _ = env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .parse_env("RUST_LOG")
        // .filter(Some("logger_example"), LevelFilter::Trace)
        .init();
}

#[allow(dead_code)]
pub fn read_json(path: &str) -> Value {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    serde_json::from_str(&contents).unwrap()
}

#[allow(dead_code)]
pub fn read_contents(path: &str) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
}

#[allow(dead_code)]
pub fn select_and_then_compare(path: &str, json: Value, target: Value) {
    let parser = PathParser::compile(path).unwrap();
    let mut selector = JsonSelector::new(parser);
    let result = selector.value(&json).select_as::<Value>().unwrap();
    assert_eq!(
        result,
        match target {
            Value::Array(vec) => vec,
            _ => panic!("Give me the Array!"),
        },
        "{}",
        path
    );
}

#[allow(dead_code)]
pub fn compare_result(result: Vec<&Value>, target: Value) {
    let result = serde_json::to_value(result).unwrap();
    assert_eq!(result, target);
}
