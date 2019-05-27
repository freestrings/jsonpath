extern crate core;
extern crate env_logger;
extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;

use std::io::Read;

use serde_json::Value;

use jsonpath::Selector;
use jsonpath::filter::value_filter::ValueFilter;

fn setup() {
    let _ = env_logger::try_init();
}

fn new_value_filter(file: &str) -> ValueFilter {
    let string = read_json(file);
    let json: Value = serde_json::from_str(string.as_str()).unwrap();
    ValueFilter::new((&json).into(), false, false)
}

fn selector(path: &str, file: &str) -> Selector {
    let string = read_json(file);
    let mut s = Selector::new();
    let _ = s.path(path);
    let _ = s.value_from_str(&string);
    s
}

fn read_json(path: &str) -> String {
    let mut f = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
}

#[test]
fn step_in() {
    setup();

    let mut jf = new_value_filter("./benches/data_obj.json");
    {
        let current = jf.step_in_str("friends");
        assert_eq!(current.is_array(), true);
    }

    let mut jf = new_value_filter("./benches/data_array.json");
    {
        let current = jf.step_in_num(&1.0);
        assert_eq!(current.get_val().is_object(), true);
    }
    {
        let current = jf.step_in_str("friends");
        assert_eq!(current.is_array(), true);
    }
    let mut jf = new_value_filter("./benches/data_obj.json");
    {
        jf.step_in_str("school");
        jf.step_in_str("friends");
        jf.step_in_all();
        let current = jf.step_in_str("name");
        let friends = json!([
                "Millicent Norman",
                "Vincent Cannon",
                "Gray Berry"
            ]);

        assert_eq!(friends, current.into_value());
    }
    let mut jf = new_value_filter("./benches/data_obj.json");
    {
        let current = jf.step_leaves_str("name");
        let names = json!([
                "Leonor Herman",
                "Millicent Norman",
                "Vincent Cannon",
                "Gray Berry",
                "Vincent Cannon",
                "Gray Berry"
            ]);
        assert_eq!(names, current.into_value());
    }
}

#[test]
fn array() {
    setup();

    let friends = json!([
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]);

    let s = selector("$.school.friends[1, 2]", "./benches/data_obj.json");
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.school.friends[1:]", "./benches/data_obj.json");
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.school.friends[:-2]", "./benches/data_obj.json");
    let friends = json!([
            {"id": 0, "name": "Millicent Norman"}
        ]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$..friends[2].name", "./benches/data_obj.json");
    let friends = json!(["Gray Berry", "Gray Berry"]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$..friends[*].name", "./benches/data_obj.json");
    let friends = json!(["Vincent Cannon","Gray Berry","Millicent Norman","Vincent Cannon","Gray Berry"]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$['school']['friends'][*].['name']", "./benches/data_obj.json");
    let friends = json!(["Millicent Norman","Vincent Cannon","Gray Berry"]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$['school']['friends'][0].['name']", "./benches/data_obj.json");
    let friends = json!("Millicent Norman");
    assert_eq!(friends, s.select_as_value().unwrap());
}

#[test]
fn return_type() {
    setup();

    let friends = json!({
            "friends": [
                {"id": 0, "name": "Millicent Norman"},
                {"id": 1, "name": "Vincent Cannon" },
                {"id": 2, "name": "Gray Berry"}
            ]
        });

    let s = selector("$.school", "./benches/data_obj.json");
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.school[?(@.friends[0])]", "./benches/data_obj.json");
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.school[?(@.friends[10])]", "./benches/data_obj.json");
    assert_eq!(Value::Null, s.select_as_value().unwrap());

    let s = selector("$.school[?(1==1)]", "./benches/data_obj.json");
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.school.friends[?(1==1)]", "./benches/data_obj.json");
    let friends = json!([
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]);
    assert_eq!(friends, s.select_as_value().unwrap());
}

#[test]
fn op_default() {
    setup();

    let s = selector("$.school[?(@.friends == @.friends)]", "./benches/data_obj.json");
    let friends = json!({
        "friends": [
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]
    });
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.friends[?(@.name)]", "./benches/data_obj.json");
    let friends = json!([
            { "id" : 1, "name" : "Vincent Cannon" },
            { "id" : 2, "name" : "Gray Berry" }
        ]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.friends[?(@.id >= 2)]", "./benches/data_obj.json");
    let friends = json!([
            { "id" : 2, "name" : "Gray Berry" }
        ]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.friends[?(@.id >= 2 || @.id == 1)]", "./benches/data_obj.json");
    let friends = json!([
            { "id" : 2, "name" : "Gray Berry" },
            { "id" : 1, "name" : "Vincent Cannon" }
        ]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$.friends[?( (@.id >= 2 || @.id == 1) && @.id == 0)]", "./benches/data_obj.json");
    assert_eq!(Value::Null, s.select_as_value().unwrap());

    let s = selector("$..friends[?(@.id == $.index)].id", "./benches/data_obj.json");
    let friends = json!([0, 0]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$..book[?($.store.bicycle.price < @.price)].price", "./benches/example.json");
    let friends = json!([22.99]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let s = selector("$..book[?( (@.price == 12.99 || @.category == 'reference') && @.price > 10)].price", "./benches/example.json");
    let friends = json!([12.99]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let ref value = json!([
        { "name": "이름1", "age": 40, "phone": "+33 12341234" },
        { "name": "이름2", "age": 42, "phone": "++44 12341234" }
    ]);
    
    let mut s = Selector::new();
    let _ = s.path("$..[?(@.age > 40)]");
    let _ = s.value(value);
    let friends = json!([
       { "name" : "이름2", "age" : 42, "phone" : "++44 12341234" }
    ]);
    assert_eq!(friends, s.select_as_value().unwrap());

    let ref value = json!({
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
    let mut s = Selector::new();
    let _ = s.path("$..[?(@.age >= 30)]");
    let _ = s.value(value);
    let friends = json!([{ "name" : "친구3", "age" : 30 }]);
    assert_eq!(friends, s.select_as_value().unwrap());
}

#[test]
fn op_number() {
    setup();

    let json = json!({ "a": 1 });
    let ret = jsonpath::select(&json, "$.[?(@.a == 1)]").unwrap();
    assert_eq!(json, ret);
    let ret = jsonpath::select(&json, "$.[?(@.a != 2)]").unwrap();
    assert_eq!(json, ret);
    let ret = jsonpath::select(&json, "$.[?(@.a < 2)]").unwrap();
    assert_eq!(json, ret);
    let ret = jsonpath::select(&json, "$.[?(@.a <= 1)]").unwrap();
    assert_eq!(json, ret);
    let ret = jsonpath::select(&json, "$.[?(@.a > 0)]").unwrap();
    assert_eq!(json, ret);
    let ret = jsonpath::select(&json, "$.[?(@.a >= 0)]").unwrap();
    assert_eq!(json, ret);
}

#[test]
fn op_string() {
    setup();

    let json = json!({ "a": "b" });
    let ret = jsonpath::select(&json, r#"$.[?(@.a == "b")]"#).unwrap();
    assert_eq!(json!({ "a": "b" }), ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a != "c")]"#).unwrap();
    assert_eq!(json!({ "a": "b" }), ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a < "b")]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a <= "b")]"#).unwrap();
    assert_eq!(json!({ "a": "b" }), ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a > "b")]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a >= "b")]"#).unwrap();
    assert_eq!(json!({ "a": "b" }), ret);
}

#[test]
fn op_object() {
    setup();

    let json = json!({
        "a": { "1": 1 },
        "b": { "2": 2 },
        "c": { "1": 1 },
    });
    let ret = jsonpath::select(&json, r#"$.[?(@.a == @.c)]"#).unwrap();
    assert_eq!(json, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a != @.c)]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a < @.c)]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a <= @.c)]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a > @.c)]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a >= @.c)]"#).unwrap();
    assert_eq!(Value::Null, ret);
}

#[test]
fn op_complex() {
    setup();

    let json = json!({ "a": { "b": 1 } });
    let ret = jsonpath::select(&json, r#"$.[?(1 == @.a)]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?("1" != @.a)]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a <= 1)]"#).unwrap();
    assert_eq!(Value::Null, ret);
    let ret = jsonpath::select(&json, r#"$.[?(@.a > "1")]"#).unwrap();
    assert_eq!(Value::Null, ret);
}

#[test]
fn example() {
    setup();

    let s = selector("$.store.book[*].author", "./benches/example.json");
    let ret = json!(["Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$..author", "./benches/example.json");
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$.store.*", "./benches/example.json");
    let ret = json!([
        [
            {"category" : "reference", "author" : "Nigel Rees","title" : "Sayings of the Century", "price" : 8.95},
            {"category" : "fiction", "author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99},
            {"category" : "fiction", "author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99},
            {"category" : "fiction", "author" : "J. R. R. Tolkien","title" : "The Lord of the Rings","isbn" : "0-395-19395-8","price" : 22.99}
        ],
        {"color" : "red","price" : 19.95},
        ]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$.store..price", "./benches/example.json");
    let ret = json!([8.95, 12.99, 8.99, 22.99, 19.95]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$..book[2]", "./benches/example.json");
    let ret = json!([{
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
        }]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$..book[-2]", "./benches/example.json");
    let ret = json!([{
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
        }]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$..book[0,1]", "./benches/example.json");
    let ret = json!([
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
        ]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$..book[:2]", "./benches/example.json");
    let ret = json!([
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
        ]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$..book[2:]", "./benches/example.json");
    let ret = json!([
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
        ]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$..book[?(@.isbn)]", "./benches/example.json");
    let ret = json!([
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
        ]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$.store.book[?(@.price < 10)]", "./benches/example.json");
    let ret = json!([
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
        ]);
    assert_eq!(ret, s.select_as_value().unwrap());

    let s = selector("$..*", "./benches/example.json");
    let json: Value = serde_json::from_str(read_json("./benches/giveme_every_thing_result.json").as_str()).unwrap();
    assert_eq!(json, s.select_as_value().unwrap());
}

#[test]
fn filer_same_obj() {
    setup();

    let mut s = Selector::new();
    let _ = s.path("$..[?(@.a == 1)]");
    let _ = s.value_from_str(r#"
    {
        "a": 1,
        "b" : {"a": 1},
        "c" : {"a": 1}
    }
    "#);
    assert_eq!(s.select_as_value().unwrap(), json!([
        {"a": 1},
        {"a": 1}
    ]));
}