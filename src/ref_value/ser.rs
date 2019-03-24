use std::result::Result;

use indexmap::IndexMap;
use serde::{self, Serialize};
use serde::ser::Impossible;

use ref_value::model::{RefValue, RefValueWrapper};

use super::serde_error::SerdeError;

///
/// see `serde_json/value/ser.rs`
///
impl Serialize for RefValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
    {
        match *self {
            RefValue::Null => serializer.serialize_unit(),
            RefValue::Bool(b) => serializer.serialize_bool(b),
            RefValue::Number(ref n) => n.serialize(serializer),
            RefValue::String(ref s) => serializer.serialize_str(s),
            RefValue::Array(ref v) => {
                use std::ops::Deref;
                let v: Vec<&RefValue> = v.iter().map(|v| v.deref()).collect();
                v.serialize(serializer)
            }
            RefValue::Object(ref m) => {
                use serde::ser::SerializeMap;
                use std::ops::Deref;
                let mut map = try!(serializer.serialize_map(Some(m.len())));
                for (k, v) in m {
                    try!(map.serialize_key(k));
                    try!(map.serialize_value(v.deref()));
                }
                map.end()
            }
        }
    }
}

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = RefValue;
    type Error = SerdeError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<RefValue, Self::Error> {
        Ok(RefValue::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<RefValue, Self::Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<RefValue, Self::Error> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<RefValue, Self::Error> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<RefValue, Self::Error> {
        Ok(RefValue::Number(value.into()))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<RefValue, Self::Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<RefValue, Self::Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<RefValue, Self::Error> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<RefValue, Self::Error> {
        Ok(RefValue::Number(value.into()))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<RefValue, Self::Error> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<RefValue, Self::Error> {
        Ok(serde_json::Number::from_f64(value).map_or(RefValue::Null, RefValue::Number))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<RefValue, Self::Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<RefValue, Self::Error> {
        Ok(RefValue::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<RefValue, Self::Error> {
        let vec = value.iter().map(|&b| RefValue::Number(b.into()).into()).collect();
        Ok(RefValue::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<RefValue, Self::Error> {
        Ok(RefValue::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<RefValue, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<RefValue, Self::Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<RefValue, Self::Error>
        where
            T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<RefValue, Self::Error>
        where
            T: Serialize,
    {
        let mut values: IndexMap<String, RefValueWrapper> = IndexMap::new();
        values.insert(String::from(variant), {
            value.serialize(Serializer)?.into()
        });
        Ok(RefValue::Object(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<RefValue, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<RefValue, Self::Error>
        where
            T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap::Map {
            map: IndexMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        match name {
            _ => self.serialize_map(Some(len)),
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: IndexMap::new(),
        })
    }
}

pub struct SerializeVec {
    vec: Vec<RefValueWrapper>,
}

pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<RefValueWrapper>,
}

pub enum SerializeMap {
    Map {
        map: IndexMap<String, RefValueWrapper>,
        next_key: Option<String>,
    },
}

pub struct SerializeStructVariant {
    name: String,
    map: IndexMap<String, RefValueWrapper>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = RefValue;
    type Error = SerdeError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize,
    {
        self.vec.push({
            value.serialize(Serializer)?.into()
        });
        Ok(())
    }

    fn end(self) -> Result<RefValue, Self::Error> {
        Ok(RefValue::Array(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = RefValue;
    type Error = SerdeError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<RefValue, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = RefValue;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<RefValue, Self::Error> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = RefValue;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize,
    {
        self.vec.push({
            let a: RefValue = value.serialize(Serializer)?;
            a.into()
        });
        Ok(())
    }

    fn end(self) -> Result<RefValue, Self::Error> {
        let mut object: IndexMap<String, RefValueWrapper> = IndexMap::new();

        object.insert(self.name, RefValue::Array(self.vec).into());

        Ok(RefValue::Object(object))
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = RefValue;
    type Error = SerdeError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
        where
            T: Serialize,
    {
        match *self {
            SerializeMap::Map {
                ref mut next_key, ..
            } => {
                *next_key = Some(key.serialize(MapKeySerializer)?);
                Ok(())
            }
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize,
    {
        match *self {
            SerializeMap::Map {
                ref mut map,
                ref mut next_key,
            } => {
                let key = next_key.take();
                // Panic because this indicates a bug in the program rather than an
                // expected failure.
                let key = key.expect("serialize_value called before serialize_key");
                map.insert(key, {
                    let a: RefValue = value.serialize(Serializer)?;
                    a.into()
                });
                Ok(())
            }
        }
    }

    fn end(self) -> Result<RefValue, Self::Error> {
        match self {
            SerializeMap::Map { map, .. } => Ok(RefValue::Object(map)),
        }
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> SerdeError {
    SerdeError::from_str("key must be string")
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = SerdeError;

    type SerializeSeq = Impossible<String, Self::Error>;
    type SerializeTuple = Impossible<String, Self::Error>;
    type SerializeTupleStruct = Impossible<String, Self::Error>;
    type SerializeTupleVariant = Impossible<String, Self::Error>;
    type SerializeMap = Impossible<String, Self::Error>;
    type SerializeStruct = Impossible<String, Self::Error>;
    type SerializeStructVariant = Impossible<String, Self::Error>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(key_must_be_a_string())
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = RefValue;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize,
    {
        match *self {
            SerializeMap::Map { .. } => {
                serde::ser::SerializeMap::serialize_key(self, key)?;
                serde::ser::SerializeMap::serialize_value(self, value)
            }
        }
    }

    fn end(self) -> Result<RefValue, Self::Error> {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::end(self),
        }
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = RefValue;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: Serialize,
    {
        self.map.insert(String::from(key), {
            let a: RefValue = value.serialize(Serializer)?;
            a.into()
        });
        Ok(())
    }

    fn end(self) -> Result<RefValue, Self::Error> {
        let mut object: IndexMap<String, RefValueWrapper> = IndexMap::new();

        object.insert(self.name, RefValue::Object(self.map).into());

        Ok(RefValue::Object(object))
    }
}
