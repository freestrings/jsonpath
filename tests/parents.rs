#[macro_use]
extern crate serde_json;

use common::{read_json, select_and_then_compare, setup};

mod common;

#[test]
fn parent_of_root() {
    setup();

    for path in &[r#"$^*"#, r#"$^^*"#, r#"$^*^*", r#"$.first^^*"#] {
        select_and_then_compare(
            path,
            json!({"first":"value"}),
            json!([]),
        );
    }
}

#[test]
fn parent_key() {
    setup();

    for path in &[r#"$.store^expensive"#, r#"$.store.bicycle^^expensive"#] {
        select_and_then_compare(
            path,
            read_json("./benchmark/example.json"),
            json!([10]),
        );
    }
}

#[test]
fn parent_parent() {
    setup();

    select_and_then_compare(
        r#"$.store.bicycle^^expensive"#,
        read_json("./benchmark/example.json"),
        json!([
            10
        ])
    );

    select_and_then_compare(
        r#"$.store.book[0].author^^^bicycle"#,
        read_json("./benchmark/example.json"),
        json!([
            {
                "color": "red",
                "price": 19.95
            }
        ])
    );
}

#[test]
fn parent_array() {
    setup();

    select_and_then_compare(
        r#"$.store.book[1]^[0]"#,
        read_json("./benchmark/example.json"),
        json!([
            {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
            }
        ])
    );

    select_and_then_compare(
        r#"$.store.book[*]^[0]"#,
        read_json("./benchmark/example.json"),
        json!([
            {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
            },
            {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
            },
            {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
            },
            {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
            }
        ])
    );
}

#[test]
fn parent_all() {
    setup();

    select_and_then_compare(
        r#"$.store.bicycle.color^*"#,
        read_json("./benchmark/example.json"),
        json!([
            "red",
            19.95
        ])
    );

    select_and_then_compare(
        r#"$.store.book[0].category^^*.author"#,
        read_json("./benchmark/example.json"),
        json!([
            "Nigel Rees",
            "Evelyn Waugh",
            "Herman Melville",
            "J. R. R. Tolkien"
        ])
    );
}

#[test]
fn parent_after_leaves() {
    setup();

    select_and_then_compare(
        r#"$..author^title"#,
        read_json("./benchmark/example.json"),
        json!([
            "Sayings of the Century",
            "Sword of Honour",
            "Moby Dick",
            "The Lord of the Rings"
        ])
    );
}

#[test]
fn parent_after_filter() {
    setup();

    select_and_then_compare(
        "$.store.book[?(@.price == 12.99)]^^bicycle",
        read_json("./benchmark/example.json"),
        json!([
            {
                "color": "red",
                "price": 19.95
            }
        ])
    );
}
