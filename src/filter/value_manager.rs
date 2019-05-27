use std::ops::Deref;

use indexmap::{IndexMap, IndexSet};
use serde_json::Value;

use ref_value::model::*;

use super::cmp::*;
use super::term::*;
use super::value_filter::*;

pub trait ArrayIndex {
    fn index(&self, v: &RefValueWrapper) -> usize;

    fn ref_value(&self, v: &RefValueWrapper) -> RefValueWrapper {
        let idx = self.index(v);
        match v.get(idx) {
            Some(v) => v.clone(),
            _ => RefValue::Null.into()
        }
    }
}

impl ArrayIndex for f64 {
    fn index(&self, v: &RefValueWrapper) -> usize {
        if v.is_array() && self < &0_f64 {
            (v.len() as f64 + self) as usize
        } else {
            *self as usize
        }
    }
}

impl ArrayIndex for isize {
    fn index(&self, v: &RefValueWrapper) -> usize {
        if v.is_array() && self < &0_isize {
            (v.len() as isize + self) as usize
        } else {
            *self as usize
        }
    }
}

impl ArrayIndex for usize {
    fn index(&self, _: &RefValueWrapper) -> usize {
        *self as usize
    }
}

fn cmp_with_term<F: PrivCmp>(val: &RefValueWrapper, et: &ExprTerm, cmp_fn: &F, default: bool, reverse: bool) -> bool {
    match val.deref() {
        RefValue::Bool(ref v1) => {
            match et {
                ExprTerm::Bool(v2) => if reverse {
                    cmp_fn.cmp_bool(v2, v1)
                } else {
                    cmp_fn.cmp_bool(v1, v2)
                },
                _ => default
            }
        }
        RefValue::Number(ref v1) => match et {
            ExprTerm::Number(v2) => if reverse {
                cmp_fn.cmp_f64(v2, &v1.as_f64().unwrap())
            } else {
                cmp_fn.cmp_f64(&v1.as_f64().unwrap(), v2)
            },
            _ => default
        },
        RefValue::String(ref v1) => {
            match et {
                ExprTerm::String(v2) => if reverse {
                    cmp_fn.cmp_string(v2, v1)
                } else {
                    cmp_fn.cmp_string(v1, v2)
                },
                _ => default
            }
        }
        _ => default
    }
}

fn collect_not_null<'a,
    I: Iterator<Item=&'a RefValueWrapper>,
    F: FnMut(&RefValueWrapper) -> RefValueWrapper>(iter: I, func: F) -> Vec<RefValueWrapper>
{
    iter.map(func)
        .filter(|v| !v.is_null())
        .collect()
}

fn collect_some<'a,
    I: Iterator<Item=&'a RefValueWrapper>,
    F: FnMut(&RefValueWrapper) -> Option<RefValueWrapper>>(iter: I, func: F) -> Vec<RefValueWrapper>
{
    iter.map(func)
        .filter(|v| v.is_some())
        .map(|v| v.unwrap())
        .collect()
}

fn get_in_array<I: ArrayIndex>(v: &RefValueWrapper, key: &I, is_relative: bool) -> Option<RefValueWrapper> {
    match v.deref() {
        RefValue::Array(vec) if vec.get(key.index(v)).is_some() => {
            Some(if is_relative { v.clone() } else { v.get(key.index(v)).unwrap().clone() })
        }
        _ => None
    }
}

fn get_in_object(v: &RefValueWrapper, key: &str, is_relative: bool) -> Option<RefValueWrapper> {
    match v.deref() {
        RefValue::Object(map) if map.contains_key(key) => {
            Some(if is_relative { v.clone() } else { v.get(key.to_string()).unwrap().clone() })
        }
        _ => None
    }
}

fn get_in_nested_array<I: ArrayIndex>(v: &RefValueWrapper, key: &I, is_relative: bool) -> Option<RefValueWrapper> {
    match v.deref() {
        RefValue::Array(vec) => {
            let ret = collect_some(vec.iter(), |v| { get_in_array(v, key, is_relative) });
            Some(RefValue::Array(ret).into())
        }
        _ => None
    }
}

fn get_object_in_array(val: &RefValueWrapper, key: &str, is_relative: bool) -> Option<RefValueWrapper> {
    match val.deref() {
        RefValue::Array(vec) => {
            let ret = collect_some(vec.iter(), |v| get_in_object(v, key, is_relative));
            Some(RefValue::Array(ret).into())
        }
        _ => None
    }
}

fn get_in_nested_object(val: &RefValueWrapper, key: &str, is_relative: bool) -> Option<RefValueWrapper> {
    match val.deref() {
        RefValue::Array(vec) => {
            let ret = vec.iter()
                .map(|v| {
                    match v.deref() {
                        RefValue::Array(vec) => {
                            Some(collect_some(vec.iter(), |v| get_in_object(v, key, is_relative)))
                        }
                        RefValue::Object(_) => {
                            match get_in_object(v, key, is_relative) {
                                Some(v) => Some(vec![v]),
                                _ => None
                            }
                        }
                        _ => None
                    }
                })
                .filter(|v| v.is_some())
                .map(|v| v.unwrap())
                .filter(|v| !v.is_empty())
                .flatten()
                .collect();
            Some(RefValue::Array(ret).into())
        }
        _ => None
    }
}

#[deprecated(since = "0.1.14", note = "Please use the ValueManager instead")]
pub type ValueWrapper = ValueManager;

#[derive(Debug)]
pub struct ValueManager {
    val: RefValueWrapper,
    is_leaves: bool,
}

impl ValueManager {
    pub fn new(val: RefValueWrapper, is_leaves: bool) -> Self {
        ValueManager { val, is_leaves }
    }

    pub fn is_leaves(&self) -> bool {
        self.is_leaves
    }

    pub fn set_leaves(&mut self, is_leaves: bool) {
        self.is_leaves = is_leaves;
    }

    pub fn cmp(&self, other: &Self, cmp_type: CmpType) -> TermContext {
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

    pub fn get_compare_with<F: PrivCmp>(&self, key: &Option<ValueFilterKey>, et: &ExprTerm, cmp: F, reverse: bool) -> Self {
        match self.val.deref() {
            RefValue::Array(vec) => {
                let mut set = IndexSet::new();
                for v in vec {
                    if let Some(ValueFilterKey::String(key)) = key {
                        if let Some(ret_v) = get_in_object(v, key, false) {
                            if cmp_with_term(&ret_v, et, &cmp, false, reverse) {
                                set.insert(v.clone());
                            }
                        }
                    } else {
                        if cmp_with_term(v, et, &cmp, false, reverse) {
                            set.insert(v.clone());
                        }
                    }
                }

                let ret = set.into_iter().collect();
                Self::new(RefValue::Array(ret).into(), false)
            }
            _ => {
                if cmp_with_term(&self.val, et, &cmp, false, reverse) {
                    Self::new(self.val.clone(), false)
                } else {
                    Self::new(RefValue::Null.into(), false)
                }
            }
        }
    }

    pub fn get_index<I: ArrayIndex>(&self, i: I) -> usize {
        i.index(&self.val)
    }

    pub fn get_with_num<I: ArrayIndex>(&self, key: &I, is_relative: bool) -> RefValueWrapper {
        if self.val.is_array() && self.is_leaves {
            match get_in_nested_array(&self.val, key, is_relative) {
                Some(v) => v,
                _ => RefValue::Null.into()
            }
        } else {
            key.ref_value(&self.val)
        }
    }

    pub fn get_as_array(&self) -> RefValueWrapper {
        let vec = match self.val.deref() {
            RefValue::Object(ref map) => {
                collect_not_null(map.values(), |v| v.clone())
            }
            RefValue::Array(ref vec) => {
                vec.clone()
            }
            RefValue::Null => Vec::new(),
            _ => vec![self.val.clone()]
        };
        RefValue::Array(vec).into()
    }

    pub fn get_with_str(&self, key: &String, is_relative: bool) -> RefValueWrapper {
        if self.val.is_array() && self.is_leaves {
            match get_in_nested_object(&self.val, key, is_relative) {
                Some(v) => v,
                _ => RefValue::Null.into()
            }
        } else if self.val.is_array() && !self.is_leaves {
            match get_object_in_array(&self.val, key, is_relative) {
                Some(v) => v,
                _ => RefValue::Null.into()
            }
        } else {
            match self.val.get(key.clone()) {
                Some(v) => v.clone(),
                _ => RefValue::Null.into()
            }
        }
    }

    pub fn range_with(&self, from: Option<isize>, to: Option<isize>) -> Option<RefValueWrapper> {
        fn _from<F: ArrayIndex>(from: Option<F>, val: &RefValueWrapper) -> usize {
            match from {
                Some(v) => v.index(val),
                _ => 0
            }
        }

        fn _to<F: ArrayIndex>(to: Option<F>, val: &RefValueWrapper) -> usize {
            match to {
                Some(v) => v.index(val),
                _ => {
                    if let RefValue::Array(v) = val.deref() {
                        v.len()
                    } else {
                        0
                    }
                }
            }
        }

        fn _range(from: usize, to: usize, v: &RefValueWrapper) -> Vec<RefValueWrapper> {
            trace!("range - {}:{}", from, to);

            (from..to).into_iter()
                .map(|i| i.ref_value(v))
                .filter(|v| !v.is_null())
                .map(|v| v.clone())
                .collect()
        }

        if let RefValue::Array(vec) = &self.val.deref() {
            let ret = if self.is_leaves {
                vec.iter()
                    .map(|v| _range(_from(from, v), _to(to, v), v))
                    .flatten()
                    .collect()
            } else {
                _range(_from(from, &self.val), _to(to, &self.val), &self.val)
            };
            Some(RefValue::Array(ret).into())
        } else {
            None
        }
    }

    pub fn pick_with_nums<I: ArrayIndex>(&self, indices: Vec<I>) -> Option<RefValueWrapper> {
        if let RefValue::Array(vec) = &self.val.deref() {
            let ret = if self.is_leaves {
                indices.iter()
                    .map(|index| collect_not_null(vec.iter(), |v| { index.ref_value(v) }))
                    .flatten()
                    .collect()
            } else {
                indices.iter()
                    .map(|index| index.ref_value(&self.val))
                    .filter(|v| !v.is_null())
                    .collect()
            };
            Some(RefValue::Array(ret).into())
        } else {
            None
        }
    }

    pub fn replace(&mut self, val: RefValueWrapper) {
        self.val = match val.deref() {
            RefValue::Array(v) if v.is_empty() => RefValue::Null.into(),
            RefValue::Object(m) if m.is_empty() => RefValue::Null.into(),
            _ => val
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

    fn into_hashset(&self) -> IndexSet<&RefValue> {
        trace!("into_hashset");
        let mut hashset = IndexSet::new();
        match self.val.deref() {
            RefValue::Array(vec) => {
                for v in vec {
                    hashset.insert(v.deref());
                }
            }
            _ => {
                hashset.insert(self.val.deref());
            }
        }
        hashset
    }

    fn into_hashmap(&self) -> IndexMap<&RefValue, RefValueWrapper> {
        trace!("into_hashmap");
        let mut hashmap = IndexMap::new();
        match self.val.deref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    hashmap.insert(v.deref(), v.clone());
                }
            }
            _ => {
                hashmap.insert(self.val.deref(), self.val.clone());
            }
        }
        hashmap
    }

    pub fn except(&self, other: &Self) -> Self {
        trace!("except");
        let hashset = self.into_hashset();
        let mut ret: IndexSet<RefValueWrapper> = IndexSet::new();

        match other.val.deref() {
            RefValue::Array(ref vec) => {
                for v in vec {
                    if !hashset.contains(v.deref()) {
                        ret.insert(v.clone());
                    }
                }
            }
            _ => {
                if !hashset.contains(&other.val.deref()) {
                    ret.insert(other.val.clone());
                }
            }
        }

        let vec = ret.into_iter().map(|v| v.clone()).collect();
        ValueManager::new(RefValue::Array(vec).into(), false)
    }

    pub fn intersect(&self, other: &Self) -> Self {
        trace!("intersect");
        let hashset = self.into_hashset();
        let mut ret: IndexSet<RefValueWrapper> = IndexSet::new();
        match other.val.deref() {
            RefValue::Array(ref v1) => {
                for v in v1 {
                    if hashset.contains(v.deref()) {
                        ret.insert(v.clone());
                    }
                }
            }
            e => {
                if hashset.contains(e) {
                    ret.insert(other.val.clone());
                }
            }
        }

        let vec = ret.into_iter().map(|v| v.clone()).collect();
        ValueManager::new(RefValue::Array(vec).into(), false)
    }

    pub fn union(&self, other: &Self) -> Self {
        trace!("union");
        let origin = self.into_hashmap();
        let mut ret = IndexSet::new();

        for o in origin.values() {
            ret.insert(o.clone());
        }

        match other.val.deref() {
            RefValue::Array(vec) => {
                for v in vec {
                    if !origin.contains_key(v.deref()) {
                        ret.insert(v.clone());
                    }
                }
            }
            _ => {
                if !origin.contains_key(&other.val.deref()) {
                    ret.insert(other.val.clone());
                }
            }
        }

        let vec = ret.into_iter().map(|v| v.clone()).collect();
        ValueManager::new(RefValue::Array(vec).into(), false)
    }

    pub fn into_term(&self, key: &Option<ValueFilterKey>) -> TermContext {
        match self.val.deref() {
            RefValue::String(ref s) => TermContext::Constants(ExprTerm::String(s.clone())),
            RefValue::Number(ref n) => TermContext::Constants(ExprTerm::Number(n.as_f64().unwrap())),
            RefValue::Bool(b) => TermContext::Constants(ExprTerm::Bool(*b)),
            _ => TermContext::Json(match key {
                Some(vk) => Some(vk.clone()),
                _ => None
            }, ValueManager::new(self.val.clone(), false))
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

        ValueManager::new(v, false)
    }
}
