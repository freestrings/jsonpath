#[macro_use]
extern crate serde_json;

use common::{select_and_then_compare, setup};

mod common;

#[test]
fn dolla_token_in_path() {
    setup();

    select_and_then_compare(
        "$..$ref",
        json!({
            "Junk1": "This is a test to illustrate use of '$' in the attr for the expression $..['$ref'] ",
            "$ref": "Match Root",
            "Subset1":[
                {"Junk2": "Data...",
                 "$ref": "Match Subset1"
                }
            ],
            "hierachy1":{
                "hierachy2.1":{
                    "hierachy2.1.1":{ "$ref":"Match 2.1.1"},
                    "hierachy2.1.2":{ "ref":"Match 2.1.2"},
                    "hierachy2.1.3":{ "ref":"No Match 2.1.3"},
                    "hierachy2.1.4":{ "$ref":"Match 2.1.4"},
                    "hierachy2.1.5":{ "ref":"No Match 2.1.5"}
                },
                "hierachy2.2":{
                    "hierachy2.2.1":{ "ref":"No Match 2.2.1"},
                    "hierachy2.2.2":{ "$ref":"Match 2.2.2"},
                    "hierachy2.2.3":{ "ref":"No Match 2.2.3"},
                    "hierachy2.2.4":{ "ref":"No Match 2.2.5"},
                    "hierachy2.2.5":{ "$ref":"Match 2.2.5"}
                },
                "hierachy2.3":{
                    "hierachy2.3.1":{ "ref":"No Match 2.3.1"},
                    "hierachy2.3.2":{ "ref":"No Match 2.3.2"},
                    "hierachy2.3.3":{ "ref":"No Match 2.3.3"},
                    "hierachy2.3.4":{ "ref":"No Match 2.3.4"},
                    "hierachy2.3.5":{ "ref":"No Match 2.3.5"},
                    "hierachy2.3.6":{
                        "hierachy2.3.6.1":{ "$ref":"Match 2.3.6.1"},
                        "hierachy2.3.6.2":{ "ref":"No Match 2.3.6.2"},
                        "hierachy2.3.6.3":{ "ref":"No Match 2.3.6.3"},
                        "hierachy2.3.6.4":{ "ref":"No Match 2.3.6.4"},
                        "hierachy2.3.6.5":{ "ref":"No Match 2.3.6.5"}
                        }
                    }
            }
        }),
        json!([
            "Match Root",
            "Match Subset1",
            "Match 2.1.1",
            "Match 2.1.4",
            "Match 2.2.2",
            "Match 2.2.5",
            "Match 2.3.6.1"
        ]),
    );

    select_and_then_compare(
        "$..['$ref']",
        json!({
            "Junk1": "This is a test to illustrate use of '$' in the attr for the expression $..['$ref'] ",
            "$ref": "Match Root",
            "Subset1":[
                {"Junk2": "Data...",
                 "$ref": "Match Subset1"
                }
            ],
            "hierachy1":{
                "hierachy2.1":{
                    "hierachy2.1.1":{ "$ref":"Match 2.1.1"},
                    "hierachy2.1.2":{ "ref":"Match 2.1.2"},
                    "hierachy2.1.3":{ "ref":"No Match 2.1.3"},
                    "hierachy2.1.4":{ "$ref":"Match 2.1.4"},
                    "hierachy2.1.5":{ "ref":"No Match 2.1.5"}
                },
                "hierachy2.2":{
                    "hierachy2.2.1":{ "ref":"No Match 2.2.1"},
                    "hierachy2.2.2":{ "$ref":"Match 2.2.2"},
                    "hierachy2.2.3":{ "ref":"No Match 2.2.3"},
                    "hierachy2.2.4":{ "ref":"No Match 2.2.5"},
                    "hierachy2.2.5":{ "$ref":"Match 2.2.5"}
                },
                "hierachy2.3":{
                    "hierachy2.3.1":{ "ref":"No Match 2.3.1"},
                    "hierachy2.3.2":{ "ref":"No Match 2.3.2"},
                    "hierachy2.3.3":{ "ref":"No Match 2.3.3"},
                    "hierachy2.3.4":{ "ref":"No Match 2.3.4"},
                    "hierachy2.3.5":{ "ref":"No Match 2.3.5"},
                    "hierachy2.3.6":{
                        "hierachy2.3.6.1":{ "$ref":"Match 2.3.6.1"},
                        "hierachy2.3.6.2":{ "ref":"No Match 2.3.6.2"},
                        "hierachy2.3.6.3":{ "ref":"No Match 2.3.6.3"},
                        "hierachy2.3.6.4":{ "ref":"No Match 2.3.6.4"},
                        "hierachy2.3.6.5":{ "ref":"No Match 2.3.6.5"}
                        }
                    }
            }
        }),
        json!([
            "Match Root",
            "Match Subset1",
            "Match 2.1.1",
            "Match 2.1.4",
            "Match 2.2.2",
            "Match 2.2.5",
            "Match 2.3.6.1"
        ]),
    );
}
