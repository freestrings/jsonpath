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

extern crate core;
extern crate env_logger;
extern crate indexmap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
extern crate serde_json;

use core::borrow::BorrowMut;
use std::result;

use serde_json::Value;

#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod filter;
#[doc(hidden)]
pub mod ref_value;
#[doc(hidden)]
pub mod select;

pub use select::Selector;

/// It is a high-order function. it compile a JsonPath and then returns a function. this return-function can be reused for different JsonObjects.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let mut template = jsonpath::compile("$..friends[0]");
///
/// let json_obj = json!({
/// "school": {
///    "friends": [
///         {"name": "친구1", "age": 20},
///         {"name": "친구2", "age": 20}
///     ]
/// },
/// "friends": [
///     {"name": "친구3", "age": 30},
///     {"name": "친구4"}
/// ]});
///
/// let json = template(&json_obj).unwrap();
/// let ret = json!([
///     {"name": "친구3", "age": 30},
///     {"name": "친구1", "age": 20}
/// ]);
/// assert_eq!(json, ret);
/// ```
pub fn compile<'a>(path: &'a str) -> impl FnMut(&Value) -> result::Result<Value, String> + 'a {
    let mut selector = Selector::new();
    let _ = selector.path(path);
    let mut selector = Box::new(selector);
    move |json| {
        let s: &mut Selector = selector.borrow_mut();
        let _ = s.value(&json);
        s.select_as_value()
    }
}

/// It is a high-order function that return a function. this return-function has a jsonpath as argument and return a serde_json::value::Value. so you can use different JsonPath for one JsonObject.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let json_obj = json!({
/// "school": {
///    "friends": [
///         {"name": "친구1", "age": 20},
///         {"name": "친구2", "age": 20}
///     ]
/// },
/// "friends": [
///     {"name": "친구3", "age": 30},
///     {"name": "친구4"}
/// ]});
///
/// let mut selector = jsonpath::selector(&json_obj);
///
/// let json = selector("$..friends[0]").unwrap();
/// let ret = json!([
///     {"name": "친구3", "age": 30},
///     {"name": "친구1", "age": 20}
/// ]);
/// assert_eq!(json, ret);
///
/// let json = selector("$..friends[1]").unwrap();
/// let ret = json!([
///     {"name": "친구4"},
///     {"name": "친구2", "age": 20}
/// ]);
/// assert_eq!(json, ret);
/// ```
pub fn selector<'a>(json: &Value) -> impl FnMut(&'a str) -> result::Result<Value, String> {
    let mut selector = Selector::new();
    let _ = selector.value(json.into());
    let mut selector = Box::new(selector);
    move |path: &'a str| {
        let s: &mut Selector = selector.borrow_mut();
        s.path(path)?.select_as_value()
    }
}

/// It is a high-order function that returns a function. this return-function has a jsonpath as argument and return a serde::Deserialize. so you can use different JsonPath for one JsonObject.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// extern crate serde;
/// #[macro_use] extern crate serde_json;
///
/// use serde::{Deserialize, Serialize};
///
/// let json_obj = json!({
/// "school": {
///    "friends": [
///         {"name": "친구1", "age": 20},
///         {"name": "친구2", "age": 20}
///     ]
/// },
/// "friends": [
///     {"name": "친구3", "age": 30},
///     {"name": "친구4"}
/// ]});
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Friend {
///     name: String,
///     age: Option<u8>,
/// }
///
/// let mut selector = jsonpath::selector_as::<Vec<Friend>>(&json_obj);
///
/// let json = selector("$..friends[0]").unwrap();
/// let ret = vec!(
///     Friend { name: "친구3".to_string(), age: Some(30) },
///     Friend { name: "친구1".to_string(), age: Some(20) }
/// );
/// assert_eq!(json, ret);
///
/// let json = selector("$..friends[1]").unwrap();
/// let ret = vec!(
///     Friend { name: "친구4".to_string(), age: None },
///     Friend { name: "친구2".to_string(), age: Some(20) }
/// );
/// assert_eq!(json, ret);
/// ```
pub fn selector_as<T: serde::de::DeserializeOwned>(json: &Value) -> impl FnMut(&str) -> result::Result<T, String> {
    let mut selector = Selector::new();
    let _ = selector.value(json.into());
    move |path: &str| {
        selector.path(path)?.select_as()
    }
}

#[deprecated(since = "0.1.4", note = "Please use the selector function instead")]
pub fn reader<'a>(json: &Value) -> impl FnMut(&'a str) -> result::Result<Value, String> {
    selector(json)
}

/// This function compile a jsonpath everytime and it convert `serde_json's Value` to `jsonpath's RefValue` everytime and then it return a `serde_json::value::Value`.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let json_obj = json!({
/// "school": {
///    "friends": [
///         {"name": "친구1", "age": 20},
///         {"name": "친구2", "age": 20}
///     ]
/// },
/// "friends": [
///     {"name": "친구3", "age": 30},
///     {"name": "친구4"}
/// ]});
///
/// let json = jsonpath::select(&json_obj, "$..friends[0]").unwrap();
///
/// let ret = json!([
///     {"name": "친구3", "age": 30},
///     {"name": "친구1", "age": 20}
/// ]);
/// assert_eq!(json, ret);
/// ```
pub fn select(json: &Value, path: &str) -> result::Result<Value, String> {
    let mut selector = Selector::new();
    selector.path(path)?.value(json.into())?.select_as_value()
}

#[deprecated(since = "0.1.4", note = "Please use the select function instead")]
pub fn read(json: &Value, path: &str) -> result::Result<Value, String> {
    select(json, path)
}

#[deprecated(since = "0.1.7", note = "Please use the select_as_str function instead")]
pub fn select_str(json: &str, path: &str) -> result::Result<String, String> {
    select_as_str(json, path)
}

/// This function compile a jsonpath everytime and it convert `&str` to `jsonpath's RefValue` everytime and then it return a json string.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let ret = jsonpath::select_as_str(r#"
/// {
///     "school": {
///         "friends": [
///                 {"name": "친구1", "age": 20},
///                 {"name": "친구2", "age": 20}
///             ]
///     },
///     "friends": [
///         {"name": "친구3", "age": 30},
///         {"name": "친구4"}
///     ]
/// }
/// "#, "$..friends[0]").unwrap();
///
/// assert_eq!(ret, r#"[{"name":"친구3","age":30},{"name":"친구1","age":20}]"#);
/// ```
pub fn select_as_str(json: &str, path: &str) -> result::Result<String, String> {
    Selector::new()
        .path(path)?
        .value_from_str(json)?
        .select_as_str()
}

/// This function compile a jsonpath everytime and it convert `&str` to `jsonpath's RefValue` everytime and then it return a deserialized-instance of type `T`.
///
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
pub fn select_as<T: serde::de::DeserializeOwned>(json: &str, path: &str) -> result::Result<T, String> {
    Selector::new()
        .path(path)?
        .value_from_str(json)?
        .select_as()
}