use crate::select::select_value::{SelectValue, SelectValueType, ValueUpdater};
use crate::select::JsonPathError;
use serde_json::map::Entry;
use serde_json::Value;
use ijson::{IValue, ValueType};

impl SelectValue for Value {
    fn get_type(&self) -> SelectValueType {
        match self {
            Value::Bool(_) => SelectValueType::Bool,
            Value::String(_) => SelectValueType::String,
            Value::Null => SelectValueType::Null,
            Value::Array(_) => SelectValueType::Array,
            Value::Object(_) => SelectValueType::Object,
            Value::Number(n) => {
                if n.is_i64() {
                    SelectValueType::Long
                } else if n.is_u64() {
                    SelectValueType::Long
                } else if n.is_f64() {
                    SelectValueType::Double
                } else {
                    panic!("bad type for Number value");
                }
            }
        }
    }

    fn contains_key(&self, key: &str) -> bool {
        match self {
            Value::Object(o) => o.contains_key(key),
            _ => false,
        }
    }

    fn values<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a Self> + 'a>> {
        match self {
            Value::Array(arr) => Some(Box::new(arr.iter())),
            Value::Object(o) => Some(Box::new(o.values())),
            _ => None,
        }
    }

    fn keys<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a str> + 'a>> {
        match self {
            Value::Object(o) => Some(Box::new(o.keys().map(|k| &k[..]))),
            _ => None,
        }
    }

    fn items<'a>(&'a self) -> Option<Box<dyn Iterator<Item = (&'a str, &'a Self)> + 'a>>{
        match self {
            Value::Object(o) => Some(Box::new(o.iter().map(|(k,v)| (&k[..],v)))),
            _ => None,
        }
    }

    fn len(&self) -> Option<usize> {
        match self {
            Value::Array(arr) => Some(arr.len()),
            Value::Object(obj) => Some(obj.len()),
            _ => None,
        }
    }

    fn get_key<'a>(&'a self, key: &str) -> Option<&'a Self> {
        match self {
            Value::Object(o) => o.get(key),
            _ => None,
        }
    }

    fn get_index<'a>(&'a self, index: usize) -> Option<&'a Self> {
        match self {
            Value::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    fn is_array(&self) -> bool {
        match self {
            Value::Array(_) => true,
            _ => false,
        }
    }

    fn get_str(&self) -> String {
        match self {
            Value::String(s) => s.to_string(),
            _ => {
                panic!("not a string");
            }
        }
    }

    fn as_str<'a>(&'a self) -> &'a str {
        match self {
            Value::String(s) => s.as_str(),
            _ => {
                panic!("not a string");
            }
        }
    }

    fn get_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => {
                assert!(false, "not a bool");
                false
            }
        }
    }

    fn get_long(&self) -> i64 {
        match self {
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    n.as_i64().unwrap()
                } else {
                    assert!(false, "not a long");
                    0
                }
            }
            _ => {
                assert!(false, "not a long");
                0
            }
        }
    }

    fn get_double(&self) -> f64 {
        match self {
            Value::Number(n) => {
                if n.is_f64() {
                    n.as_f64().unwrap()
                } else {
                    assert!(false, "not a double");
                    0.1
                }
            }
            _ => {
                assert!(false, "not a double");
                0.1
            }
        }
    }
}

impl SelectValue for IValue {
    fn get_type(&self) -> SelectValueType {
        match self.type_() {
            ValueType::Bool => SelectValueType::Bool,
            ValueType::String => SelectValueType::String,
            ValueType::Null => SelectValueType::Null,
            ValueType::Array => SelectValueType::Array,
            ValueType::Object => SelectValueType::Object,
            ValueType::Number => {
                let num = self.as_number().unwrap();
                if num.has_decimal_point() {
                    SelectValueType::Double
                } else {
                    SelectValueType::Long
                } 
            }
        }
    }

    fn contains_key(&self, key: &str) -> bool {
        match self.as_object() {
            Some(o) => o.contains_key(key),
            _ => false,
        }
    }

    fn values<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a Self> + 'a>> {
        if let Some(arr) = self.as_array(){
            Some(Box::new(arr.iter()))
        } else if  let Some(o) = self.as_object(){
            Some(Box::new(o.values()))
        } else {
            None
        }
    }

    fn keys<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a str> + 'a>> {
        match self.as_object() {
            Some(o) => Some(Box::new(o.keys().map(|k| &k[..]))),
            _ => None,
        }
    }

    fn items<'a>(&'a self) -> Option<Box<dyn Iterator<Item = (&'a str, &'a Self)> + 'a>>{
        match self.as_object() {
            Some(o) => Some(Box::new(o.iter().map(|(k,v)| (&k[..],v)))),
            _ => None,
        }
    }

    fn len(&self) -> Option<usize> {
        if let Some(arr) = self.as_array(){
            Some(arr.len())
        } else if let Some(obj) = self.as_object(){
            Some(obj.len())
        } else {
            None
        }
    }

    fn get_key<'a>(&'a self, key: &str) -> Option<&'a Self> {
        match self.as_object() {
            Some(o) => o.get(key),
            _ => None,
        }
    }

    fn get_index<'a>(&'a self, index: usize) -> Option<&'a Self> {
        match self.as_array() {
            Some(arr) => arr.get(index),
            _ => None,
        }
    }

    fn is_array(&self) -> bool {
        self.is_array() 
    }

    fn get_str(&self) -> String {
        match self.as_string() {
            Some(s) => s.to_string(),
            _ => {
                panic!("not a string");
            }
        }
    }

    fn as_str<'a>(&'a self) -> &'a str {
        match self.as_string() {
            Some(s) => s.as_str(),
            _ => {
                panic!("not a string");
            }
        }
    }

    fn get_bool(&self) -> bool {
        match self.to_bool() {
            Some(b) => b,
            _ => {
                assert!(false, "not a bool");
                false
            }
        }
    }

    fn get_long(&self) -> i64 {
        match self.as_number() {
            Some(n) => {
                if n.has_decimal_point() {
                    assert!(false, "not a long");
                    0
                } else {
                    n.to_i64().unwrap()
                }
            }
            _ => {
                assert!(false, "not a long");
                0
            }
        }
    }

    fn get_double(&self) -> f64 {
        match self.as_number() {
            Some(n) => {
                if n.has_decimal_point() {
                    n.to_f64().unwrap()
                } else {
                    assert!(false, "not a double");
                    0.1
                }
            }
            _ => {
                assert!(false, "not a double");
                0.1
            }
        }
    }
}


pub struct JsonValueUpdater<F: FnMut(Value) -> Option<Value>> {
    func: F,
}

impl<F> JsonValueUpdater<F>
where
    F: FnMut(Value) -> Option<Value>,
{
    pub fn new(func: F) -> JsonValueUpdater<F> {
        JsonValueUpdater { func: func }
    }
}

impl<F> ValueUpdater<Value> for JsonValueUpdater<F>
where
    F: FnMut(Value) -> Option<Value>,
{
    fn update(
        &mut self,
        mut path: Vec<String>,
        root: &mut Value,
    ) -> Result<&mut Self, JsonPathError> {
        let mut target = root;

        let last_index = path.len().saturating_sub(1);
        for (i, token) in path.drain(..).enumerate() {
            let target_once = target;
            let is_last = i == last_index;
            let target_opt = match *target_once {
                Value::Object(ref mut map) => {
                    if is_last {
                        if let Entry::Occupied(mut e) = map.entry(token) {
                            let v = e.insert(Value::Null);
                            if let Some(res) = (self.func)(v) {
                                e.insert(res);
                            } else {
                                e.remove();
                            }
                        }
                        return Ok(self);
                    }
                    map.get_mut(&token)
                }
                Value::Array(ref mut vec) => {
                    if let Ok(x) = token.parse::<usize>() {
                        if is_last {
                            let v = std::mem::replace(&mut vec[x], Value::Null);
                            if let Some(res) = (self.func)(v) {
                                vec[x] = res;
                            } else {
                                vec.remove(x);
                            }
                            return Ok(self);
                        }
                        vec.get_mut(x)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(t) = target_opt {
                target = t;
            } else {
                break;
            }
        }

        Ok(self)
    }
}
