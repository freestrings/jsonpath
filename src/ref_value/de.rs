use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;
use std::result::Result;
use std::vec;

use indexmap::IndexMap;
use serde::{self, Deserialize, Deserializer};
use serde::de::{DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde_json::Value;

use super::model::*;
use super::serde_error::SerdeError;

///
/// see `serde_json/value/de.rs`
///

macro_rules! deserialize_prim_number {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, SerdeError>
        where
            V: Visitor<'de>,
        {
            match self {
                RefValue::Number(ref n) => {
                    match n.deserialize_any(visitor) {
                        Ok(v) => Ok(v),
                        Err(e) => Err(SerdeError::new(format!("{:?}", e)))
                    }
                }
                _ => Err(SerdeError::from_str("invalid type")),
            }
        }
    }
}

impl<'de> Deserialize<'de> for RefValue {
    fn deserialize<D>(deserializer: D) -> Result<RefValue, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct RefValueVisitor {}

        impl<'de> Visitor<'de> for RefValueVisitor {
            type Value = RefValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                Ok(RefValue::Bool(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                Ok(RefValue::Number(v.into()))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                Ok(RefValue::Number(v.into()))
            }

            #[inline]
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

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                self.visit_string(String::from(v))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                Ok(RefValue::String(v))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                Ok(RefValue::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where
                D: Deserializer<'de>, {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                Ok(RefValue::Null)
            }

            #[inline]
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

        deserializer.deserialize_any(RefValueVisitor {})
    }
}

fn visit_array<'de, V>(array: Vec<RefValueWrapper>, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
{
    let mut deserializer = SeqDeserializer::new(array);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(SerdeError::from_str("fewer elements in array"))
    }
}

fn visit_object<'de, V>(object: IndexMap<String, RefValueWrapper>, visitor: V) -> Result<V::Value, SerdeError>
    where
        V: Visitor<'de>,
{
    let mut deserializer = MapDeserializer::new(object);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(SerdeError::from_str("fewer elements in map"))
    }
}

fn to_vec(vec: &Vec<RefValueWrapper>) -> Vec<RefValueWrapper> {
    vec.iter().map(|v| v.clone()).collect()
}

fn to_map(object: &IndexMap<String, RefValueWrapper>) -> IndexMap<String, RefValueWrapper> {
    let mut map = IndexMap::new();
    for (k, v) in object {
        map.insert(k.to_string(), v.clone());
    }
    map
}

impl<'de> serde::Deserializer<'de> for RefValue {
    type Error = SerdeError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error> where
        V: Visitor<'de> {
        match self {
            RefValue::Null => visitor.visit_unit(),
            RefValue::Bool(v) => visitor.visit_bool(v),
            RefValue::Number(n) => {
                n.deserialize_any(visitor).map_err(|e| SerdeError::new(format!("{:?}", e)))
            }
            RefValue::String(v) => visitor.visit_string(v),
            RefValue::Array(array) => visit_array(array, visitor),
            RefValue::Object(object) => visit_object(object, visitor)
        }
    }

    deserialize_prim_number!(deserialize_i8);
    deserialize_prim_number!(deserialize_i16);
    deserialize_prim_number!(deserialize_i32);
    deserialize_prim_number!(deserialize_i64);
    deserialize_prim_number!(deserialize_u8);
    deserialize_prim_number!(deserialize_u16);
    deserialize_prim_number!(deserialize_u32);
    deserialize_prim_number!(deserialize_u64);
    deserialize_prim_number!(deserialize_f32);
    deserialize_prim_number!(deserialize_f64);

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        let (variant, value) = match self {
            RefValue::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(SerdeError::from_str("map with a single key"));
                    }
                };
                if iter.next().is_some() {
                    return Err(SerdeError::from_str("map with a single key"));
                }
                (variant, Some(value))
            }
            RefValue::String(variant) => (variant, None),
            _ => {
                return Err(SerdeError::from_str("string or map"));
            }
        };

        visitor.visit_enum(EnumDeserializer {
            variant: variant,
            value: value,
        })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Bool(v) => visitor.visit_bool(v),
            _ => Err(SerdeError::from_str("invalid type: bool")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::String(v) => visitor.visit_string(v),
            _ => Err(SerdeError::from_str("invalid type: string")),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::String(v) => visitor.visit_string(v),
            RefValue::Array(v) => visit_array(v, visitor),
            _ => Err(SerdeError::from_str("invalid type: string or array")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Null => visitor.visit_unit(),
            _ => Err(SerdeError::from_str("invalid type: null")),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Array(v) => visit_array(v, visitor),
            _ => Err(SerdeError::from_str("invalid type: array")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Object(v) => visit_object(v, visitor),
            _ => Err(SerdeError::from_str("invalid type: object"))
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Array(v) => visit_array(v, visitor),
            RefValue::Object(v) => visit_object(v, visitor),
            _ => Err(SerdeError::from_str("invalid type: array, object"))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

impl<'de> serde::Deserializer<'de> for &RefValue {
    type Error = SerdeError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error> where
        V: Visitor<'de> {
        match self {
            RefValue::Null => visitor.visit_unit(),
            RefValue::Bool(v) => visitor.visit_bool(*v),
            RefValue::Number(n) => {
                n.deserialize_any(visitor).map_err(|e| SerdeError::new(format!("{:?}", e)))
            }
            RefValue::String(v) => visitor.visit_string(v.to_string()),
            RefValue::Array(array) => visit_array(to_vec(array), visitor),
            RefValue::Object(object) => {
                visit_object(to_map(object), visitor)
            }
        }
    }

    deserialize_prim_number!(deserialize_i8);
    deserialize_prim_number!(deserialize_i16);
    deserialize_prim_number!(deserialize_i32);
    deserialize_prim_number!(deserialize_i64);
    deserialize_prim_number!(deserialize_u8);
    deserialize_prim_number!(deserialize_u16);
    deserialize_prim_number!(deserialize_u32);
    deserialize_prim_number!(deserialize_u64);
    deserialize_prim_number!(deserialize_f32);
    deserialize_prim_number!(deserialize_f64);

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        let (variant, value) = match self {
            RefValue::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(SerdeError::from_str("map with a single key"));
                    }
                };
                if iter.next().is_some() {
                    return Err(SerdeError::from_str("map with a single key"));
                }
                (variant, Some(value))
            }
            RefValue::String(variant) => (variant, None),
            _ => {
                return Err(SerdeError::from_str("string or map"));
            }
        };

        visitor.visit_enum(EnumDeserializer {
            variant: variant.to_string(),
            value: match value {
                Some(v) => Some(v.clone()),
                _ => None
            },
        })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Bool(v) => visitor.visit_bool(*v),
            _ => Err(SerdeError::from_str("invalid type: bool")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::String(v) => visitor.visit_string(v.to_string()),
            _ => Err(SerdeError::from_str("invalid type: string")),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::String(v) => visitor.visit_string(v.to_string()),
            RefValue::Array(vec) => visit_array(to_vec(vec), visitor),
            _ => Err(SerdeError::from_str("invalid type: string or array")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Null => visitor.visit_unit(),
            _ => Err(SerdeError::from_str("invalid type: null")),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Array(vec) => visit_array(to_vec(vec), visitor),
            _ => Err(SerdeError::from_str("invalid type: array")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Object(object) => {
                visit_object(to_map(object), visitor)
            }
            _ => Err(SerdeError::from_str("invalid type: object"))
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self {
            RefValue::Array(vec) => visit_array(to_vec(vec), visitor),
            RefValue::Object(object) => {
                visit_object(to_map(object), visitor)
            }
            _ => Err(SerdeError::from_str("invalid type: array, object"))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}


struct SeqDeserializer {
    iter: vec::IntoIter<RefValueWrapper>,
}

impl SeqDeserializer {
    fn new(vec: Vec<RefValueWrapper>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = SerdeError;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = visitor.visit_seq(&mut self)?;
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(SerdeError::from_str("fewer elements in array"))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = SerdeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value.deref()).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer {
    iter: <IndexMap<String, RefValueWrapper> as IntoIterator>::IntoIter,
    value: Option<RefValueWrapper>,
}

impl MapDeserializer {
    fn new(map: IndexMap<String, RefValueWrapper>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = SerdeError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::Owned(key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value.deref()),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapDeserializer {
    type Error = SerdeError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct MapKeyDeserializer<'de> {
    key: Cow<'de, str>,
}

macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, SerdeError>
        where
            V: Visitor<'de>,
        {
            match (self.key.parse(), self.key) {
                (Ok(integer), _) => visitor.$visit(integer),
                (Err(_), Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
                (Err(_), Cow::Owned(s)) => visitor.visit_string(s),
            }
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapKeyDeserializer<'de> {
    type Error = SerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        BorrowedCowStrDeserializer::new(self.key).deserialize_any(visitor)
    }

    deserialize_integer_key!(deserialize_i8 => visit_i8);
    deserialize_integer_key!(deserialize_i16 => visit_i16);
    deserialize_integer_key!(deserialize_i32 => visit_i32);
    deserialize_integer_key!(deserialize_i64 => visit_i64);
    deserialize_integer_key!(deserialize_u8 => visit_u8);
    deserialize_integer_key!(deserialize_u16 => visit_u16);
    deserialize_integer_key!(deserialize_u32 => visit_u32);
    deserialize_integer_key!(deserialize_u64 => visit_u64);

    serde_if_integer128! {
        deserialize_integer_key!(deserialize_i128 => visit_i128);
        deserialize_integer_key!(deserialize_u128 => visit_u128);
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        // Map keys cannot be null.
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        self.key
            .into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        bool f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct BorrowedCowStrDeserializer<'de> {
    value: Cow<'de, str>,
}

impl<'de> BorrowedCowStrDeserializer<'de> {
    fn new(value: Cow<'de, str>) -> Self {
        BorrowedCowStrDeserializer { value: value }
    }
}

impl<'de> serde::Deserializer<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = SerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de> serde::de::EnumAccess<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = SerdeError;
    type Variant = UnitOnly;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
        where
            T: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self)?;
        Ok((value, UnitOnly))
    }
}

struct UnitOnly;

impl<'de> serde::de::VariantAccess<'de> for UnitOnly {
    type Error = SerdeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
        where
            T: serde::de::DeserializeSeed<'de>,
    {
        Err(SerdeError::from_str("newtype variant"))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
    {
        Err(SerdeError::from_str("tuple variant"))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
    {
        Err(SerdeError::from_str("struct variant"))
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<RefValueWrapper>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = SerdeError;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Self::Error>
        where
            V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<RefValueWrapper>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = SerdeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value.deref()),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
        where
            T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value.deref()),
            None => Err(SerdeError::from_str("newtype variant")),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self.value {
            Some(ref_value) => {
                match ref_value.deref() {
                    RefValue::Array(vec) => {
                        serde::Deserializer::deserialize_any(SeqDeserializer::new(to_vec(vec)), visitor)
                    }
                    _ => Err(SerdeError::from_str("tuple variant"))
                }
            }
            None => Err(SerdeError::from_str("tuple variant")),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
    {
        match self.value {
            Some(ref_value) => {
                match ref_value.deref() {
                    RefValue::Object(vec) => {
                        serde::Deserializer::deserialize_any(MapDeserializer::new(to_map(vec)), visitor)
                    }
                    _ => Err(SerdeError::from_str("struct variant"))
                }
            }
            _ => Err(SerdeError::from_str("struct variant")),
        }
    }
}