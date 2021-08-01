extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;

use serde_json::Value;

use common::{read_json, setup};
use jsonpath::{PathParser, JsonSelector, JsonSelectorMut};

mod common;

#[test]
fn selector_mut() {
    setup();

    let parser = PathParser::compile("$.store..price").unwrap();
    let mut selector_mut = JsonSelectorMut::new(parser);

    let mut nums = Vec::new();
    let result = selector_mut.value(read_json("./benchmark/example.json"))
        .replace_with(&mut |v| {
            if let Value::Number(n) = v {
                nums.push(n.as_f64().unwrap());
            }
            Some(Value::String("a".to_string()))
        })
        .unwrap()
        .take()
        .unwrap();

    assert_eq!(
        nums,
        vec![8.95_f64, 12.99_f64, 8.99_f64, 22.99_f64, 19.95_f64]
    );

    let parser = PathParser::compile("$.store..price").unwrap();
    let mut selector = JsonSelector::new(parser);
    let result = selector.value(&result)
        .select()
        .unwrap();

    assert_eq!(
        vec![
            &json!("a"),
            &json!("a"),
            &json!("a"),
            &json!("a"),
            &json!("a")
        ],
        result
    );
}

#[test]
fn selector_delete_multi_elements_from_array() {
    setup();

    let parser = PathParser::compile("$[0,2]").unwrap();
    let mut selector_mut = JsonSelectorMut::new(parser);

    let result = selector_mut.value(serde_json::from_str("[1,2,3]").unwrap())
        .remove()
        .unwrap()
        .take()
        .unwrap();

    assert_eq!(
        result,
        serde_json::from_str::<serde_json::Value>("[2,3]").unwrap(),
    );
}

#[test]
fn selector_delete() {
    setup();

    let parser = PathParser::compile("$.store..price[?(@>13)]").unwrap();
    let mut selector_mut = JsonSelectorMut::new(parser);

    let result = selector_mut.value(read_json("./benchmark/example.json"))
        .delete()
        .unwrap()
        .take()
        .unwrap();

    let parser = PathParser::compile("$.store..price").unwrap();
    let mut selector = JsonSelector::new(parser);
    let result = selector.value(&result)
        .select()
        .unwrap();

    assert_eq!(
        result,
        vec![
            &json!(8.95),
            &json!(12.99),
            &json!(8.99),
            &Value::Null,
            &Value::Null
        ]
    );
}

#[test]
fn selector_remove() {
    setup();
    let parser = PathParser::compile("$.store..price[?(@>13)]").unwrap();
    let mut selector_mut = JsonSelectorMut::new(parser);

    let result = selector_mut.value(read_json("./benchmark/example.json"))
        .remove()
        .unwrap()
        .take()
        .unwrap();

    let parser = PathParser::compile("$.store..price").unwrap();
    let mut selector = JsonSelector::new(parser);
    let result = selector.value(&result)
        .select()
        .unwrap();

    assert_eq!(
        result,
        vec![
            &json!(8.95),
            &json!(12.99),
            &json!(8.99)
        ]
    );
}
