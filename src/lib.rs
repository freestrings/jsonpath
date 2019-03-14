//! JsonPath implementation for Rust
//!
//! # Example
//! ```
//!    extern crate jsonpath_lib as jsonpath;
//!    #[macro_use] extern crate serde_json;
//!
//!    let json_obj = json!({
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
//!    });
//!
//!    let mut selector = jsonpath::selector(&json_obj);
//!
//!    //
//!    // $.store.book[*].author
//!    //
//!    let json = selector("$.store.book[*].author").unwrap();
//!    let ret = json!(["Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"]);
//!    assert_eq!(json, ret);
//!
//!    //
//!    // $..author
//!    //
//!    let json = selector("$..author").unwrap();
//!    let ret = json!(["Nigel Rees","Evelyn Waugh","Herman Melville","J. R. R. Tolkien"]);
//!    assert_eq!(json, ret);
//!
//!    //
//!    // $.store.*
//!    //
//!    let json = selector("$.store.*").unwrap();
//!    let ret = json!([
//!        [
//!         {"category" : "reference", "author" : "Nigel Rees","title" : "Sayings of the Century", "price" : 8.95},
//!         {"category" : "fiction", "author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99},
//!         {"category" : "fiction", "author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99},
//!         {"category" : "fiction", "author" : "J. R. R. Tolkien","title" : "The Lord of the Rings","isbn" : "0-395-19395-8","price" : 22.99}
//!        ],
//!        {"color" : "red","price" : 19.95},
//!    ]);
//!    assert_eq!(ret, json);
//!
//!    //
//!    // $.store..price
//!    //
//!    let json = selector("$.store..price").unwrap();
//!    let ret = json!([8.95, 12.99, 8.99, 22.99, 19.95]);
//!    assert_eq!(ret, json);
//!
//!    //
//!    // $..book[2]
//!    //
//!    let json = selector("$..book[2]").unwrap();
//!    let ret = json!([{
//!        "category" : "fiction",
//!        "author" : "Herman Melville",
//!        "title" : "Moby Dick",
//!        "isbn" : "0-553-21311-3",
//!        "price" : 8.99
//!    }]);
//!    assert_eq!(ret, json);
//!
//!    //
//!    // $..book[-2]
//!    //
//!    let json = selector("$..book[-2]").unwrap();
//!    let ret = json!([{
//!        "category" : "fiction",
//!        "author" : "Herman Melville",
//!        "title" : "Moby Dick",
//!        "isbn" : "0-553-21311-3",
//!        "price" : 8.99
//!     }]);
//!    assert_eq!(ret, json);
//!
//!    //
//!    // $..book[0,1]
//!    //
//!    let json = selector("$..book[0,1]").unwrap();
//!    let ret = json!([
//!        {"category" : "reference","author" : "Nigel Rees","title" : "Sayings of the Century","price" : 8.95},
//!        {"category" : "fiction","author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99}
//!    ]);
//!    assert_eq!(ret, json);
//!
//!    //
//!    // $..book[:2]
//!    //
//!    let json = selector("$..book[:2]").unwrap();
//!    let ret = json!([
//!        {"category" : "reference","author" : "Nigel Rees","title" : "Sayings of the Century","price" : 8.95},
//!        {"category" : "fiction","author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99}
//!    ]);
//!    assert_eq!(ret, json);
//!
//!    //
//!    // $..book[2:]
//!    //
//!    let json = selector("$..book[2:]").unwrap();
//!    let ret = json!([
//!        {"category" : "fiction","author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99},
//!        {"category" : "fiction","author" : "J. R. R. Tolkien","title" : "The Lord of the Rings","isbn" : "0-395-19395-8","price" : 22.99}
//!    ]);
//!    assert_eq!(ret, json);
//!
//!    //
//!    // $..book[?(@.isbn)]
//!    //
//!    let json = selector("$..book[?(@.isbn)]").unwrap();
//!    let ret = json!([
//!        {"category" : "fiction","author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99},
//!        {"category" : "fiction","author" : "J. R. R. Tolkien","title" : "The Lord of the Rings","isbn" : "0-395-19395-8","price" : 22.99}
//!    ]);
//!    assert_eq!(ret, json);
//!
//!    //
//!    // $.store.book[?(@.price < 10)]
//!    //
//!    let json = selector("$.store.book[?(@.price < 10)]").unwrap();
//!    let ret = json!([
//!        {"category" : "reference","author" : "Nigel Rees","title" : "Sayings of the Century","price" : 8.95},
//!        {"category" : "fiction","author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99}
//!    ]);
//!    assert_eq!(ret, json);
//! ```

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde_json;
extern crate indexmap;

#[doc(hidden)]
mod parser;
#[doc(hidden)]
mod filter;
#[doc(hidden)]
mod ref_value;
pub mod prelude;

use parser::prelude::*;
use filter::prelude::*;

use std::result;
use serde_json::Value;

use ref_value::*;

type Result = result::Result<Value, String>;

/// # Read multiple Json multiple times with the same JsonPath
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
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
/// let json = template(&json_obj).unwrap();
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
/// let json = template(&json_obj).unwrap();
/// let ret = json!([ {"id": 0}, {"name": "Millicent Norman"} ]);
/// assert_eq!(json, ret);
/// ```
pub fn compile<'a>(path: &'a str) -> impl FnMut(&Value) -> Result + 'a {
    let mut parser = Parser::new(path);
    let node = parser.compile();
    move |json| {
        match &node {
            Ok(n) => {
                let mut jf = JsonValueFilter::new_from_value(json.into());
                jf.visit(n.clone());
                Ok(jf.take_value().into_value())
            }
            Err(e) => Err(e.clone())
        }
    }
}


/// # Read the same Json multiple times using different JsonPath
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let json_obj = json!({
/// "school": {
///    "friends": [{"id": 0}, {"id": 1}]
/// },
/// "friends": [{"id": 0},{"id": 1}]
/// });
///
/// let mut selector = jsonpath::selector(&json_obj);
///
/// let json = selector("$..friends[0]").unwrap();
/// let ret = json!([ {"id": 0}, {"id": 0} ]);
/// assert_eq!(json, ret);
///
/// let json = selector("$..friends[1]").unwrap();
/// let ret = json!([ {"id": 1}, {"id": 1} ]);
/// assert_eq!(json, ret);
/// ```
pub fn selector(json: &Value) -> impl FnMut(&str) -> Result {
    let wrapper: RefValueWrapper = json.into();
    move |path: &str| {
        let mut jf = JsonValueFilter::new_from_value(wrapper.clone());
        let mut parser = Parser::new(path);
        parser.parse(&mut jf)?;
        Ok(jf.take_value().into_value())
    }
}

/// # Read the same Json multiple times using different JsonPath - Deprecated. use selector
pub fn reader(json: &Value) -> impl FnMut(&str) -> Result {
    selector(json)
}

/// # Read Json using JsonPath
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let json_obj = json!({
/// "school": {
///    "friends": [{"id": 0}, {"id": 1}]
/// },
/// "friends": [{"id": 0}, {"id": 1}]
/// });
/// let json = jsonpath::select(&json_obj, "$..friends[0]").unwrap();
/// let ret = json!([ {"id": 0}, {"id": 0} ]);
/// assert_eq!(json, ret);
/// ```
pub fn select(json: &Value, path: &str) -> Result {
    let mut jf = JsonValueFilter::new_from_value(json.into());
    let mut parser = Parser::new(path);
    parser.parse(&mut jf)?;
    Ok(jf.take_value().into_value())
}

/// # Read Json using JsonPath - Deprecated. use select
pub fn read(json: &Value, path: &str) -> Result {
    select(json, path)
}

