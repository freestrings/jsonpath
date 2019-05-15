extern crate jsonpath_lib as jsonpath;
extern crate serde;
#[macro_use]
extern crate serde_json;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use jsonpath::Selector;

#[test]
fn readme_selector() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Friend {
        name: String,
        age: Option<u8>,
    }

    let json_obj = json!({
        "school": {
            "friends": [
                {"name": "친구1", "age": 20},
                {"name": "친구2", "age": 20}
            ]
        },
        "friends": [
            {"name": "친구3", "age": 30},
            {"name": "친구4"}
    ]});

    let mut selector = Selector::new();

    let result = selector
        .path("$..[?(@.age >= 30)]").unwrap()
//    .value_from_str(&serde_json::to_string(&json_obj).unwrap() /*&str*/).unwrap()
//    .value_from(&json_obj /*&impl serde::ser::Serialize*/).unwrap()
        .value(&json_obj /*serde_json::value::Value*/).unwrap()
        .select_as_value().unwrap();

    assert_eq!(json!([{"name": "친구3", "age": 30}]), result);

    let result = selector.select_as_str().unwrap();
    assert_eq!(r#"[{"name":"친구3","age":30}]"#, result);

    let result = selector.select_as::<Vec<Friend>>().unwrap();
    assert_eq!(vec![Friend { name: "친구3".to_string(), age: Some(30) }], result);

    let _ = selector.map(|v| {
        let r = match v {
            Value::Array(mut vec) => {
                for mut v in &mut vec {
                    v.as_object_mut().unwrap().remove("age");
                }
                Value::Array(vec)
            }
            _ => Value::Null
        };
        Some(r)
    });
    assert_eq!(json!([{ "name": "친구3"}]), selector.get().unwrap());

    let _ = selector.value(&json_obj).unwrap()
        .map_as(|mut v: Vec<Friend>| {
            let mut f = v.pop().unwrap();
            f.name = "friend3".to_string();
            f.age = None;
            Some(vec![f])
        });
    assert_eq!(vec![Friend { name: "friend3".to_string(), age: None }],
               selector.get_as::<Vec<Friend>>().unwrap());
}

#[test]
fn readme_select() {
    let json_obj = json!({
        "school": {
            "friends": [
                {"name": "친구1", "age": 20},
                {"name": "친구2", "age": 20}
            ]
        },
        "friends": [
            {"name": "친구3", "age": 30},
            {"name": "친구4"}
    ]});

    let json = jsonpath::select(&json_obj, "$..friends[0]").unwrap();

    let ret = json!([
        {"name": "친구3", "age": 30},
        {"name": "친구1", "age": 20}
    ]);
    assert_eq!(json, ret);
}

#[test]
fn readme_select_as_str() {
    let ret = jsonpath::select_as_str(r#"
    {
        "school": {
            "friends": [
                    {"name": "친구1", "age": 20},
                    {"name": "친구2", "age": 20}
                ]
        },
        "friends": [
            {"name": "친구3", "age": 30},
            {"name": "친구4"}
        ]
    }
    "#, "$..friends[0]").unwrap();

    assert_eq!(ret, r#"[{"name":"친구3","age":30},{"name":"친구1","age":20}]"#);
}

#[test]
fn readme_select_as() {
    #[derive(Deserialize, PartialEq, Debug)]
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

#[test]
fn readme_compile() {
    let mut template = jsonpath::compile("$..friends[0]");

    let json_obj = json!({
        "school": {
            "friends": [
                {"name": "친구1", "age": 20},
                {"name": "친구2", "age": 20}
            ]
        },
        "friends": [
            {"name": "친구3", "age": 30},
            {"name": "친구4"}
    ]});

    let json = template(&json_obj).unwrap();

    let ret = json!([
        {"name": "친구3", "age": 30},
        {"name": "친구1", "age": 20}
    ]);

    assert_eq!(json, ret);
}

#[test]
fn readme_selector_fn() {
    let json_obj = json!({
        "school": {
            "friends": [
                {"name": "친구1", "age": 20},
                {"name": "친구2", "age": 20}
            ]
        },
        "friends": [
            {"name": "친구3", "age": 30},
            {"name": "친구4"}
    ]});

    let mut selector = jsonpath::selector(&json_obj);

    let json = selector("$..friends[0]").unwrap();

    let ret = json!([
        {"name": "친구3", "age": 30},
        {"name": "친구1", "age": 20}
    ]);

    assert_eq!(json, ret);

    let json = selector("$..friends[1]").unwrap();

    let ret = json!([
        {"name": "친구4"},
        {"name": "친구2", "age": 20}
    ]);

    assert_eq!(json, ret);
}

#[test]
fn readme_selector_as() {
    let json_obj = json!({
        "school": {
           "friends": [
                {"name": "친구1", "age": 20},
                {"name": "친구2", "age": 20}
            ]
        },
        "friends": [
            {"name": "친구3", "age": 30},
            {"name": "친구4"}
    ]});

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Friend {
        name: String,
        age: Option<u8>,
    }

    let mut selector = jsonpath::selector_as::<Vec<Friend>>(&json_obj);

    let json = selector("$..friends[0]").unwrap();

    let ret = vec!(
        Friend { name: "친구3".to_string(), age: Some(30) },
        Friend { name: "친구1".to_string(), age: Some(20) }
    );
    assert_eq!(json, ret);

    let json = selector("$..friends[1]").unwrap();

    let ret = vec!(
        Friend { name: "친구4".to_string(), age: None },
        Friend { name: "친구2".to_string(), age: Some(20) }
    );

    assert_eq!(json, ret);
}