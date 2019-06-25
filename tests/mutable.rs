extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;

use common::{read_json, setup};
use jsonpath::{Selector, SelectorMut};
use serde_json::Value;

mod common;

#[test]
fn selector_mut() {
    setup();

    let mut selector_mut = SelectorMut::default();

    let mut nums = Vec::new();
    let result = selector_mut
        .str_path(r#"$.store..price"#)
        .unwrap()
        .value(read_json("./benches/example.json"))
        .replace_with(&mut |v| {
            match v {
                Value::Number(n) => {
                    nums.push(n.as_f64().unwrap());
                }
                _ => {}
            }
            Value::String("a".to_string())
        })
        .unwrap()
        .take()
        .unwrap();

    assert_eq!(
        nums,
        vec![8.95_f64, 12.99_f64, 8.99_f64, 22.99_f64, 19.95_f64]
    );

    let mut selector = Selector::default();
    let result = selector
        .str_path(r#"$.store..price"#)
        .unwrap()
        .value(&result)
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
