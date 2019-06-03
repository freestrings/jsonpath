//! JsonPath implementation written in Rust.
//!
//! # Example
//! ```
//! extern crate jsonpath_lib as jsonpath;
//! #[macro_use] extern crate serde_json;
//! let json_obj = json!({
//!     "store": {
//!         "book": [
//!             {
//!                 "category": "reference",
//!                 "author": "Nigel Rees",
//!                 "title": "Sayings of the Century",
//!                 "price": 8.95
//!             },
//!             {
//!                 "category": "fiction",
//!                 "author": "Evelyn Waugh",
//!                 "title": "Sword of Honour",
//!                 "price": 12.99
//!             },
//!             {
//!                 "category": "fiction",
//!                 "author": "Herman Melville",
//!                 "title": "Moby Dick",
//!                 "isbn": "0-553-21311-3",
//!                 "price": 8.99
//!             },
//!             {
//!                 "category": "fiction",
//!                 "author": "J. R. R. Tolkien",
//!                 "title": "The Lord of the Rings",
//!                 "isbn": "0-395-19395-8",
//!                 "price": 22.99
//!             }
//!         ],
//!         "bicycle": {
//!             "color": "red",
//!             "price": 19.95
//!         }
//!     },
//!     "expensive": 10
//! });
//!
//! let mut selector = jsonpath::selector(&json_obj);
//!
//! assert_eq!(selector("$.store.book[*].author").unwrap(),
//!             vec![
//!                 "Nigel Rees", "Evelyn Waugh", "Herman Melville", "J. R. R. Tolkien"
//!             ]);
//!
//! assert_eq!(selector("$..author").unwrap(),
//!             vec![
//!                 "Nigel Rees", "Evelyn Waugh", "Herman Melville", "J. R. R. Tolkien"
//!             ]);
//!
//! assert_eq!(selector("$.store.*").unwrap(),
//!             vec![
//!                 &json!([
//!                     { "category": "reference", "author": "Nigel Rees", "title": "Sayings of the Century", "price": 8.95 },
//!                     { "category": "fiction", "author": "Evelyn Waugh", "title": "Sword of Honour", "price": 12.99 },
//!                     { "category": "fiction", "author": "Herman Melville", "title": "Moby Dick", "isbn": "0-553-21311-3", "price": 8.99 },
//!                     { "category": "fiction", "author": "J. R. R. Tolkien", "title": "The Lord of the Rings", "isbn": "0-395-19395-8", "price": 22.99 }
//!                 ]),
//!                 &json!({ "color": "red", "price": 19.95 })
//!             ]);
//!
//! assert_eq!(selector("$.store..price").unwrap(),
//!             vec![
//!                 8.95, 12.99, 8.99, 22.99, 19.95
//!             ]);
//!
//! assert_eq!(selector("$..book[2]").unwrap(),
//!             vec![
//!                 &json!({
//!                     "category" : "fiction",
//!                     "author" : "Herman Melville",
//!                     "title" : "Moby Dick",
//!                     "isbn" : "0-553-21311-3",
//!                     "price" : 8.99
//!                 })
//!             ]);
//!
//! assert_eq!(selector("$..book[-2]").unwrap(),
//!             vec![
//!                 &json!({
//!                     "category" : "fiction",
//!                     "author" : "Herman Melville",
//!                     "title" : "Moby Dick",
//!                     "isbn" : "0-553-21311-3",
//!                     "price" : 8.99
//!                 })
//!             ]);
//!
//! assert_eq!(selector("$..book[0,1]").unwrap(),
//!             vec![
//!                 &json!({"category" : "reference","author" : "Nigel Rees","title" : "Sayings of the Century","price" : 8.95}),
//!                 &json!({"category" : "fiction","author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99})
//!             ]);
//!
//! assert_eq!(selector("$..book[:2]").unwrap(),
//!             vec![
//!                 &json!({"category" : "reference","author" : "Nigel Rees","title" : "Sayings of the Century","price" : 8.95}),
//!                 &json!({"category" : "fiction","author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99})
//!             ]);
//!
//! assert_eq!(selector("$..book[:2]").unwrap(),
//!             vec![
//!                 &json!({"category" : "reference","author" : "Nigel Rees","title" : "Sayings of the Century","price" : 8.95}),
//!                 &json!({"category" : "fiction","author" : "Evelyn Waugh","title" : "Sword of Honour","price" : 12.99})
//!             ]);
//!
//! assert_eq!(selector("$..book[?(@.isbn)]").unwrap(),
//!             vec![
//!                 &json!({"category" : "fiction","author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99}),
//!                 &json!({"category" : "fiction","author" : "J. R. R. Tolkien","title" : "The Lord of the Rings","isbn" : "0-395-19395-8","price" : 22.99})
//!             ]);
//!
//! assert_eq!(selector("$.store.book[?(@.price < 10)]").unwrap(),
//!             vec![
//!                 &json!({"category" : "reference","author" : "Nigel Rees","title" : "Sayings of the Century","price" : 8.95}),
//!                 &json!({"category" : "fiction","author" : "Herman Melville","title" : "Moby Dick","isbn" : "0-553-21311-3","price" : 8.99})
//!             ]);
//! ```
extern crate array_tool;
extern crate core;
extern crate env_logger;
extern crate indexmap;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;

use serde_json::Value;

#[doc(hidden)]
mod parser;
#[doc(hidden)]
mod select;

pub use select::Selector;
pub use select::JsonPathError;
pub use parser::parser::{Node, Parser};

/// It is a high-order function. it compile a JsonPath and then returns a function. this return-function can be reused for different JsonObjects.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let mut template = jsonpath::compile("$..friends[0]");
///
/// let json_obj = json!({
///     "school": {
///         "friends": [
///             {"name": "친구1", "age": 20},
///             {"name": "친구2", "age": 20}
///         ]
///     },
///     "friends": [
///         {"name": "친구3", "age": 30},
///         {"name": "친구4"}
/// ]});
///
/// let json = template(&json_obj).unwrap();
///
/// assert_eq!(json, vec![
///     &json!({"name": "친구3", "age": 30}),
///     &json!({"name": "친구1", "age": 20})
/// ]);
/// ```
pub fn compile(path: &str) -> impl FnMut(&Value) -> Result<Vec<&Value>, JsonPathError> {
    let mut parser = Parser::new(path);
    let node = parser.compile();
    move |json| {
        let mut selector = Selector::new();
        match &node {
            Ok(node) => selector.compiled_path(node.clone()),
            Err(e) => return Err(JsonPathError::Path(e.clone()))
        };
        selector.value(json).select()
    }
}

/// It is a high-order function that return a function. this return-function has a jsonpath as argument and return a serde_json::value::Value. so you can use different JsonPath for one JsonObject.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let json_obj = json!({
///     "school": {
///         "friends": [
///             {"name": "친구1", "age": 20},
///             {"name": "친구2", "age": 20}
///         ]
///     },
///     "friends": [
///         {"name": "친구3", "age": 30},
///         {"name": "친구4"}
/// ]});
///
/// let mut selector = jsonpath::selector(&json_obj);
///
/// let json = selector("$..friends[0]").unwrap();
///
/// assert_eq!(json, vec![
///     &json!({"name": "친구3", "age": 30}),
///     &json!({"name": "친구1", "age": 20})
/// ]);
///
/// let json = selector("$..friends[1]").unwrap();
///
/// assert_eq!(json, vec![
///     &json!({"name": "친구4"}),
///     &json!({"name": "친구2", "age": 20})
/// ]);
/// ```
pub fn selector<'a>(json: &'a Value) -> impl FnMut(&'a str) -> Result<Vec<&Value>, JsonPathError> {
    let mut selector = Selector::new();
    let _ = selector.value(json);
    move |path: &str| {
        selector.path(path)?.reset_value().select()
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
///     "school": {
///         "friends": [
///             {"name": "친구1", "age": 20},
///             {"name": "친구2", "age": 20}
///         ]
///     },
///     "friends": [
///         {"name": "친구3", "age": 30},
///         {"name": "친구4"}
/// ]});
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct Friend {
///     name: String,
///     age: Option<u8>,
/// }
///
/// let mut selector = jsonpath::selector_as::<Friend>(&json_obj);
///
/// let json = selector("$..friends[0]").unwrap();
///
/// let ret = vec!(
///     Friend { name: "친구3".to_string(), age: Some(30) },
///     Friend { name: "친구1".to_string(), age: Some(20) }
/// );
/// assert_eq!(json, ret);
///
/// let json = selector("$..friends[1]").unwrap();
///
/// let ret = vec!(
///     Friend { name: "친구4".to_string(), age: None },
///     Friend { name: "친구2".to_string(), age: Some(20) }
/// );
///
/// assert_eq!(json, ret);
/// ```
pub fn selector_as<T: serde::de::DeserializeOwned>(json: &Value) -> impl FnMut(&str) -> Result<Vec<T>, JsonPathError> + '_ {
    let mut selector = Selector::new();
    let _ = selector.value(json);
    move |path: &str| {
        selector.path(path)?.reset_value().select_as()
    }
}

/// This function compile a jsonpath everytime and it convert `serde_json's Value` to `jsonpath's RefValue` everytime and then it return a `serde_json::value::Value`.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// #[macro_use] extern crate serde_json;
///
/// let json_obj = json!({
///     "school": {
///         "friends": [
///             {"name": "친구1", "age": 20},
///             {"name": "친구2", "age": 20}
///         ]
///     },
///     "friends": [
///         {"name": "친구3", "age": 30},
///         {"name": "친구4"}
/// ]});
///
/// let json = jsonpath::select(&json_obj, "$..friends[0]").unwrap();
///
/// assert_eq!(json, vec![
///     &json!({"name": "친구3", "age": 30}),
///     &json!({"name": "친구1", "age": 20})
/// ]);
/// ```
pub fn select<'a>(json: &'a Value, path: &'a str) -> Result<Vec<&'a Value>, JsonPathError> {
    Selector::new().path(path)?.value(json).select()
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
pub fn select_as_str(json_str: &str, path: &str) -> Result<String, JsonPathError> {
    let json = serde_json::from_str(json_str).map_err(|e| JsonPathError::Serde(e.to_string()))?;
    let ret = Selector::new().path(path)?.value(&json).select()?;
    serde_json::to_string(&ret).map_err(|e| JsonPathError::Serde(e.to_string()))
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
/// let ret: Vec<Person> = jsonpath::select_as(r#"
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
/// assert_eq!(ret[0], person);
/// ```
pub fn select_as<T: serde::de::DeserializeOwned>(json_str: &str, path: &str) -> Result<Vec<T>, JsonPathError> {
    let json = serde_json::from_str(json_str).map_err(|e| JsonPathError::Serde(e.to_string()))?;
    Selector::new().path(path)?.value(&json).select_as()
}