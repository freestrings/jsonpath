extern crate env_logger;
extern crate jsonpath_lib as jsonpath;
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::io::Read;

use serde::{Deserialize, Serialize};
use serde_json::Value;

fn read_json(path: &str) -> Value {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    serde_json::from_str(contents.as_str()).unwrap()
}

fn read_contents(path: &str) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
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

#[test]
fn select_str() {
    let json_str = read_contents("./benches/example.json");
    let result_str = jsonpath::select_as_str(&json_str, "$..book[2]").unwrap();
    let ret = json!([{
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
        }]);
    let json: Value = serde_json::from_str(&result_str).unwrap();
    assert_eq!(json, ret);
}

#[test]
fn test_to_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Person {
        name: String,
        age: u8,
        phones: Vec<String>,
    }

    let ret: Person = jsonpath::select_as(r#"
    {
        "person":
            {
                "name": "Doe John",
                "age": 44,
                "phones": [
                    "+44 1234567",
                    "+44 2345678"
                ]
            }
    }
    "#, "$.person").unwrap();

    let person = Person {
        name: "Doe John".to_string(),
        age: 44,
        phones: vec!["+44 1234567".to_string(), "+44 2345678".to_string()],
    };

    assert_eq!(person, ret);
}