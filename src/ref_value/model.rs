use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

use indexmap::map::IndexMap;
use serde::ser::Serialize;
use serde_json::{Number, Value};

type TypeRefValue = Arc<Box<RefValue>>;

#[derive(Debug, PartialEq)]
pub struct RefValueWrapper {
    data: TypeRefValue
}

impl RefValueWrapper {
    pub fn try_unwrap(self) -> RefValue {
        match Arc::try_unwrap(self.data) {
            Ok(ref_value) => *ref_value,
            Err(e) => panic!("{:?}", e)
        }
    }
}

impl Eq for RefValueWrapper {}

impl Deref for RefValueWrapper {
    type Target = RefValue;

    fn deref(&self) -> &Self::Target {
        &(**self.data)
    }
}

impl Hash for RefValueWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl Clone for RefValueWrapper {
    fn clone(&self) -> Self {
        RefValueWrapper {
            data: self.data.clone()
        }
    }
}

///
/// serde_json::Value 참고
///
pub trait RefIndex {
    fn index_into<'v>(&self, v: &'v RefValue) -> Option<&'v RefValueWrapper>;
    fn index_into_mut<'v>(&self, v: &'v mut RefValue) -> Option<&'v mut RefValueWrapper>;
    fn index_or_insert<'v>(&self, v: &'v mut RefValue) -> &'v mut RefValueWrapper;
}

impl RefIndex for usize {
    fn index_into<'v>(&self, v: &'v RefValue) -> Option<&'v RefValueWrapper> {
        match *v {
            RefValue::Array(ref vec) => vec.get(*self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut RefValue) -> Option<&'v mut RefValueWrapper> {
        match *v {
            RefValue::Array(ref mut vec) => vec.get_mut(*self),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut RefValue) -> &'v mut RefValueWrapper {
        match *v {
            RefValue::Array(ref mut vec) => {
                let len = vec.len();
                vec.get_mut(*self).unwrap_or_else(|| {
                    panic!(
                        "cannot access index {} of JSON array of length {}",
                        self, len
                    )
                })
            }
            _ => panic!("cannot access index {} of JSON {:?}", self, v),
        }
    }
}

impl RefIndex for str {
    fn index_into<'v>(&self, v: &'v RefValue) -> Option<&'v RefValueWrapper> {
        match *v {
            RefValue::Object(ref map) => map.get(self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut RefValue) -> Option<&'v mut RefValueWrapper> {
        match *v {
            RefValue::Object(ref mut map) => map.get_mut(self),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut RefValue) -> &'v mut RefValueWrapper {
        if let RefValue::Null = *v {
            *v = RefValue::Object(IndexMap::new());
        }
        match *v {
            RefValue::Object(ref mut map) => {
                map.entry(self.to_owned()).or_insert(RefValue::Null.into())
            }
            _ => panic!("cannot access key {:?} in JSON {:?}", self, v),
        }
    }
}

impl RefIndex for String {
    fn index_into<'v>(&self, v: &'v RefValue) -> Option<&'v RefValueWrapper> {
        self[..].index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut RefValue) -> Option<&'v mut RefValueWrapper> {
        self[..].index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut RefValue) -> &'v mut RefValueWrapper {
        self[..].index_or_insert(v)
    }
}

#[derive(Debug, PartialEq)]
pub enum RefValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<RefValueWrapper>),
    Object(IndexMap<String, RefValueWrapper>),
}

static REF_VALUE_NULL: &'static str = "$jsonpath::ref_value::model::RefValue::Null";

impl Hash for RefValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RefValue::Null => {
                REF_VALUE_NULL.hash(state)
            }
            RefValue::Bool(b) => {
                b.hash(state)
            }
            RefValue::Number(n) => {
                if n.is_f64() {
                    n.as_f64().unwrap().to_string().hash(state)
                } else if n.is_i64() {
                    n.as_i64().unwrap().hash(state);
                } else {
                    n.as_u64().unwrap().hash(state);
                }
            }
            RefValue::String(s) => {
                s.hash(state)
            }
            RefValue::Object(map) => {
                for (_, v) in map {
                    v.hash(state);
                }
            }
            RefValue::Array(v) => {
                for i in v {
                    i.hash(state);
                }
            }
        }
    }
}

impl Eq for RefValue {}

impl RefValue {
    pub fn get<I: RefIndex>(&self, index: I) -> Option<&RefValueWrapper> {
        index.index_into(self)
    }

    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    pub fn as_object(&self) -> Option<&IndexMap<String, RefValueWrapper>> {
        match *self {
            RefValue::Object(ref map) => Some(map),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn as_array(&self) -> Option<&Vec<RefValueWrapper>> {
        match *self {
            RefValue::Array(ref array) => Some(&*array),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            RefValue::String(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            RefValue::Number(_) => true,
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<Number> {
        match *self {
            RefValue::Number(ref n) => Some(n.clone()),
            _ => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            RefValue::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    pub fn as_null(&self) -> Option<()> {
        match *self {
            RefValue::Null => Some(()),
            _ => None,
        }
    }
}

impl Into<RefValueWrapper> for RefValue {
    fn into(self) -> RefValueWrapper {
        RefValueWrapper {
            data: Arc::new(Box::new(self))
        }
    }
}

impl Into<RefValueWrapper> for &Value {
    fn into(self) -> RefValueWrapper {
        match self.serialize(super::ser::Serializer) {
            Ok(v) => v.into(),
            Err(e) => panic!("Error Value into RefValue: {:?}", e)
        }
    }
}

impl Into<Value> for RefValueWrapper {
    fn into(self) -> Value {
        match serde_json::to_value(self.deref()) {
            Ok(v) => v,
            Err(e) => panic!("Error RefValueWrapper into Value: {:?}", e)
        }
    }
}

impl Into<Value> for &RefValueWrapper {
    fn into(self) -> Value {
        match serde_json::to_value(self.deref()) {
            Ok(v) => v,
            Err(e) => panic!("Error RefValueWrapper into Value: {:?}", e)
        }
    }
}