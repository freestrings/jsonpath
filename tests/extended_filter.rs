#[macro_use]
extern crate serde_json;

use common::{select_and_then_compare, setup};

mod common;

#[test]
fn extended_filter_in() {
    setup();

    select_and_then_compare(
        "$..[?(@.size in ['M', 'L', 0])]",
        json!({
            "red" : {
                "size": "M"
            },
            "blue" : {
                "size" : "L"
            },
            "yellow" : {
                "size" : "XL"
            }
        }),
        json!([
           {
              "size" : "M"
           },
           {
              "size" : "L"
           }
        ]),
    );
}