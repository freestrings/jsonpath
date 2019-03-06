use indexmap::map::IndexMap;

use serde_json::Value;

use super::cmp::*;
use super::term::*;
use super::value_filter::*;

#[derive(Debug)]
pub struct ValueWrapper {
    val: Value,
    is_leaves: bool,
}

impl ValueWrapper {
    pub fn new(val: Value, leaves: bool) -> Self {
        ValueWrapper { val, is_leaves: leaves }
    }

    pub fn is_leaves(&self) -> bool {
        self.is_leaves
    }

    pub fn set_leaves(&mut self, is_leaves: bool) {
        self.is_leaves = is_leaves;
    }

    pub fn cmp(&mut self, other: &mut ValueWrapper, cmp_type: CmpType) -> TermContext {
        match cmp_type {
            CmpType::Eq => {
                TermContext::Json(None, self.intersect(other))
            }
            CmpType::Ne => {
                TermContext::Json(None, self.except(other))
            }
            CmpType::Gt | CmpType::Ge | CmpType::Lt | CmpType::Le => {
                TermContext::Constants(ExprTerm::Bool(false))
            }
        }
    }

    fn cmp_with_term<F: PrivCmp>(val: &Value, et: &ExprTerm, cmp_fn: &F, default: bool, reverse: bool) -> bool {
        match val {
            Value::Bool(ref v1) => {
                match et {
                    ExprTerm::Bool(v2) => if reverse { cmp_fn.cmp_bool(v2, v1) } else { cmp_fn.cmp_bool(v1, v2) },
                    _ => default
                }
            }
            Value::Number(ref v1) => match v1.as_f64() {
                Some(ref v1) => {
                    match et {
                        ExprTerm::Number(v2) => if reverse { cmp_fn.cmp_f64(v2, v1) } else { cmp_fn.cmp_f64(v1, v2) },
                        _ => default
                    }
                }
                _ => default
            },
            Value::String(ref v1) => {
                match et {
                    ExprTerm::String(v2) => if reverse { cmp_fn.cmp_string(v2, v1) } else { cmp_fn.cmp_string(v1, v2) },
                    _ => default
                }
            }
            _ => default
        }
    }

    fn take_object_in_array<F: PrivCmp>(&mut self, key: &String, et: &ExprTerm, cmp: &F, reverse: bool) -> Option<Self> {
        fn _filter_with_object<F: Fn(&Value) -> bool>(v: &&mut Value, key: &String, fun: F) -> bool {
            match &v {
                Value::Object(map) => {
                    match map.get(key) {
                        Some(vv) => fun(vv),
                        _ => false
                    }
                }
                _ => false
            }
        }

        match self.val.take() {
            Value::Array(mut vec) => {
                let mut ret: Vec<Value> = vec.iter_mut()
                    .filter(|v| {
                        _filter_with_object(v, key, |vv| Self::cmp_with_term(vv, et, cmp, false, reverse))
                    })
                    .map(|v| v.take())
                    .collect();
                Some(ValueWrapper::new(Value::Array(ret), false))
            }
            _ => None
        }
    }

    fn take_with_key_type<F: PrivCmp>(&mut self, key: &Option<ValueFilterKey>, et: &ExprTerm, cmp: &F, reverse: bool) -> Option<Self> {
        match key {
            Some(ValueFilterKey::String(key)) => {
                self.take_object_in_array(key, et, cmp, reverse)
            }
            _ => None
        }
    }

    pub fn take_with<F: PrivCmp>(&mut self, key: &Option<ValueFilterKey>, et: &ExprTerm, cmp: F, reverse: bool) -> Self {
        match self.take_with_key_type(key, et, &cmp, reverse) {
            Some(vw) => vw,
            _ => {
                match self.val.take() {
                    Value::Array(mut vec) => {
                        let mut ret = vec.iter_mut()
                            .filter(|v| Self::cmp_with_term(&v, et, &cmp, false, reverse))
                            .map(|v| v.take())
                            .collect();
                        ValueWrapper::new(Value::Array(ret), false)
                    }
                    other => {
                        if Self::cmp_with_term(&other, et, &cmp, false, reverse) {
                            ValueWrapper::new(other, false)
                        } else {
                            ValueWrapper::new(Value::Null, false)
                        }
                    }
                }
            }
        }
    }

    pub fn replace(&mut self, val: Value) {
        let is_null = match &val {
            Value::Array(v) => if v.is_empty() { true } else { false },
            Value::Object(m) => if m.is_empty() { true } else { false },
            _ => val.is_null()
        };
        self.val = if is_null { Value::Null } else { val };
    }

    pub fn get_val(&self) -> &Value {
        &self.val
    }

    pub fn get_val_mut(&mut self) -> &mut Value {
        &mut self.val
    }

    pub fn clone_val(&self) -> Value {
        self.val.clone()
    }

    pub fn is_array(&self) -> bool {
        self.val.is_array()
    }

    fn uuid(v: &Value) -> String {
        fn _fn(v: &Value) -> String {
            match v {
                Value::Null => "null".to_string(),
                Value::String(v) => v.to_string(),
                Value::Bool(v) => v.to_string(),
                Value::Number(v) => v.to_string(),
                Value::Array(v) => {
                    v.iter().enumerate()
                        .map(|(i, v)| { format!("{}{}", i, _fn(v)) })
                        .collect()
                }
                Value::Object(v) => {
                    v.into_iter().map(|(k, v)| { format!("{}{}", k, _fn(v)) }).collect()
                }
            }
        }
        _fn(v)
    }

    fn into_map(&mut self) -> IndexMap<String, Value> {
        let mut map = IndexMap::new();
        match &mut self.val {
            Value::Array(v1) => {
                for v in v1 {
                    map.insert(Self::uuid(v), v.take());
                }
            }
            other => {
                map.insert(Self::uuid(other), other.take());
            }
        }
        map
    }

    pub fn except(&mut self, other: &mut Self) -> Self {
        let map = self.into_map();
        let mut ret: IndexMap<String, Value> = IndexMap::new();
        match &mut other.val {
            Value::Array(v1) => {
                for v in v1 {
                    let key = Self::uuid(v);
                    if !map.contains_key(&key) {
                        ret.insert(key, v.take());
                    }
                }
            }
            other => {
                let key = Self::uuid(other);
                if !map.contains_key(&key) {
                    ret.insert(key, other.take());
                }
            }
        }

        let v = ret.values_mut().into_iter().map(|v| v.take()).collect();
        ValueWrapper::new(v, false)
    }

    pub fn intersect(&mut self, other: &mut Self) -> Self {
        let map = self.into_map();
        let mut ret: IndexMap<String, Value> = IndexMap::new();
        match &mut other.val {
            Value::Array(v1) => {
                for v in v1 {
                    let key = Self::uuid(v);
                    if map.contains_key(&key) {
                        ret.insert(key, v.take());
                    }
                }
            }
            other => {
                let key = Self::uuid(other);
                if map.contains_key(&key) {
                    ret.insert(key, other.take());
                }
            }
        }

        let v = ret.values_mut().into_iter().map(|v| v.take()).collect();
        ValueWrapper::new(v, false)
    }

    pub fn union(&mut self, other: &mut Self) -> Self {
        let mut map = self.into_map();
        match &mut other.val {
            Value::Array(v1) => {
                for v in v1 {
                    let key = Self::uuid(v);
                    if !map.contains_key(&key) {
                        map.insert(key, v.take());
                    }
                }
            }
            other => {
                let key = Self::uuid(other);
                if !map.contains_key(&key) {
                    map.insert(key, other.take());
                }
            }
        }

        let mut vw = ValueWrapper::new(Value::Null, false);
        let list: Vec<Value> = map.values_mut().into_iter().map(|val| val.take()).collect();
        vw.replace(Value::Array(list));
        vw
    }

    pub fn into_term(&mut self, key: &mut Option<ValueFilterKey>) -> TermContext {
        match self.val.take() {
            Value::String(s) => TermContext::Constants(ExprTerm::String(s)),
            Value::Number(n) => TermContext::Constants(ExprTerm::Number(n.as_f64().unwrap())),
            Value::Bool(b) => TermContext::Constants(ExprTerm::Bool(b)),
            other => TermContext::Json(match key {
                Some(vk) => Some(vk.clone()),
                _ => None
            }, ValueWrapper::new(other, false))
        }
    }

    pub fn filter(&mut self, key: &mut Option<ValueFilterKey>) -> Self {
        let v = match &mut self.val {
            Value::Array(vec) => {
                let ret = vec.iter_mut()
                    .filter(|v| match key {
                        Some(ValueFilterKey::String(val_key)) => {
                            v.get(val_key.as_str()).is_some()
                        }
                        _ => false
                    })
                    .map(|v| v.take())
                    .collect();
                Value::Array(ret)
            }
            Value::Object(map) => {
                match key {
                    Some(ValueFilterKey::String(val_key)) => match map.get_mut(val_key) {
                        Some(v) => v.take(),
                        _ => Value::Null
                    },
                    _ => Value::Null
                }
            }
            other => other.take()
        };

        ValueWrapper::new(v, false)
    }
}
