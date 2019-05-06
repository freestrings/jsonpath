extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use jsonpath::Selector;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Person {
    name: String,
    age: u8,
    phone: String,
}

fn input_str() -> &'static str {
    r#"[
        {
            "name": "이름1",
            "age": 40,
            "phone": "+33 12341234"
        },
        {
            "name": "이름2",
            "age": 42,
            "phone": "++44 12341234"
        },
        {
            "name": "이름3",
            "age": 50,
            "phone": "++55 111111"
        },
        {
            "name": "이름4",
            "age": 51,
            "phone": "++55 12341234"
        }
    ]"#
}

fn input_json() -> Value {
    serde_json::from_str(input_str()).unwrap()
}

fn input_person() -> Vec<Person> {
    serde_json::from_str(input_str()).unwrap()
}

#[test]
fn selector_value_from() {
    let result = Selector::new()
        .path("$..[?(@.age > 40)]").unwrap()
        .value_from(&input_person()).unwrap()
        .select_to::<Vec<Person>>().unwrap();
    assert_eq!(input_person()[1], result[0]);
}

#[test]
fn selector_value() {
    let result = Selector::new()
        .path("$..[?(@.age > 40)]").unwrap()
        .value((&input_json()).into()).unwrap()
        .select_to_value().unwrap();
    assert_eq!(input_json()[1], result[0]);
}

#[test]
fn selector_value_from_str() {
    let result = Selector::new()
        .path("$..[?(@.age > 40)]").unwrap()
        .value_from_str(input_str()).unwrap()
        .select_to_value().unwrap();
    assert_eq!(input_json()[1], result[0]);
}

#[test]
fn selector_select_to() {
    let mut selector = Selector::new();

    let result = selector
        .path("$..[?(@.age > 40)]").unwrap()
        .value_from_str(input_str()).unwrap()
        .select_to_value().unwrap();
    assert_eq!(input_json()[1], result[0]);

    let result = selector.select_to_str().unwrap();
    let value: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(input_json()[1], value[0]);

    let result = selector.select_to::<Vec<Person>>().unwrap();
    assert_eq!(input_person()[1], result[0]);

    let _ = selector.path("$..[?(@.age == 40)]");

    let result = selector.select_to_value().unwrap();
    assert_eq!(input_json()[0], result[0]);

    let result = selector.select_to_str().unwrap();
    assert_eq!(serde_json::to_string(&vec![&input_json()[0].clone()]).unwrap(), result);

    let result = selector.select_to::<Vec<Person>>().unwrap();
    assert_eq!(input_person()[0], result[0]);
}