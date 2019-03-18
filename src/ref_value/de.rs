use std::error::Error;
use std::fmt;
use std::result::Result;

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde_json::Value;

use super::model::*;

impl<'de> Deserialize<'de> for RefValue {
    fn deserialize<D>(deserializer: D) -> Result<RefValue, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RefValueVisitor {})
    }
}

struct RefValueVisitor {}

impl<'de> Visitor<'de> for RefValueVisitor {
    type Value = RefValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid JSON value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: Error, {
        Ok(RefValue::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(RefValue::Number(v.into()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(RefValue::Number(v.into()))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        let n: Value = v.into();
        if let Value::Number(n) = n {
            Ok(RefValue::Number(n))
        } else {
            unreachable!()
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_string(String::from(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(RefValue::String(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(RefValue::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where
        D: Deserializer<'de>, {
        Deserialize::deserialize(deserializer)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(RefValue::Null)
    }

    fn visit_seq<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>, {
        let mut vec = Vec::new();

        while let Some(elem) = visitor.next_element()? {
            let e: RefValue = elem;
            let v: RefValueWrapper = e.into();
            vec.push(v);
        }

        Ok(RefValue::Array(vec))
    }

    fn visit_map<A>(self, mut visitor: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>, {
        let mut values = IndexMap::new();
        match visitor.next_key() {
            Ok(Some(first_key)) => {
                let next: RefValue = visitor.next_value()?;
                values.insert(first_key, next.into());
                while let Some((k, v)) = visitor.next_entry()? {
                    let value: RefValue = v;
                    values.insert(k, value.into());
                }
                Ok(RefValue::Object(values))
            }
            _ => Ok(RefValue::Object(IndexMap::new())),
        }
    }
}