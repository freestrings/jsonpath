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

extern crate indexmap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
extern crate serde_json;

use std::ops::Deref;
use std::result;

use serde_json::Value;

use filter::value_filter::JsonValueFilter;
use parser::parser::{NodeVisitor, Parser};
use ref_value::model::{RefValue, RefValueWrapper};

#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod filter;
#[doc(hidden)]
pub mod ref_value;

/// # Compile a Jsonpath so it select a JsonObject immediately.
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
pub fn compile<'a>(path: &'a str) -> impl FnMut(&Value) -> result::Result<Value, String> + 'a {
    let mut parser = Parser::new(path);
    let node = parser.compile();
    move |json| {
        match &node {
            Ok(n) => {
                let mut jf = JsonValueFilter::new_from_value(json.into());
                jf.visit(n.clone());
                Ok(jf.take_value().into())
            }
            Err(e) => Err(e.clone())
        }
    }
}


/// # Use multiple JsonPaths for one JsonObject.
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
pub fn selector(json: &Value) -> impl FnMut(&str) -> result::Result<Value, String> {
    let wrapper: RefValueWrapper = json.into();
    move |path: &str| {
        let mut jf = JsonValueFilter::new_from_value(wrapper.clone());
        let mut parser = Parser::new(path);
        parser.parse(&mut jf)?;
        Ok(jf.take_value().into())
    }
}

#[deprecated(since = "0.1.4", note = "Please use the selector function instead")]
pub fn reader(json: &Value) -> impl FnMut(&str) -> result::Result<Value, String> {
    selector(json)
}

/// # Select a JsonObject
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
pub fn select(json: &Value, path: &str) -> result::Result<Value, String> {
    let mut jf = JsonValueFilter::new_from_value(json.into());
    let mut parser = Parser::new(path);
    parser.parse(&mut jf)?;
    Ok(jf.take_value().into())
}

#[deprecated(since = "0.1.4", note = "Please use the select function instead")]
pub fn read(json: &Value, path: &str) -> result::Result<Value, String> {
    select(json, path)
}

#[deprecated(since = "0.1.7", note = "Please use the select_as_str function instead")]
pub fn select_str(json: &str, path: &str) -> result::Result<String, String> {
    select_as_str(json, path)
}

/// # Return to json string
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let ret = jsonpath::select_as_str(r#"{
///     "school": { "friends": [{"id": 0}, {"id": 1}] },
///     "friends": [{"id": 0}, {"id": 1}]
/// }"#, "$..friends[0]").unwrap();
/// assert_eq!(ret, r#"[{"id":0},{"id":0}]"#);
/// ```
pub fn select_as_str(json: &str, path: &str) -> result::Result<String, String> {
    let ref_value: RefValue = serde_json::from_str(json).map_err(|e| format!("{:?}", e))?;
    let mut jf = JsonValueFilter::new_from_value(ref_value.into());
    let mut parser = Parser::new(path);
    parser.parse(&mut jf)?;
    serde_json::to_string(&jf.take_value().deref()).map_err(|e| format!("{:?}", e))
}

/// # Return to deserializeable.
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// extern crate serde;
/// #[macro_use] extern crate serde_json;
///
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct Person {
///     name: String,
///     age: u8,
///     phones: Vec<String>,
/// }
///
/// let ret: Person = jsonpath::select_as(r#"
/// {
///     "person":
///         {
///             "name": "Doe John",
///             "age": 44,
///             "phones": [
///                 "+44 1234567",
///                 "+44 2345678"
///             ]
///         }
/// }
/// "#, "$.person").unwrap();
///
/// let person = Person {
///     name: "Doe John".to_string(),
///     age: 44,
///     phones: vec!["+44 1234567".to_string(), "+44 2345678".to_string()],
/// };
///
/// assert_eq!(person, ret);
/// ```
pub fn select_as<'a, T: serde::Deserialize<'a>>(json: &str, path: &str) -> result::Result<T, String> {
    let ref_value: RefValue = serde_json::from_str(json).map_err(|e| format!("{:?}", e))?;
    let mut jf = JsonValueFilter::new_from_value(ref_value.into());
    let mut parser = Parser::new(path);
    parser.parse(&mut jf)?;
    T::deserialize(jf.take_value().deref()).map_err(|e| format!("{:?}", e))
}