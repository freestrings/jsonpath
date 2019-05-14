use std::ops::Deref;
use std::result;

use serde_json::Value;

use super::filter::value_filter::*;
use super::parser::parser::*;
use super::ref_value::model::*;

/// Utility. Functions like jsonpath::selector or jsonpath::compile are also implemented using this structure.
///
/// ```rust
/// extern crate jsonpath_lib as jsonpath;
/// extern crate serde;
/// extern crate serde_json;
///
/// use serde::{Deserialize, Serialize};
/// use serde_json::Value;
///
/// use jsonpath::Selector;
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Person {
///     name: String,
///     age: u8,
///     phone: String,
/// }
///
/// fn input_str() -> &'static str {
///     r#"[
///         {
///             "name": "이름1",
///             "age": 40,
///             "phone": "+33 12341234"
///         },
///         {
///             "name": "이름2",
///             "age": 42,
///             "phone": "++44 12341234"
///         }
///     ]"#
/// }
///
/// fn input_json() -> Value {
///     serde_json::from_str(input_str()).unwrap()
/// }
///
/// fn input_person() -> Vec<Person> {
///     serde_json::from_str(input_str()).unwrap()
/// }
///
///
/// let mut selector = Selector::new();
///
/// let result = selector
///     .path("$..[?(@.age > 40)]").unwrap()
///     .value_from_str(input_str()).unwrap()
///     .select_to_value().unwrap();
/// assert_eq!(input_json()[1], result[0]);
///
/// let result = selector.select_to_str().unwrap();
/// assert_eq!(serde_json::to_string(&vec![&input_json()[1].clone()]).unwrap(), result);
///
/// let result = selector.select_to::<Vec<Person>>().unwrap();
/// assert_eq!(input_person()[1], result[0]);
///
/// let _ = selector.path("$..[?(@.age == 40)]");
///
/// let result = selector.select_to_value().unwrap();
/// assert_eq!(input_json()[0], result[0]);
///
/// let result = selector.select_to_str().unwrap();
/// assert_eq!(serde_json::to_string(&vec![&input_json()[0].clone()]).unwrap(), result);
///
/// let result = selector.select_to::<Vec<Person>>().unwrap();
/// assert_eq!(input_person()[0], result[0]);
/// ```
#[derive(Debug)]
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

    pub fn value(&mut self, value: &Value) -> result::Result<&mut Self, String> {
        self.value = Some(value.into());
        Ok(self)
    }

    pub fn value_from(&mut self, serializable: &impl serde::ser::Serialize) -> result::Result<&mut Self, String> {
        let ref_value: RefValue = serializable
            .serialize(super::ref_value::ser::Serializer)
            .map_err(|e| format!("{:?}", e))?;
        self.value = Some(ref_value.into());
        Ok(self)
    }

    pub fn value_from_str(&mut self, json_str: &str) -> result::Result<&mut Self, String> {
        let value = serde_json::from_str(json_str)
            .map_err(|e| format!("{:?}", e))?;
        self.value(&value)
    }

    fn jf(&self) -> result::Result<JsonValueFilter, String> {
        match &self.value {
            Some(v) => Ok(JsonValueFilter::new_from_value(v.clone())),
            _ => return Err("Empty value".to_owned())
        }
    }

    fn select(&self) -> result::Result<RefValueWrapper, String> {
        let mut jf = self.jf()?;

        match &self.node {
            Some(node) => {
                jf.visit(node.clone());
                Ok(jf.take_value())
            }
            _ => Err("Empty path".to_owned())
        }
    }

    pub fn select_to_str(&self) -> result::Result<String, String> {
        serde_json::to_string(self.select()?.deref()).map_err(|e| format!("{:?}", e))
    }

    pub fn select_to_value(&self) -> result::Result<Value, String> {
        Ok((&self.select()?).into())
    }

    pub fn select_to<T: serde::de::DeserializeOwned>(&self) -> result::Result<T, String> {
        let mut jf = self.jf()?;
        match &self.node {
            Some(node) => {
                jf.visit(node.clone());
                T::deserialize(jf.take_value().deref()).map_err(|e| format!("{:?}", e))
            }
            _ => Err("Path is empty".to_owned())
        }
    }
}