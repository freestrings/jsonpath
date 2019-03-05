//! JSONPath implementation for Rust
//!
//! # Example
//! ```
//! extern crate jsonpath;
//! #[macro_use]
//! extern crate serde_json;
//!
//! let json_obj = json!({
//!    "store": {
//!        "book": [
//!            {
//!                "category": "reference",
//!                "author": "Nigel Rees",
//!                "title": "Sayings of the Century",
//!                "price": 8.95
//!            },
//!            {
//!                "category": "fiction",
//!                "author": "Evelyn Waugh",
//!                "title": "Sword of Honour",
//!                "price": 12.99
//!            },
//!            {
//!                "category": "fiction",
//!                "author": "Herman Melville",
//!                "title": "Moby Dick",
//!                "isbn": "0-553-21311-3",
//!                "price": 8.99
//!            },
//!            {
//!                "category": "fiction",
//!                "author": "J. R. R. Tolkien",
//!                "title": "The Lord of the Rings",
//!                "isbn": "0-395-19395-8",
//!                "price": 22.99
//!            }
//!        ],
//!        "bicycle": {
//!            "color": "red",
//!            "price": 19.95
//!        }
//!    },
//!    "expensive": 10
//!  });
//!
//! let mut reader = jsonpath::reader(json_obj);
//!
//! //
//! // $.store.book[*].author
//! //
//! let json = reader("$.store.book[*].author");
//! let ret = json!(["Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"]);
//! assert_eq!(json, ret);
//!
//! //
//! // $..author
//! //
//! let json = reader("$..author");
//! let ret = json!(["Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"]);
//! assert_eq!(json, ret);
//!
//! //
//! // $.store.*
//! //
//! let json = reader("$.store.*");
//! let ret = json!(["Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"]);
//! let ret = json!([
//! [
//!     {"category" : "reference", "author" : "Nigel Rees","title" : "Sayings of the Century", "price" : 8.95},
//!     {"category" : "fiction", "author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99},
//!     {"category" : "fiction", "author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99},
//!     {"category" : "fiction", "author" : "J. R. R. Tolkien","title" : "The Lord of the Rings","isbn" : "0-395-19395-8","price" : 22.99}
//! ],
//! {"color" : "red","price" : 19.95},
//! ]);
//! assert_eq!(ret, json);
//!
//! //
//! // $.store..price
//! //
//! let json = reader("$.store..price");
//! let ret = json!([8.95, 12.99, 8.99, 22.99, 19.95]);
//! assert_eq!(ret, json);
//!
//! //
//! // $..book[2]
//! //
//! let json = reader("$..book[2]");
//! let ret = json!([{
//!     "category" : "fiction",
//!     "author" : "Herman Melville",
//!     "title" : "Moby Dick",
//!     "isbn" : "0-553-21311-3",
//!     "price" : 8.99
//! }]);
//! assert_eq!(ret, json);
//!
//! //
//! // $..book[-2]
//! //
//! let json = reader("$..book[-2]");
//! let ret = json!([{
//!     "category" : "fiction",
//!     "author" : "Herman Melville",
//!     "title" : "Moby Dick",
//!     "isbn" : "0-553-21311-3",
//!     "price" : 8.99
//! }]);
//! assert_eq!(ret, json);
//!
//! //
//! // $..book[0,1]
//! //
//! let json = reader("$..book[0,1]");
//! let ret = json!([
//! {
//!     "category" : "reference",
//!     "author" : "Nigel Rees",
//!     "title" : "Sayings of the Century",
//!     "price" : 8.95
//! },
//! {
//!     "category" : "fiction",
//!     "author" : "Evelyn Waugh",
//!     "title" : "Sword of Honour",
//!     "price" : 12.99
//! }
//! ]);
//! assert_eq!(ret, json);
//!
//! //
//! // $..book[:2]
//! //
//! let json = reader("$..book[:2]");
//! let ret = json!([
//! {
//!     "category" : "reference",
//!     "author" : "Nigel Rees",
//!     "title" : "Sayings of the Century",
//!     "price" : 8.95
//! },
//! {
//!     "category" : "fiction",
//!     "author" : "Evelyn Waugh",
//!     "title" : "Sword of Honour",
//!     "price" : 12.99
//! }
//! ]);
//! assert_eq!(ret, json);
//!
//! //
//! // $..book[2:]
//! //
//! let json = reader("$..book[2:]");
//! let ret = json!([
//! {
//!     "category" : "fiction",
//!     "author" : "Herman Melville",
//!     "title" : "Moby Dick",
//!     "isbn" : "0-553-21311-3",
//!     "price" : 8.99
//! },
//! {
//!     "category" : "fiction",
//!     "author" : "J. R. R. Tolkien",
//!     "title" : "The Lord of the Rings",
//!     "isbn" : "0-395-19395-8",
//!     "price" : 22.99
//! }
//! ]);
//! assert_eq!(ret, json);
//!
//! //
//! // $..book[?(@.isbn)]
//! //
//! let json = reader("$..book[?(@.isbn)]");
//! let ret = json!([
//! {
//!     "category" : "fiction",
//!     "author" : "Herman Melville",
//!     "title" : "Moby Dick",
//!     "isbn" : "0-553-21311-3",
//!     "price" : 8.99
//! },
//! {
//!     "category" : "fiction",
//!     "author" : "J. R. R. Tolkien",
//!     "title" : "The Lord of the Rings",
//!     "isbn" : "0-395-19395-8",
//!     "price" : 22.99
//! }
//! ]);
//! assert_eq!(ret, json);
//!
//! //
//! // $.store.book[?(@.price < 10)]
//! //
//! let json = reader("$.store.book[?(@.price < 10)]");
//! let ret = json!([
//! {
//!     "category" : "reference",
//!     "author" : "Nigel Rees",
//!     "title" : "Sayings of the Century",
//!     "price" : 8.95
//! },
//! {
//!     "category" : "fiction",
//!     "author" : "Herman Melville",
//!     "title" : "Moby Dick",
//!     "isbn" : "0-553-21311-3",
//!     "price" : 8.99
//! }
//! ]);
//! assert_eq!(ret, json);
//!
//! ```
//!
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_json;
#[cfg(not(test))]
extern crate serde_json;

extern crate core;
extern crate indexmap;

#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod filter;

use parser::parser::*;
use filter::value_filter::*;

use std::result;
use serde_json::Value;

type Result = result::Result<Value, String>;

/// # Read multiple Json multiple times with the same JsonPath
///
/// ```rust
/// let mut template = jsonpath::compile("$..friends[0]");
///
///
/// let json_obj = json!({
/// "school": {
///    "friends": [ {"id": 0}, {"id": 1} ]
/// },
/// "friends": [ {"id": 0}, {"id": 1} ]
/// });
///
/// let json = template(json_obj).unwrap();
/// let ret = json!([ {"id": 0}, {"id": 0} ]);
/// assert_eq!(json, ret);
///
///
/// let json_obj = json!({
/// "school": {
///    "friends": [ {"name": "Millicent Norman"}, {"name": "Vincent Cannon"} ]
/// },
/// "friends": [ {"id": 0}, {"id": 1} ]
/// });
///
/// let json = template(json_obj).unwrap();
/// let ret = json!([ {"id": 0}, {"name": "Millicent Norman"} ]);
/// assert_eq!(json, ret);
/// ```
pub fn compile<'a>(path: &'a str) -> impl FnMut(Value) -> Result + 'a {
    let mut parser = Parser::new(path);
    let node = parser.compile();
    move |json| {
        match &node {
            Ok(n) => {
                let mut jf = JsonValueFilter::new_from_value(json);
                jf.visit(n.clone());
                Ok(jf.take_value())
            }
            Err(e) => Err(e.clone())
        }
    }
}


/// # Read the same Json multiple times using different JsonPath
///
/// ```rust
/// let json_obj = json!({
/// "school": {
///    "friends": [{"id": 0}, {"id": 1}]
/// },
/// "friends": [{"id": 0},{"id": 1}]
/// });
///
/// let mut reader = jsonpath::reader(json_obj);
///
/// let json = reader("$..friends[0]").unwrap();
/// let ret = json!([ {"id": 0}, {"id": 0} ]);
/// assert_eq!(json, ret);
///
/// let json = reader("$..friends[1]").unwrap();
/// let ret = json!([ {"id": 1}, {"id": 1} ]);
/// assert_eq!(json, ret);
/// ```
pub fn reader(json: Value) -> impl FnMut(&str) -> Result {
    let mut jf = JsonValueFilter::new_from_value(json);
    move |path: &str| {
        let mut parser = Parser::new(path);
        parser.parse(&mut jf)?;
        Ok(jf.take_value())
    }
}

/// # Read Json using JsonPath
///
/// ```rust
/// let json_obj = json!({
/// "school": {
///    "friends": [{"id": 0}, {"id": 1}]
/// },
/// "friends": [{"id": 0}, {"id": 1}]
/// });
/// let mut reader = jsonpath::read(json_obj, "$..friends[0]");
/// let ret = json!([ {"id": 0}, {"id": 0} ]);
/// assert_eq!(json, ret);
/// ```
pub fn read(json: Value, path: &str) -> Result {
    let mut jf = JsonValueFilter::new_from_value(json);
    let mut parser = Parser::new(path);
    parser.parse(&mut jf)?;
    Ok(jf.take_value())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Read;

    fn read_json(path: &str) -> Value {
        let mut f = std::fs::File::open(path).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();
        serde_json::from_str(contents.as_str()).unwrap()
    }

    #[test]
    fn compile() {
        let mut template = super::compile("$..friends[2]");
        let json_obj = read_json("./benches/data_obj.json");
        let json = template(json_obj).unwrap();
        let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Gray Berry"}
        ]);
        assert_eq!(json, ret);

        let json_obj = read_json("./benches/data_array.json");
        let json = template(json_obj).unwrap();
        let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Rosetta Erickson"}
        ]);
        assert_eq!(json, ret);
    }

    #[test]
    fn reader() {
        let json_obj = read_json("./benches/data_obj.json");
        let mut reader = super::reader(json_obj);
        let json = reader("$..friends[2]").unwrap();
        let ret = json!([
            {"id": 2,"name": "Gray Berry"},
            {"id": 2,"name": "Gray Berry"}
        ]);
        assert_eq!(json, ret);

        let json = reader("$..friends[0]").unwrap();
        let ret = json!([
            {"id": 0},
            {"id": 0,"name": "Millicent Norman"}
        ]);
        assert_eq!(json, ret);
    }

    #[test]
    fn read() {
        let json_obj = read_json("./benches/example.json");
        let json = super::read(json_obj, "$..book[2]").unwrap();
        let ret = json!([{
            "category" : "fiction",
            "author" : "Herman Melville",
            "title" : "Moby Dick",
            "isbn" : "0-553-21311-3",
            "price" : 8.99
        }]);
        assert_eq!(json, ret);
    }
}