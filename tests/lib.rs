extern crate env_logger;
extern crate jsonpath_lib as jsonpath;
extern crate log;
#[macro_use]
extern crate serde_json;

use std::io::Read;

use serde_json::Value;

fn read_json(path: &str) -> Value {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    serde_json::from_str(contents.as_str()).unwrap()
}

#[test]
fn compile() {
    let mut template = jsonpath::compile("$..friends[2]");
    let json_obj = read_json("./benches/data_obj.json");
    let json = template(&json_obj).unwrap();
    let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Gray Berry"}
        ]);
    assert_eq!(json, ret);

    let json_obj = read_json("./benches/data_array.json");
    let json = template(&json_obj).unwrap();
    let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Rosetta Erickson"}
        ]);
    assert_eq!(json, ret);
}

#[test]
fn selector() {
    let json_obj = read_json("./benches/data_obj.json");
    let mut reader = jsonpath::selector(&json_obj);
    let json = reader("$..friends[2]").unwrap();
    let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Gray Berry"}
        ]);
    assert_eq!(json, ret);

    let json = reader("$..friends[0]").unwrap();
    let ret = json!([
            {"id": 0},
            {"id": 0,"name": "Millicent Norman"}
        ]);
    assert_eq!(json, ret);
}

#[test]
fn select() {
    let json_obj = read_json("./benches/example.json");
    let json = jsonpath::select(&json_obj, "$..book[2]").unwrap();
    let ret = json!([{
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
        }]);
    assert_eq!(json, ret);
}