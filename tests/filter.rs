#[macro_use]
extern crate serde_json;

use serde_json::Value;

use common::{read_json, select_and_then_compare, setup};

mod common;

#[test]
fn array() {
    setup();

    select_and_then_compare("$.school.friends[1, 2]", read_json("./benches/data_obj.json"), json!([
        {"id": 1, "name": "Vincent Cannon" },
        {"id": 2, "name": "Gray Berry"}
    ]));

    select_and_then_compare("$.school.friends[1: ]", read_json("./benches/data_obj.json"), json!([
        {"id": 1, "name": "Vincent Cannon" },
        {"id": 2, "name": "Gray Berry"}
    ]));

    select_and_then_compare("$.school.friends[:-2]", read_json("./benches/data_obj.json"), json!([
        {"id": 0, "name": "Millicent Norman"}
    ]));

    select_and_then_compare("$..friends[2].name", read_json("./benches/data_obj.json"), json!([
        "Gray Berry", "Gray Berry"
    ]));

    select_and_then_compare("$..friends[*].name", read_json("./benches/data_obj.json"), json!([
        "Vincent Cannon","Gray Berry","Millicent Norman","Vincent Cannon","Gray Berry"
    ]));

    select_and_then_compare("$['school']['friends'][*].['name']", read_json("./benches/data_obj.json"), json!([
        "Millicent Norman","Vincent Cannon","Gray Berry"
    ]));

    select_and_then_compare("$['school']['friends'][0].['name']", read_json("./benches/data_obj.json"), json!([
        "Millicent Norman"
    ]));
}

#[test]
fn return_type() {
    setup();

    select_and_then_compare("$.school", read_json("./benches/data_obj.json"), json!([{
        "friends": [
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]
    }]));

    select_and_then_compare("$.school[?(@.friends[0])]", read_json("./benches/data_obj.json"), json!([{
        "friends": [
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]
    }]));

    select_and_then_compare("$.school[?(@.friends[10])]", read_json("./benches/data_obj.json"), json!([{
        "friends": [
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]
    }]));

    select_and_then_compare("$.school[?(1==1)]", read_json("./benches/data_obj.json"), json!([{
        "friends": [
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]
    }]));

    select_and_then_compare("$.school.friends[?(1==1)]", read_json("./benches/data_obj.json"), json!([[
        {"id": 0, "name": "Millicent Norman"},
        {"id": 1, "name": "Vincent Cannon" },
        {"id": 2, "name": "Gray Berry"}
    ]]));
}

#[test]
fn op_default() {
    setup();

    select_and_then_compare("$.school[?(@.friends == @.friends)]", read_json("./benches/data_obj.json"), json!([{
        "friends": [
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]
    }]));

    select_and_then_compare("$.friends[?(@.name)]", read_json("./benches/data_obj.json"), json!([
        { "id" : 1, "name" : "Vincent Cannon" },
        { "id" : 2, "name" : "Gray Berry" }
    ]));

    select_and_then_compare("$.friends[?(@.id >= 2)]", read_json("./benches/data_obj.json"), json!([
        { "id" : 2, "name" : "Gray Berry" }
    ]));

    select_and_then_compare("$.friends[?(@.id >= 2 || @.id == 1)]", read_json("./benches/data_obj.json"), json!([
        { "id" : 2, "name" : "Gray Berry" },
        { "id" : 1, "name" : "Vincent Cannon" }
    ]));

    select_and_then_compare("$.friends[?( (@.id >= 2 || @.id == 1) && @.id == 0)]", read_json("./benches/data_obj.json"), json!([
        Value::Null
    ]));

    select_and_then_compare("$..friends[?(@.id == $.index)].id", read_json("./benches/data_obj.json"), json!([
        0, 0
    ]));

    select_and_then_compare("$..book[?($.store.bicycle.price < @.price)].price", read_json("./benches/example.json"), json!([
        22.99
    ]));

    select_and_then_compare("$..book[?( (@.price == 12.99 || @.category == 'reference') && @.price > 10)].price", read_json("./benches/example.json"), json!([
        12.99
    ]));

    select_and_then_compare("$..[?(@.age > 40)]", json!([
        { "name": "이름1", "age": 40, "phone": "+33 12341234" },
        { "name": "이름2", "age": 42, "phone": "++44 12341234" }
    ]), json!([
        { "name" : "이름2", "age" : 42, "phone" : "++44 12341234" }
    ]));

    select_and_then_compare("$..[?(@.age >= 30)]", json!({
        "school": {
            "friends": [
                {"name": "친구1", "age": 20},
                {"name": "친구2", "age": 20}
            ]
        },
        "friends": [
            {"name": "친구3", "age": 30},
            {"name": "친구4"}
    ]}), json!([
        { "name" : "친구3", "age" : 30 }
    ]));
}

#[test]
fn op_number() {
    setup();

    select_and_then_compare("$.[?(@.a == 1)]", json!({ "a": 1 }), json!([{ "a": 1 }]));
    select_and_then_compare("$.[?(@.a != 2)]", json!({ "a": 1 }), json!([{ "a": 1 }]));
    select_and_then_compare("$.[?(@.a < 2)]", json!({ "a": 1 }), json!([{ "a": 1 }]));
    select_and_then_compare("$.[?(@.a <= 1)]", json!({ "a": 1 }), json!([{ "a": 1 }]));
    select_and_then_compare("$.[?(@.a > 0)]", json!({ "a": 1 }), json!([{ "a": 1 }]));
    select_and_then_compare("$.[?(@.a >= 0)]", json!({ "a": 1 }), json!([{ "a": 1 }]));
}

#[test]
fn op_string() {
    setup();

    select_and_then_compare(r#"$.[?(@.a == "b")]"#, json!({ "a": "b" }), json!([{ "a": "b" }]));
    select_and_then_compare(r#"$.[?(@.a != "c")]"#, json!({ "a": "b" }), json!([{ "a": "b" }]));
    select_and_then_compare(r#"$.[?(@.a < "b")]"#, json!({ "a": "b" }), json!([Value::Null]));
    select_and_then_compare(r#"$.[?(@.a <= "b")]"#, json!({ "a": "b" }), json!([{ "a": "b" }]));
    select_and_then_compare(r#"$.[?(@.a > "b")]"#, json!({ "a": "b" }), json!([Value::Null]));
    select_and_then_compare(r#"$.[?(@.a >= "b")]"#, json!({ "a": "b" }), json!([{ "a": "b" }]));
}

#[test]
fn op_object() {
    setup();

    select_and_then_compare(r#"$.[?(@.a == @.c)]"#,
                            json!({"a": { "1": 1 }, "b": { "2": 2 }, "c": { "1": 1 }}),
                            json!([{"a": { "1": 1 }, "b": { "2": 2 }, "c": { "1": 1 }}]));
    select_and_then_compare(r#"$.[?(@.a != @.c)]"#,
                            json!({"a": { "1": 1 }, "b": { "2": 2 }, "c": { "1": 1 }}),
                            json!([Value::Null]));
    select_and_then_compare(r#"$.[?(@.a < @.c)]"#,
                            json!({"a": { "1": 1 }, "b": { "2": 2 }, "c": { "1": 1 }}),
                            json!([Value::Null]));
    select_and_then_compare(r#"$.[?(@.a <= @.c)]"#,
                            json!({"a": { "1": 1 }, "b": { "2": 2 }, "c": { "1": 1 }}),
                            json!([Value::Null]));
    select_and_then_compare(r#"$.[?(@.a > @.c)]"#,
                            json!({"a": { "1": 1 }, "b": { "2": 2 }, "c": { "1": 1 }}),
                            json!([Value::Null]));
    select_and_then_compare(r#"$.[?(@.a >= @.c)]"#,
                            json!({"a": { "1": 1 }, "b": { "2": 2 }, "c": { "1": 1 }}),
                            json!([Value::Null]));
}

#[test]
fn op_complex() {
    setup();

    select_and_then_compare(r#"$.[?(1 == @.a)]"#, json!({ "a": { "b": 1 } }), json!([Value::Null]));
    select_and_then_compare(r#"$.[?("1" != @.a)]"#, json!({ "a": { "b": 1 } }), json!([Value::Null]));
    select_and_then_compare(r#"$.[?(@.a <= 1)]"#, json!({ "a": { "b": 1 } }), json!([Value::Null]));
    select_and_then_compare(r#"$.[?(@.a > "1")]"#, json!({ "a": { "b": 1 } }), json!([Value::Null]));
}

#[test]
fn example() {
    setup();

    select_and_then_compare(r#"$.store.book[*].author"#, read_json("./benches/example.json"), json!([
        "Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"
    ]));

    select_and_then_compare(r#"$..author"#, read_json("./benches/example.json"), json!([
        "Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"
    ]));

    select_and_then_compare(r#"$.store.*"#, read_json("./benches/example.json"), json!([
         [
            {"category" : "reference", "author" : "Nigel Rees","title" : "Sayings of the Century", "price" : 8.95},
            {"category" : "fiction", "author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99},
            {"category" : "fiction", "author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99},
            {"category" : "fiction", "author" : "J. R. R. Tolkien","title" : "The Lord of the Rings","isbn" : "0-395-19395-8","price" : 22.99}
        ],
        {"color" : "red","price" : 19.95},
    ]));

    select_and_then_compare(r#"$.store..price"#, read_json("./benches/example.json"), json!([
        8.95, 12.99, 8.99, 22.99, 19.95
    ]));

    select_and_then_compare(r#"$..book[2]"#, read_json("./benches/example.json"), json!([
        {
        "category" : "fiction",
        "author" : "Herman Melville",
        "title" : "Moby Dick",
        "isbn" : "0-553-21311-3",
        "price" : 8.99
        }
    ]));

    select_and_then_compare(r#"$..book[-2]"#, read_json("./benches/example.json"), json!([
        {
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
        }
    ]));

    select_and_then_compare(r#"$..book[0, 1]"#, read_json("./benches/example.json"), json!([
        {
            "category" : "reference",
            "author" : "Nigel Rees",
            "title" : "Sayings of the Century",
            "price" : 8.95
        },
        {
            "category" : "fiction",
            "author" : "Evelyn Waugh",
            "title" : "Sword of Honour",
            "price" : 12.99
        }
    ]));

    select_and_then_compare(r#"$..book[:2]"#, read_json("./benches/example.json"), json!([
        {
            "category" : "reference",
            "author" : "Nigel Rees",
            "title" : "Sayings of the Century",
            "price" : 8.95
        },
        {
            "category" : "fiction",
            "author" : "Evelyn Waugh",
            "title" : "Sword of Honour",
            "price" : 12.99
        }
    ]));

    select_and_then_compare(r#"$..book[2:]"#, read_json("./benches/example.json"), json!([
        {
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
       },
       {
            "category" : "fiction",
            "author" : "J. R. R. Tolkien",
            "title" : "The Lord of the Rings",
            "isbn" : "0-395-19395-8",
            "price" : 22.99
       }
    ]));

    select_and_then_compare(r#"$..book[?(@.isbn)]"#, read_json("./benches/example.json"), json!([
        {
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
       },
       {
            "category" : "fiction",
            "author" : "J. R. R. Tolkien",
            "title" : "The Lord of the Rings",
            "isbn" : "0-395-19395-8",
            "price" : 22.99
       }
    ]));

    select_and_then_compare(r#"$.store.book[?(@.price < 10)]"#, read_json("./benches/example.json"), json!([
        {
            "category" : "reference",
            "author" : "Nigel Rees",
            "title" : "Sayings of the Century",
            "price" : 8.95
       },
       {
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
       }
    ]));

    select_and_then_compare(r#"$..*"#, read_json("./benches/example.json"),
                            read_json("./benches/giveme_every_thing_result.json"));
}

#[test]
fn filer_same_obj() {
    setup();

    select_and_then_compare(r#"$..[?(@.a == 1)]"#, json!({
        "a": 1,
        "b" : {"a": 1},
        "c" : {"a": 1}
    }), json!([
        {"a": 1},
        {"a": 1}
    ]));
}

#[test]
fn range() {
    setup();

    select_and_then_compare("$[:]", json!(["first", "second"]), json!(["first", "second"]));
    select_and_then_compare("$[::]", json!(["first", "second", "third", "forth", "fifth"]), json!(["first", "second", "third", "forth", "fifth"]));
    select_and_then_compare("$[::2]", json!(["first", "second", "third", "forth", "fifth"]), json!(["first", "third", "fifth"]));
    select_and_then_compare("$[1: :]", json!(["first", "second", "third", "forth", "fifth"]), json!(["second", "third", "forth", "fifth"]));
    select_and_then_compare("$[1:2:]", json!(["first", "second", "third", "forth", "fifth"]), json!(["second"]));
    select_and_then_compare("$[1::2]", json!(["first", "second", "third", "forth", "fifth"]), json!(["second", "forth"]));
    select_and_then_compare("$[0:3:1]", json!(["first", "second", "third", "forth", "fifth"]), json!(["first", "second", "third"]));
    select_and_then_compare("$[0:3:2]", json!(["first", "second", "third", "forth", "fifth"]), json!(["first", "third"]));
}

#[test]
fn quote() {
    setup();

    select_and_then_compare(r#"$['single\'quote']"#, json!({"single'quote":"value"}), json!(["value"]));
    select_and_then_compare(r#"$["double\"quote"]"#, json!({"double\"quote":"value"}), json!(["value"]));
}