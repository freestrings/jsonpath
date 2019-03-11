use indexmap::map::IndexMap;
use ref_value::*;

use super::cmp::*;
use super::term::*;
use super::value_filter::*;

#[derive(Debug)]
pub struct ValueWrapper {
    val: RefValueWrapper,
    is_leaves: bool,
}

impl ValueWrapper {
    pub fn new(val: RefValueWrapper, leaves: bool) -> Self {
        ValueWrapper { val, is_leaves: leaves }
    }

    pub fn is_leaves(&self) -> bool {
        self.is_leaves
    }

    pub fn set_leaves(&mut self, is_leaves: bool) {
        self.is_leaves = is_leaves;
    }

    pub fn cmp(&self, other: &ValueWrapper, cmp_type: CmpType) -> TermContext {
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

    fn cmp_with_term<F: PrivCmp>(val: &RefValueWrapper, et: &ExprTerm, cmp_fn: &F, default: bool, reverse: bool) -> bool {
        match val.get_data_ref() {
            RefValue::Bool(ref v1) => {
                match et {
                    ExprTerm::Bool(v2) => if reverse { cmp_fn.cmp_bool(v2, v1) } else { cmp_fn.cmp_bool(v1, v2) },
                    _ => default
                }
            }
            RefValue::Number(ref v1) => match et {
                ExprTerm::Number(v2) => if reverse { cmp_fn.cmp_f64(v2, &v1.as_f64().unwrap()) } else { cmp_fn.cmp_f64(&v1.as_f64().unwrap(), v2) },
                _ => default
            },
            RefValue::String(ref v1) => {
                match et {
                    ExprTerm::String(v2) => if reverse { cmp_fn.cmp_string(v2, v1) } else { cmp_fn.cmp_string(v1, v2) },
                    _ => default
                }
            }
            _ => default
        }
    }

    fn take_object_in_array<F: PrivCmp>(&self, key: &String, et: &ExprTerm, cmp: &F, reverse: bool) -> Option<Self> {
        fn _filter_with_object<F: Fn(&RefValueWrapper) -> bool>(v: &RefValueWrapper, key: &String, fun: F) -> bool {
            match v.get_data_ref() {
                RefValue::Object(map) => {
                    match map.get(key) {
                        Some(val) => fun(&val.into()),
                        _ => false
                    }
                }
                _ => false
            }
        }

        match self.val.get_data_ref() {
            RefValue::Array(vec) => {
                let mut ret = Vec::new();
                for v in vec {
                    if _filter_with_object(&v.into(), key, |vv| {
                        Self::cmp_with_term(vv, et, cmp, false, reverse)
                    }) {
                        ret.push(v.clone());
                    }
                }

                Some(ValueWrapper::new(RefValue::Array(ret).into(), false))
            }
            _ => None
        }
    }

    fn take_with_key_type<F: PrivCmp>(&self, key: &Option<ValueFilterKey>, et: &ExprTerm, cmp: &F, reverse: bool) -> Option<Self> {
        match key {
            Some(ValueFilterKey::String(key)) => {
                self.take_object_in_array(key, et, cmp, reverse)
            }
            _ => None
        }
    }

    pub fn take_with<F: PrivCmp>(&self, key: &Option<ValueFilterKey>, et: &ExprTerm, cmp: F, reverse: bool) -> Self {
        match self.take_with_key_type(key, et, &cmp, reverse) {
            Some(vw) => vw,
            _ => {
                match self.val.get_data_ref() {
                    RefValue::Array(vec) => {
                        let mut ret = Vec::new();
                        for v in vec {
                            if Self::cmp_with_term(&v.into(), et, &cmp, false, reverse) {
                                ret.push(v.clone());
                            }
                        }
                        ValueWrapper::new(RefValue::Array(ret).into(), false)
                    }
                    _ => {
                        if Self::cmp_with_term(&self.val, et, &cmp, false, reverse) {
                            ValueWrapper::new(self.val.clone(), false)
                        } else {
                            ValueWrapper::new(RefValue::Null.into(), false)
                        }
                    }
                }
            }
        }
    }

    pub fn replace(&mut self, val: RefValueWrapper) {
        let is_null = match val.get_data_ref() {
            RefValue::Array(v) => if v.is_empty() { true } else { false },
            RefValue::Object(m) => if m.is_empty() { true } else { false },
            _ => val.is_null()
        };
        self.val = if is_null {
            let v = RefValueWrapper::wrap(RefValue::Null);
            RefValueWrapper::new(v)
        } else {
            val
        };
    }

    pub fn get_val(&self) -> &RefValueWrapper {
        &self.val
    }

    pub fn clone_val(&self) -> RefValueWrapper {
        self.val.clone()
    }

    pub fn is_array(&self) -> bool {
        self.val.is_array()
    }

    fn uuid(v: &RefValueWrapper) -> String {
        fn _fn(v: &RefValueWrapper, acc: &mut String) {
            match v.get_data_ref() {
                RefValue::Null => acc.push_str("null"),
                RefValue::String(v) => acc.push_str(v),
                RefValue::Bool(v) => acc.push_str(if *v { "true" } else { "false" }),
                RefValue::Number(v) => acc.push_str(&*v.to_string()),
                RefValue::Array(v) => {
                    for (i, v) in v.iter().enumerate() {
                        acc.push_str(&*i.to_string());
                        _fn(&v.into(), acc);
                    }
                }
                RefValue::Object(ref v) => {
                    for (k, v) in v.into_iter() {
                        acc.push_str(&*k.to_string());
                        _fn(&v.into(), acc);
                    }
                }
            }
        }
        let mut acc = String::new();
        _fn(v, &mut acc);
        acc
    }

    fn into_map(&self) -> IndexMap<String, RefValueWrapper> {
        let mut map = IndexMap::new();
        match self.val.get_data_ref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    let wrapper = v.into();
                    let key = Self::uuid(&wrapper);
                    map.insert(key, wrapper);
                }
            }
            _ => {
                map.insert(Self::uuid(&self.val), self.val.clone());
            }
        }
        map
    }

    pub fn except(&self, other: &Self) -> Self {
        let map = self.into_map();
        let mut ret: IndexMap<String, RefValueWrapper> = IndexMap::new();
        match other.val.get_data_ref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    let wrapper = v.into();
                    let key = Self::uuid(&wrapper);
                    if !map.contains_key(&key) {
                        ret.insert(key, wrapper);
                    }
                }
            }
            _ => {
                let key = Self::uuid(&other.val);
                if !map.contains_key(&key) {
                    ret.insert(key, other.val.clone());
                }
            }
        }

        let vec = ret.values().into_iter().map(|v| v.clone_data()).collect();
        ValueWrapper::new(RefValue::Array(vec).into(), false)
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let map = self.into_map();
        let mut ret: IndexMap<String, RefValueWrapper> = IndexMap::new();
        match other.val.get_data_ref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    let wrapper = v.into();
                    let key = Self::uuid(&wrapper);
                    if map.contains_key(&key) {
                        ret.insert(key, wrapper);
                    }
                }
            }
            _ => {
                let key = Self::uuid(&other.val);
                if map.contains_key(&key) {
                    ret.insert(key, other.val.clone());
                }
            }
        }

        let vec = ret.values().into_iter().map(|v| v.clone_data()).collect();
        ValueWrapper::new(RefValue::Array(vec).into(), false)
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut map = self.into_map();
        match other.val.get_data_ref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    let wrapper = v.into();
                    let key = Self::uuid(&wrapper);
                    if !map.contains_key(&key) {
                        map.insert(key, wrapper);
                    }
                }
            }
            _ => {
                let key = Self::uuid(&other.val);
                if !map.contains_key(&key) {
                    map.insert(key, other.val.clone());
                }
            }
        }

        let mut vw = ValueWrapper::new(RefValue::Null.into(), false);
        let list = map.values().into_iter().map(|val| val.clone_data()).collect();
        vw.replace(RefValue::Array(list).into());
        vw
    }

    pub fn into_term(&self, key: &Option<ValueFilterKey>) -> TermContext {
        match self.val.get_data_ref() {
            RefValue::String(ref s) => TermContext::Constants(ExprTerm::String(s.clone())),
            RefValue::Number(ref n) => TermContext::Constants(ExprTerm::Number(n.as_f64().unwrap())),
            RefValue::Bool(b) => TermContext::Constants(ExprTerm::Bool(*b)),
            _ => TermContext::Json(match key {
                Some(vk) => Some(vk.clone()),
                _ => None
            }, ValueWrapper::new(self.val.clone(), false))
        }
    }

    pub fn filter(&self, key: &Option<ValueFilterKey>) -> Self {
        let v = match self.val.get_data_ref() {
            RefValue::Array(ref vec) => {
                let mut ret = Vec::new();
                for v in vec {
                    if let Some(ValueFilterKey::String(k)) = key {
                        let wrapper: RefValueWrapper = v.into();
                        if wrapper.get(k.clone()).is_some() {
                            ret.push(v.clone());
                        }
                    }
                }
                RefValue::Array(ret).into()
            }
            RefValue::Object(ref map) => {
                match key {
                    Some(ValueFilterKey::String(k)) => match map.get(k) {
                        Some(v) => v.into(),
                        _ => RefValue::Null.into()
                    },
                    _ => RefValue::Null.into()
                }
            }
            _ => self.val.clone()
        };

        ValueWrapper::new(v, false)
    }
}
