use std::ops::Deref;

use indexmap::IndexMap;
use serde_json::Value;

use ref_value::model::{RefValue, RefValueWrapper};

pub struct RefValueConverter;

impl RefValueConverter {
    pub fn new(value: &Value) -> RefValueWrapper {
        RefValueConverter {}.visit_value(value)
    }

    fn visit_value(&self, value: &Value) -> RefValueWrapper {
        match value {
            Value::Null => self.visit_null(),
            Value::Bool(v) => self.visit_bool(v),
            Value::Number(v) => self.visit_number(v),
            Value::String(v) => self.visit_string(v),
            Value::Array(v) => self.visit_array(v),
            Value::Object(v) => self.visit_object(v),
        }
    }

    fn visit_null(&self) -> RefValueWrapper {
        RefValue::Null.into()
    }

    fn visit_bool(&self, value: &bool) -> RefValueWrapper {
        RefValue::Bool(*value).into()
    }

    fn visit_number(&self, value: &serde_json::Number) -> RefValueWrapper {
        RefValue::Number(value.clone()).into()
    }

    fn visit_string(&self, value: &String) -> RefValueWrapper {
        RefValue::String(value.to_string()).into()
    }

    fn visit_array(&self, value: &Vec<Value>) -> RefValueWrapper {
        let mut values = Vec::new();
        for v in value {
            values.push(self.visit_value(v));
        }
        RefValue::Array(values).into()
    }

    fn visit_object(&self, value: &serde_json::Map<String, Value>) -> RefValueWrapper {
        let mut map = IndexMap::new();
        for (key, _) in value {
            let value = self.visit_value(match value.get(key) {
                Some(v) => v,
                _ => &Value::Null
            });
            map.insert(key.clone(), value);
        }
        RefValue::Object(map).into()
    }
}

pub struct ValueConverter;

impl ValueConverter {
    pub fn new(value: &RefValueWrapper) -> Value {
        ValueConverter {}.visit_value(value)
    }

    fn visit_value(&self, value: &RefValueWrapper) -> Value {
        match value.deref() {
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
    fn visit_array(&self, value: &Vec<RefValueWrapper>) -> Value {
        let mut values = Vec::new();
        for v in value {
            values.push(self.visit_value(v));
        }
        Value::Array(values)
    }
    fn visit_object(&self, map: &IndexMap<String, RefValueWrapper>) -> Value {
        let mut ret = serde_json::Map::new();
        let tmp_null = &RefValue::Null.into();
        for (k, _) in map {
            let value = self.visit_value(match map.get(k) {
                Some(e) => e,
                _ => tmp_null
            });
            ret.insert(k.to_string(), value);
        }
        Value::Object(ret)
    }
}