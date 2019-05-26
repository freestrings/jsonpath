use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

use indexmap::map::IndexMap;
use serde::ser::Serialize;
use serde_json::{Number, Value};
use std::fmt;

type TypeRefValue = Arc<RefCell<RefValue>>;

pub struct RefValueWrapper {
    data: TypeRefValue,
}

impl fmt::Debug for RefValueWrapper {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl RefValueWrapper {
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}

impl PartialEq for RefValueWrapper {
    fn eq(&self, other: &RefValueWrapper) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl Eq for RefValueWrapper {}

impl Deref for RefValueWrapper {
    type Target = RefValue;

    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ptr().as_mut().unwrap() }
    }
}

//impl DerefMut for RefValueWrapper {
//    fn deref_mut(&mut self) -> &mut RefValue {
//        unsafe { self.data.as_ptr().as_mut().unwrap() }
//    }
//}

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

pub enum RefValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<RefValueWrapper>),
    Object(IndexMap<String, RefValueWrapper>),
}

impl fmt::Debug for RefValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl PartialEq for RefValue {
    fn eq(&self, other: &RefValue) -> bool {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        self.hash(&mut hasher1);
        other.hash(&mut hasher2);

        hasher1.finish() == hasher2.finish()
    }
}

static REF_VALUE_NULL: &'static str = "$jsonpath::ref_value::model::RefValue::Null";

impl Hash for RefValue {
    fn hash<H: Hasher>(&self, state: &mut H) {

//        println!("###hash - RefValue - {:?}", self);

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
                for (k, v) in map {
                    k.hash(state);
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
        match *self {
            RefValue::Object(_) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match *self {
            RefValue::Array(_) => true,
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        match &self {
            RefValue::Object(m) => m.len(),
            RefValue::Array(v) => v.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self {
            RefValue::Object(m) => m.is_empty(),
            RefValue::Array(v) => v.is_empty(),
            RefValue::Null => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match *self {
            RefValue::Null => true,
            _ => false,
        }
    }
}

impl Into<RefValueWrapper> for RefValue {
    fn into(self) -> RefValueWrapper {
        RefValueWrapper {
            data: Arc::new(RefCell::new(self))
        }
    }
}

impl Into<RefValue> for &Value {
    fn into(self) -> RefValue {
        match self.serialize(super::ser::RefValueSerializer) {
            Ok(v) => v,
            Err(e) => panic!("Error Value into RefValue: {:?}", e)
        }
    }
}

impl Into<RefValueWrapper> for &Value {
    fn into(self) -> RefValueWrapper {
        match self.serialize(super::ser::RefValueSerializer) {
            Ok(v) => v.into(),
            Err(e) => panic!("Error Value into RefValue: {:?}", e)
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