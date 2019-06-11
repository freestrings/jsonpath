extern crate jsonpath_lib as jsonpath;
extern crate serde;
#[macro_use]
extern crate serde_json;

use serde::Deserialize;
use serde_json::Value;

use common::{compare_result, read_contents, read_json, setup};

mod common;

#[test]
fn compile() {
    setup();

    let mut template = jsonpath::compile("$..friends[2]");
    let json_obj = read_json("./benches/data_obj.json");
    let json = template(&json_obj).unwrap();
    let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Gray Berry"}
        ]);
    compare_result(json, ret);

    let json_obj = read_json("./benches/data_array.json");
    let json = template(&json_obj).unwrap();
    let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Rosetta Erickson"}
        ]);
    compare_result(json, ret);
}

#[test]
fn selector() {
    setup();

    let json_obj = read_json("./benches/data_obj.json");
    let mut reader = jsonpath::selector(&json_obj);
    let json = reader("$..friends[2]").unwrap();
    let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Gray Berry"}
        ]);
    compare_result(json, ret);

    let json = reader("$..friends[0]").unwrap();
    let ret = json!([
            {"id": 0},
            {"id": 0,"name": "Millicent Norman"}
        ]);
    compare_result(json, ret);
}

#[test]
fn selector_as() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Friend {
        id: u8,
        name: Option<String>,
    }

    let json_obj = read_json("./benches/data_obj.json");
    let mut selector = jsonpath::selector_as::<Friend>(&json_obj);
    let json = selector("$..friends[2]").unwrap();

    let ret = vec!(
        Friend { id: 2, name: Some("Gray Berry".to_string()) },
        Friend { id: 2, name: Some("Gray Berry".to_string()) },
    );
    assert_eq!(json, ret);

    let json = selector("$..friends[0]").unwrap();
    let ret = vec!(
        Friend { id: 0, name: None },
        Friend { id: 0, name: Some("Millicent Norman".to_string()) },
    );
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
    compare_result(json, ret);
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
    #[derive(Deserialize, PartialEq, Debug)]
    struct Person {
        name: String,
        age: u8,
        phones: Vec<String>,
    }

    let ret: Vec<Person> = jsonpath::select_as(r#"
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

    assert_eq!(vec![person], ret);
}