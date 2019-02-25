mod cmp;
mod term;
mod value_filter;
mod value_wrapper;


#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::io::Read;
    use std::sync::{Once, ONCE_INIT};

    use jsonpath::parser::Parser;

    use serde_json::Value;

    use super::value_filter::*;

    static INIT: Once = ONCE_INIT;

    fn setup() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    fn new_value_filter(file: &str) -> ValueFilter {
        let string = read_json(file);
        let json: Value = serde_json::from_str(string.as_str()).unwrap();
        ValueFilter::new(json, false, false)
    }

    fn do_filter(path: &str, file: &str) -> JsonValueFilter {
        let string = read_json(file);
        let mut jf = JsonValueFilter::new(string.as_str()).unwrap();
        let mut parser = Parser::new(path);
        parser.parse(&mut jf).unwrap();
        jf
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
            assert_eq!(&friends, current.get_val());
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
            assert_eq!(&names, current.get_val());
        }
    }

    #[test]
    fn array() {
        setup();

        let friends = json!([
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]);

        let jf = do_filter("$.school.friends[1, 2]", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school.friends[1:]", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school.friends[:-2]", "./benches/data_obj.json");
        let friends = json!([
            {"id": 0, "name": "Millicent Norman"}
        ]);
        assert_eq!(&friends, jf.current_value());
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

        let jf = do_filter("$.school", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school[?(@.friends[0])]", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school[?(@.friends[10])]", "./benches/data_obj.json");
        assert_eq!(&Value::Null, jf.current_value());

        let jf = do_filter("$.school[?(1==1)]", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school.friends[?(1==1)]", "./benches/data_obj.json");
        let friends = json!([
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]);
        assert_eq!(&friends, jf.current_value());
    }

    #[test]
    fn op() {
        setup();

        let jf = do_filter("$.school[?(@.friends == @.friends)]", "./benches/data_obj.json");
        let friends = json!({
            "friends": [
                {"id": 0, "name": "Millicent Norman"},
                {"id": 1, "name": "Vincent Cannon" },
                {"id": 2, "name": "Gray Berry"}
            ]
        });
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.friends[?(@.name)]", "./benches/data_obj.json");
        let friends = json!([
            { "id" : 1, "name" : "Vincent Cannon" },
            { "id" : 2, "name" : "Gray Berry" }
        ]);
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.friends[?(@.id >= 2)]", "./benches/data_obj.json");
        let friends = json!([
            { "id" : 2, "name" : "Gray Berry" }
        ]);
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.friends[?(@.id >= 2 || @.id == 1)]", "./benches/data_obj.json");
        let friends = json!([
            { "id" : 2, "name" : "Gray Berry" },
            { "id" : 1, "name" : "Vincent Cannon" }
        ]);
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.friends[?( (@.id >= 2 || @.id == 1) && @.id == 0)]", "./benches/data_obj.json");
        assert_eq!(&Value::Null, jf.current_value());
    }

    #[test]
    fn example() {
        setup();

        let jf = do_filter("$.store.book[*].author", "./benches/example.json");
        let ret = json!(["Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"]);
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$..author", "./benches/example.json");
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$.store.*", "./benches/example.json");
        let ret = json!([
        [
            {"category" : "reference", "author" : "Nigel Rees","title" : "Sayings of the Century", "price" : 8.95},
            {"category" : "fiction", "author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99},
            {"category" : "fiction", "author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99},
            {"category" : "fiction", "author" : "J. R. R. Tolkien","title" : "The Lord of the Rings","isbn" : "0-395-19395-8","price" : 22.99}
        ],
        {"color" : "red","price" : 19.95},
        ]);
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$.store..price", "./benches/example.json");
        let ret = json!([8.95, 12.99, 8.99, 22.99, 19.95]);
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$..book[2]", "./benches/example.json");
        let ret = json!([{
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
        }]);
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$..book[-2]", "./benches/example.json");
        let ret = json!([{
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
        }]);
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$..book[0,1]", "./benches/example.json");
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
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$..book[:2]", "./benches/example.json");
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
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$..book[2:]", "./benches/example.json");
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
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$..book[?(@.isbn)]", "./benches/example.json");
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
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$.store.book[?(@.price < 10)]", "./benches/example.json");
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
        assert_eq!(&ret, jf.current_value());

        let jf = do_filter("$..*", "./benches/example.json");
        let json: Value = serde_json::from_str(read_json("./benches/giveme_every_thing_result.json").as_str()).unwrap();
        assert_eq!(&json, jf.current_value());
    }
}