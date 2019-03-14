extern crate indexmap;
extern crate serde_json;

use std::sync::Arc;
use std::convert::Into;

use indexmap::map::IndexMap;
use serde_json::{Number, Value};

pub type TypeRefValue = Arc<Box<RefValue>>;

impl Into<RefValueWrapper> for TypeRefValue {
    fn into(self) -> RefValueWrapper {
        RefValueWrapper::new(self.clone())
    }
}

impl Into<RefValueWrapper> for &TypeRefValue {
    fn into(self) -> RefValueWrapper {
        RefValueWrapper::new(self.clone())
    }
}

///
/// serde_json::Value 참고
///

pub trait RefIndex {
    fn index_into<'v>(&self, v: &'v RefValue) -> Option<&'v TypeRefValue>;
    fn index_into_mut<'v>(&self, v: &'v mut RefValue) -> Option<&'v mut TypeRefValue>;
    fn index_or_insert<'v>(&self, v: &'v mut RefValue) -> &'v mut TypeRefValue;
}

impl RefIndex for usize {
    fn index_into<'v>(&self, v: &'v RefValue) -> Option<&'v TypeRefValue> {
        match *v {
            RefValue::Array(ref vec) => vec.get(*self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut RefValue) -> Option<&'v mut TypeRefValue> {
        match *v {
            RefValue::Array(ref mut vec) => vec.get_mut(*self),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut RefValue) -> &'v mut TypeRefValue {
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
    fn index_into<'v>(&self, v: &'v RefValue) -> Option<&'v TypeRefValue> {
        match *v {
            RefValue::Object(ref map) => map.get(self),
            _ => None,
        }
    }
    fn index_into_mut<'v>(&self, v: &'v mut RefValue) -> Option<&'v mut TypeRefValue> {
        match *v {
            RefValue::Object(ref mut map) => map.get_mut(self),
            _ => None,
        }
    }
    fn index_or_insert<'v>(&self, v: &'v mut RefValue) -> &'v mut TypeRefValue {
        if let RefValue::Null = *v {
            *v = RefValue::Object(IndexMap::new());
        }
        match *v {
            RefValue::Object(ref mut map) => {
                map.entry(self.to_owned()).or_insert(RefValueWrapper::wrap(RefValue::Null))
            }
            _ => panic!("cannot access key {:?} in JSON {:?}", self, v),
        }
    }
}

impl RefIndex for String {
    fn index_into<'v>(&self, v: &'v RefValue) -> Option<&'v TypeRefValue> {
        self[..].index_into(v)
    }
    fn index_into_mut<'v>(&self, v: &'v mut RefValue) -> Option<&'v mut TypeRefValue> {
        self[..].index_into_mut(v)
    }
    fn index_or_insert<'v>(&self, v: &'v mut RefValue) -> &'v mut TypeRefValue {
        self[..].index_or_insert(v)
    }
}

#[derive(Debug)]
pub struct RefValueWrapper {
    data: TypeRefValue
}

impl RefValueWrapper {
    pub fn new(ref_value: TypeRefValue) -> Self {
        RefValueWrapper { data: ref_value }
    }

    pub fn wrap(ref_val: RefValue) -> TypeRefValue {
        Arc::new(Box::new(ref_val))
    }

    pub fn into_value(&self) -> Value {
        ValueConverter::new(&self.data)
    }

    pub fn clone(&self) -> Self {
        RefValueWrapper { data: self.data.clone() }
    }

    pub fn clone_data(&self) -> TypeRefValue {
        self.data.clone()
    }

    pub fn get<I: RefIndex>(&self, index: I) -> Option<RefValueWrapper> {
        index.index_into(&**self.data).map(|v| Self::new(v.clone()))
    }

    pub fn is_object(&self) -> bool {
        (**self.data).is_object()
    }

    pub fn as_object(&self) -> Option<&IndexMap<String, TypeRefValue>> {
        (**self.data).as_object()
    }

    pub fn is_array(&self) -> bool {
        (**self.data).is_array()
    }

    pub fn as_array(&self) -> Option<&Vec<TypeRefValue>> {
        (**self.data).as_array()
    }

    pub fn is_string(&self) -> bool {
        (**self.data).is_string()
    }

    pub fn as_str(&self) -> Option<&str> {
        (**self.data).as_str()
    }

    pub fn is_number(&self) -> bool {
        (**self.data).is_number()
    }

    pub fn as_number(&self) -> Option<Number> {
        (**self.data).as_number()
    }

    pub fn is_boolean(&self) -> bool {
        (**self.data).is_boolean()
    }

    pub fn as_bool(&self) -> Option<bool> {
        (**self.data).as_bool()
    }

    pub fn is_null(&self) -> bool {
        (**self.data).is_null()
    }

    pub fn as_null(&self) -> Option<()> {
        (**self.data).as_null()
    }

    pub fn get_data_ref(&self) -> &RefValue {
        &(**self.data)
    }
}

impl Into<RefValueWrapper> for &Value {
    fn into(self) -> RefValueWrapper {
        let ref_val = RefValueConverter::new(self);
        RefValueWrapper::new(ref_val)
    }
}

#[derive(Debug)]
pub enum RefValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<TypeRefValue>),
    Object(IndexMap<String, TypeRefValue>),
}

impl RefValue {
    pub fn get<I: RefIndex>(&self, index: I) -> Option<&TypeRefValue> {
        index.index_into(self)
    }

    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    pub fn as_object(&self) -> Option<&IndexMap<String, TypeRefValue>> {
        match *self {
            RefValue::Object(ref map) => Some(map),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn as_array(&self) -> Option<&Vec<TypeRefValue>> {
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
        let wrap = RefValueWrapper::wrap(self);
        RefValueWrapper::new(wrap)
    }
}

struct RefValueConverter;

impl RefValueConverter {
    fn new(value: &Value) -> TypeRefValue {
        RefValueConverter {}.visit_value(value)
    }

    fn visit_value(&self, value: &Value) -> TypeRefValue {
        match value {
            Value::Null => self.visit_null(),
            Value::Bool(v) => self.visit_bool(v),
            Value::Number(v) => self.visit_number(v),
            Value::String(v) => self.visit_string(v),
            Value::Array(v) => self.visit_array(v),
            Value::Object(v) => self.visit_object(v),
        }
    }
    fn visit_null(&self) -> TypeRefValue {
        RefValueWrapper::wrap(RefValue::Null)
    }
    fn visit_bool(&self, value: &bool) -> TypeRefValue {
        RefValueWrapper::wrap(RefValue::Bool(*value))
    }
    fn visit_number(&self, value: &serde_json::Number) -> TypeRefValue {
        RefValueWrapper::wrap(RefValue::Number(value.clone()))
    }
    fn visit_string(&self, value: &String) -> TypeRefValue {
        RefValueWrapper::wrap(RefValue::String(value.to_string()))
    }
    fn visit_array(&self, value: &Vec<Value>) -> TypeRefValue {
        let mut values = Vec::new();
        for v in value {
            values.push(self.visit_value(v));
        }
        RefValueWrapper::wrap(RefValue::Array(values))
    }
    fn visit_object(&self, value: &serde_json::Map<String, Value>) -> TypeRefValue {
        let mut map = IndexMap::new();
        let keys: Vec<String> = value.keys().into_iter().map(|k| k.to_string()).collect();
        for k in keys {
            let value = self.visit_value(match value.get(&k) {
                Some(v) => v,
                _ => &Value::Null
            });
            map.insert(k, value);
        }
        RefValueWrapper::wrap(RefValue::Object(map))
    }
}

struct ValueConverter;

impl ValueConverter {
    fn new(value: &TypeRefValue) -> Value {
        ValueConverter {}.visit_value(value)
    }

    fn visit_value(&self, value: &TypeRefValue) -> Value {
        match &***value {
            RefValue::Null => self.visit_null(),
            RefValue::Bool(v) => self.visit_bool(v),
            RefValue::Number(v) => self.visit_number(v),
            RefValue::String(v) => self.visit_string(v),
            RefValue::Array(v) => self.visit_array(v),
            RefValue::Object(v) => self.visit_object(v),
        }
    }
    fn visit_null(&self) -> Value {
        Value::Null
    }
    fn visit_bool(&self, value: &bool) -> Value {
        Value::Bool(*value)
    }
    fn visit_number(&self, value: &serde_json::Number) -> Value {
        Value::Number(value.clone())
    }
    fn visit_string(&self, value: &String) -> Value {
        Value::String(value.clone())
    }
    fn visit_array(&self, value: &Vec<TypeRefValue>) -> Value {
        let mut values = Vec::new();
        for v in value {
            values.push(self.visit_value(v));
        }
        Value::Array(values)
    }
    fn visit_object(&self, map: &IndexMap<String, TypeRefValue>) -> Value {
        let mut ret = serde_json::Map::new();
        let keys: Vec<String> = map.keys().into_iter().map(|k: &String| k.to_string()).collect();
        let tmp_null = &RefValueWrapper::wrap(RefValue::Null);
        for k in keys {
            let value = self.visit_value(match map.get(&k) {
                Some(e) => e,
                _ => tmp_null
            });
            ret.insert(k, value);
        }
        Value::Object(ret)
    }
}