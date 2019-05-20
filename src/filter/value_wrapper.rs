use std::ops::Deref;

use indexmap::IndexSet;
use serde_json::Value;

use ref_value::model::*;

use super::cmp::*;
use super::term::*;
use super::value_filter::*;

fn cmp_with_term<F: PrivCmp>(val: &RefValueWrapper, et: &ExprTerm, cmp_fn: &F, default: bool, reverse: bool) -> bool {
    match val.deref() {
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

    fn take_object_in_array<F: PrivCmp>(&self, key: &String, et: &ExprTerm, cmp: &F, reverse: bool) -> Option<Self> {
        fn _filter_with_object<F: Fn(&RefValueWrapper) -> bool>(v: &RefValueWrapper, key: &String, fun: F) -> bool {
            match v.deref() {
                RefValue::Object(map) => {
                    match map.get(key) {
                        Some(val) => fun(val),
                        _ => false
                    }
                }
                _ => false
            }
        }

        match self.val.deref() {
            RefValue::Array(vec) => {
                let mut set = IndexSet::new();
                for v in vec {
                    if _filter_with_object(v, key, |vv| {
                        cmp_with_term(vv, et, cmp, false, reverse)
                    }) {
                        set.insert(v.clone());
                    }
                }
                let ret = set.into_iter().collect();
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
                match &(*self.val) {
                    RefValue::Array(vec) => {
                        let mut set = IndexSet::new();
                        for v in vec {
                            if cmp_with_term(v, et, &cmp, false, reverse) {
                                set.insert(v.clone());
                            }
                        }
                        let ret = set.into_iter().collect();
                        ValueWrapper::new(RefValue::Array(ret).into(), false)
                    }
                    _ => {
                        if cmp_with_term(&self.val, et, &cmp, false, reverse) {
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
        let is_null = match val.deref() {
            RefValue::Array(v) => v.is_empty(),
            RefValue::Object(m) => m.is_empty(),
            _ => val.is_null()
        };
        self.val = if is_null {
            RefValue::Null.into()
        } else {
            val
        };
    }

    pub fn get_val(&self) -> &RefValueWrapper {
        &self.val
    }

    pub fn into_value(&self) -> Value {
        self.get_val().into()
    }

    pub fn is_array(&self) -> bool {
        self.val.is_array()
    }

    fn into_hashset(&self) -> IndexSet<RefValueWrapper> {
        trace!("into_hashset");
        let mut hashset = IndexSet::new();
        match self.val.deref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    hashset.insert(v.clone());
                }
            }
            _ => {
                hashset.insert(self.val.clone());
            }
        }
        hashset
    }

    pub fn except(&self, other: &Self) -> Self {
        trace!("except");
        let hashset = self.into_hashset();
        let mut ret: IndexSet<RefValueWrapper> = IndexSet::new();
        match other.val.deref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    if !hashset.contains(v) {
                        ret.insert(v.clone());
                    }
                }
            }
            _ => {
                if !hashset.contains(&other.val) {
                    ret.insert(other.val.clone());
                }
            }
        }

        let vec = ret.into_iter().map(|v| v.clone()).collect();
        ValueWrapper::new(RefValue::Array(vec).into(), false)
    }

    pub fn intersect(&self, other: &Self) -> Self {
        trace!("intersect");
        let hashset = self.into_hashset();
        let mut ret: IndexSet<RefValueWrapper> = IndexSet::new();
        match other.val.deref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    if hashset.contains(v) {
                        ret.insert(v.clone());
                    }
                }
            }
            _ => {
                if hashset.contains(&other.val) {
                    ret.insert(other.val.clone());
                }
            }
        }

        let vec = ret.into_iter().map(|v| v.clone()).collect();
        ValueWrapper::new(RefValue::Array(vec).into(), false)
    }

    pub fn union(&self, other: &Self) -> Self {
        trace!("union");
        let mut hashset = self.into_hashset();
        match other.val.deref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    if !hashset.contains(v) {
                        hashset.insert(v.clone());
                    }
                }
            }
            _ => {
                if !hashset.contains(&other.val) {
                    hashset.insert(other.val.clone());
                }
            }
        }

        let mut vw = ValueWrapper::new(RefValue::Null.into(), false);
        let list = hashset.into_iter().map(|val| val.clone()).collect();
        vw.replace(RefValue::Array(list).into());
        vw
    }

    pub fn into_term(&self, key: &Option<ValueFilterKey>) -> TermContext {
        match self.val.deref() {
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
        trace!("filter");
        let v = match self.val.deref() {
            RefValue::Array(ref vec) => {
                let mut ret = Vec::new();
                for v in vec {
                    if let Some(ValueFilterKey::String(k)) = key {
                        if v.get(k.clone()).is_some() {
                            ret.push(v.clone());
                        }
                    }
                }
                RefValue::Array(ret).into()
            }
            RefValue::Object(ref map) => {
                match key {
                    Some(ValueFilterKey::String(k)) => match map.get(k) {
                        Some(v) => v.clone(),
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
