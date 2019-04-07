use std::ops::Deref;
use std::result;

use serde_json::Value;

use super::filter::value_filter::*;
use super::parser::parser::*;
use super::ref_value::model::*;

/// Utility structure. Functions like jsonpath :: selector or jsonpath :: compile are also implemented using this structure.
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
///     name: "Doe John".to_owned(),
///     age: 44,
///     phones: vec!["+44 1234567".to_owned(), "+44 2345678".to_owned()],
/// };
///
/// assert_eq!(person, ret);
/// ```
pub struct Selector {
    pub(crate) node: Option<Node>,
    pub(crate) value: Option<RefValueWrapper>,
}

impl Selector {
    pub fn new() -> Self {
        Selector { node: None, value: None }
    }

    pub fn path(&mut self, path: &str) -> result::Result<&mut Self, String> {
        let mut parser = Parser::new(path);
        self.node = Some(parser.compile()?);
        Ok(self)
    }

    pub fn value(&mut self, ref_value: RefValue) -> result::Result<&mut Self, String> {
        self.value = Some(ref_value.into());
        Ok(self)
    }

    pub fn value_from(&mut self, serializable: &impl serde::ser::Serialize) -> result::Result<&mut Self, String> {
        let ref_value: RefValue = serializable
            .serialize(super::ref_value::ser::Serializer)
            .map_err(|e| format!("{:?}", e))?;
        self.value(ref_value)
    }

    pub fn value_from_str(&mut self, json_str: &str) -> result::Result<&mut Self, String> {
        let ref_value: RefValue = serde_json::from_str(json_str)
            .map_err(|e| format!("{:?}", e))?;
        self.value(ref_value)
    }

    fn jf(&mut self) -> result::Result<JsonValueFilter, String> {
        match &self.value {
            Some(v) => Ok(JsonValueFilter::new_from_value(v.clone())),
            _ => return Err("Value is empty".to_owned())
        }
    }

    pub fn select_to_str(&mut self) -> result::Result<String, String> {
        let mut jf = self.jf()?;

        match &mut self.node {
            Some(node) => {
                jf.visit(node.clone());
                return serde_json::to_string(jf.take_value().deref()).map_err(|e| format!("{:?}", e));
            }
            _ => return Err("Path is empty".to_owned())
        };
    }

    pub fn select_to_value(&mut self) -> result::Result<Value, String> {
        let mut jf = self.jf()?;
        match &mut self.node {
            Some(node) => {
                jf.visit(node.clone());
                Ok((&jf.take_value()).into())
            }
            _ => Err("Path is empty".to_owned())
        }
    }

    pub fn select_to<T: serde::de::DeserializeOwned>(&mut self) -> result::Result<T, String> {
        let mut jf = self.jf()?;
        match &mut self.node {
            Some(node) => {
                jf.visit(node.clone());
                T::deserialize(jf.take_value().deref()).map_err(|e| format!("{:?}", e))
            }
            _ => Err("Path is empty".to_owned())
        }
    }
}