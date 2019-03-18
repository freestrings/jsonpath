use std::result::Result;

use serde::{self, Serialize};

use ref_value::model::{RefValue, RefValueWrapper};

impl Serialize for RefValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: serde::Serializer {
        match *self {
            RefValue::Null => serializer.serialize_unit(),
            RefValue::Bool(b) => serializer.serialize_bool(b),
            RefValue::Number(ref n) => n.serialize(serializer),
            RefValue::String(ref s) => serializer.serialize_str(s),
            RefValue::Array(ref v) => {
                use std::ops::Deref;
                let v: Vec<&RefValue> = v.iter().map(|v| v.deref()).collect();
                v.serialize(serializer)
            },
            RefValue::Object(ref m) => {
                use serde::ser::SerializeMap;
                use std::ops::Deref;
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_key(k)?;
                    map.serialize_value(v.deref())?;
                }
                map.end()
            }
        }
    }
}

impl Serialize for RefValueWrapper {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: serde::Serializer {
        use std::ops::Deref;

        match *self.deref() {
            RefValue::Null => serializer.serialize_unit(),
            RefValue::Bool(b) => serializer.serialize_bool(b),
            RefValue::Number(ref n) => n.serialize(serializer),
            RefValue::String(ref s) => serializer.serialize_str(s),
            RefValue::Array(ref v) => {
                use std::ops::Deref;
                let v: Vec<&RefValue> = v.iter().map(|v| v.deref()).collect();
                v.serialize(serializer)
            },
            RefValue::Object(ref m) => {
                use serde::ser::SerializeMap;
                use std::ops::Deref;
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_key(k)?;
                    map.serialize_value(v.deref())?;
                }
                map.end()
            }
        }
    }
}