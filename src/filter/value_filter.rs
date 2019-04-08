use std::error::Error;
use std::ops::Deref;
use std::result::Result;

use serde_json::Value;

use filter::term::*;
use filter::value_wrapper::*;
use parser::parser::{FilterToken, NodeVisitor, ParseToken};
use ref_value::model::*;

trait ArrayIndex {
    fn index(&self, v: &RefValueWrapper) -> usize;

    fn take_value(&self, v: &RefValueWrapper) -> RefValueWrapper {
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
            (v.as_array().unwrap().len() as f64 + self) as usize
        } else {
            *self as usize
        }
    }
}

impl ArrayIndex for isize {
    fn index(&self, v: &RefValueWrapper) -> usize {
        if v.is_array() && self < &0_isize {
            (v.as_array().unwrap().len() as isize + self) as usize
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

#[derive(Debug, Clone)]
pub enum ValueFilterKey {
    Num(usize),
    String(String),
    All,
}

#[derive(Debug)]
pub struct ValueFilter {
    val_wrapper: ValueWrapper,
    last_key: Option<ValueFilterKey>,
    filter_mode: bool,
}

impl ValueFilter {
    pub fn new(v: RefValueWrapper, is_leaves: bool, filter_mode: bool) -> Self {
        ValueFilter { val_wrapper: ValueWrapper::new(v, is_leaves), last_key: None, filter_mode }
    }

    fn iter_to_value_vec<'a, I: Iterator<Item=&'a RefValueWrapper>>(iter: I) -> Vec<RefValueWrapper> {
        iter
            .map(|v| v.clone())
            .filter(|v| !v.is_null())
            .collect()
    }

    fn get_nested_array<F: ArrayIndex>(v: &RefValueWrapper, key: F, filter_mode: bool) -> RefValueWrapper {
        if v.is_array() && v.as_array().unwrap().get(key.index(v)).is_some() {
            if filter_mode {
                v.clone()
            } else {
                let idx = key.index(v);
                v.get(idx).unwrap().clone()
            }
        } else {
            key.take_value(v)
        }
    }

    fn get_nested_object(v: &RefValueWrapper, key: &String, filter_mode: bool) -> RefValueWrapper {
        if v.is_object() && v.as_object().unwrap().contains_key(key) {
            if filter_mode {
                v.clone()
            } else {
                v.get(key.clone()).unwrap().clone()
            }
        } else {
            RefValue::Null.into()
        }
    }

    fn collect_all(key: Option<&String>, v: &RefValueWrapper, buf: &mut Vec<RefValueWrapper>) {
        match v.deref() {
            RefValue::Array(vec) => {
                if key.is_none() {
                    for v in vec {
                        buf.push(v.clone());
                    }
                }
                for i in vec {
                    Self::collect_all(key, i, buf);
                }
            }
            RefValue::Object(v) => {
                for (k, v) in v.into_iter() {
                    if match key {
                        Some(map_key) => map_key == k,
                        _ => true
                    } {
                        buf.push(v.clone());
                    }
                }
                for (_, v) in v.into_iter() {
                    Self::collect_all(key, v, buf);
                }
            }
            _ => {}
        }
    }

    pub fn step_leaves_all(&mut self) -> &ValueWrapper {
        debug!("step_leaves_all");
        let mut buf = Vec::new();
        Self::collect_all(None, &self.val_wrapper.get_val(), &mut buf);
        trace!("step_leaves_all - {:?}", buf);
        self.last_key = Some(ValueFilterKey::All);
        self.val_wrapper = ValueWrapper::new(RefValue::Array(buf).into(), true);
        &self.val_wrapper
    }

    pub fn step_leaves_str(&mut self, key: &str) -> &ValueWrapper {
        self.step_leaves_string(&key.to_string())
    }

    pub fn step_leaves_string(&mut self, key: &String) -> &ValueWrapper {
        debug!("step_leaves_string");
        let mut buf = Vec::new();
        Self::collect_all(Some(key), &self.val_wrapper.get_val(), &mut buf);
        trace!("step_leaves_string - {:?}", buf);
        self.last_key = Some(ValueFilterKey::String(key.clone()));
        self.val_wrapper = ValueWrapper::new(RefValue::Array(buf).into(), true);
        &self.val_wrapper
    }

    pub fn step_in_all(&mut self) -> &ValueWrapper {
        debug!("step_in_all");

        let vec = match self.val_wrapper.get_val().deref() {
            RefValue::Object(ref map) => {
                Self::iter_to_value_vec(map.values())
            }
            RefValue::Array(ref list) => {
                Self::iter_to_value_vec(list.iter())
            }
            RefValue::Null => Vec::new(),
            _ => vec![self.val_wrapper.get_val().clone()]
        };

        self.last_key = Some(ValueFilterKey::All);
        self.val_wrapper.replace(RefValue::Array(vec).into());
        trace!("step_in_all - {:?}", self.val_wrapper.get_val());
        &self.val_wrapper
    }

    pub fn step_in_num(&mut self, key: &f64) -> &ValueWrapper {
        debug!("step_in_num");
        trace!("step_in_num - before: leaves {}, filterMode {} - {:?}"
               , self.val_wrapper.is_leaves()
               , self.filter_mode
               , self.val_wrapper.get_val());

        let v = if self.val_wrapper.is_leaves() {
            let filter_mode = self.filter_mode;
            match self.val_wrapper.get_val().deref() {
                RefValue::Array(ref vec) => {
                    let mut ret = Vec::new();
                    for v in vec {
                        let wrapper = Self::get_nested_array(v, *key, filter_mode);
                        if !wrapper.is_null() {
                            ret.push(wrapper.clone());
                        }
                    }
                    RefValue::Array(ret).into()
                }
                _ => key.take_value(&self.val_wrapper.get_val())
            }
        } else {
            key.take_value(&self.val_wrapper.get_val())
        };

        self.last_key = Some(ValueFilterKey::Num(key.index(&v)));
        self.val_wrapper.replace(v);
        trace!("step_in_num - after: {:?}", self.val_wrapper.get_val());
        &self.val_wrapper
    }

    pub fn step_in_str(&mut self, key: &str) -> &ValueWrapper {
        self.step_in_string(&key.to_string())
    }

    pub fn step_in_string(&mut self, key: &String) -> &ValueWrapper {
        debug!("step_in_string");
        trace!("step_in_string - before: {},{},{:?}"
               , self.val_wrapper.is_leaves()
               , self.filter_mode
               , self.val_wrapper.get_val());

        let filter_mode = self.filter_mode;
        let is_leaves = self.val_wrapper.is_leaves();
        let val = match self.val_wrapper.get_val().deref() {
            RefValue::Array(ref vec) if is_leaves => {
                let mut buf = Vec::new();
                for mut v in vec {
                    if v.is_array() {
                        let vec = v.as_array().unwrap();
                        let mut ret = Vec::new();
                        for v in vec {
                            let nested_wrapper = Self::get_nested_object(v, key, filter_mode);
                            if !nested_wrapper.is_null() {
                                ret.push(nested_wrapper.clone());
                            }
                        }
                        buf.append(&mut ret);
                    } else if v.is_object() {
                        let nested_wrapper = Self::get_nested_object(v, key, filter_mode);
                        if !nested_wrapper.is_null() {
                            buf.push(nested_wrapper.clone());
                        }
                    } else {
                        match v.get(key.clone()) {
                            Some(v) => buf.push(v.clone()),
                            _ => {}
                        }
                    }
                }

                RefValue::Array(buf).into()
            }
            RefValue::Array(ref vec) if !is_leaves => {
                let mut ret = Vec::new();
                for v in vec {
                    let wrapper = Self::get_nested_object(v, key, filter_mode);
                    if !wrapper.is_null() {
                        ret.push(wrapper.clone());
                    }
                }
                RefValue::Array(ret).into()
            }
            _ => {
                match self.val_wrapper.get_val().get(key.clone()) {
                    Some(v) => v.clone(),
                    _ => RefValue::Null.into()
                }
            }
        };

        self.last_key = Some(ValueFilterKey::String(key.clone()));
        self.val_wrapper.replace(val);
        trace!("step_in_string - after: {},{},{:?}"
               , self.val_wrapper.is_leaves()
               , self.filter_mode
               , self.val_wrapper.get_val());
        &self.val_wrapper
    }
}

pub struct JsonValueFilter {
    json: RefValueWrapper,
    filter_stack: Vec<ValueFilter>,
    token_stack: Vec<ParseToken>,
    term_stack: Vec<TermContext>,
}

impl JsonValueFilter {
    pub fn new(json: &str) -> Result<Self, String> {
        let json: RefValue = serde_json::from_str(json)
            .map_err(|e| e.description().to_string())?;
        Ok(JsonValueFilter::new_from_value(json.into()))
    }

    pub fn new_from_value(json: RefValueWrapper) -> Self {
        JsonValueFilter {
            json,
            filter_stack: Vec::new(),
            token_stack: Vec::new(),
            term_stack: Vec::new(),
        }
    }

    fn is_peek_token_array(&self) -> bool {
        if let Some(ParseToken::Array) = self.token_stack.last() {
            true
        } else {
            false
        }
    }

    fn push_value_filter(&mut self, from_current: bool) {
        if from_current {
            self.filter_stack.last()
                .map(|vf| {
                    ValueFilter::new(vf.val_wrapper.get_val().clone(), vf.val_wrapper.is_leaves(), from_current)
                })
                .and_then(|vf| {
                    Some(self.filter_stack.push(vf))
                });
        } else {
            self.filter_stack.push({
                ValueFilter::new(self.json.clone(), false, from_current)
            });
        }
    }

    fn replace_filter_stack(&mut self, v: RefValueWrapper, is_leaves: bool) {
        if self.filter_stack.is_empty() {
            self.filter_stack.push(ValueFilter::new(v, is_leaves, false));
        } else {
            match self.filter_stack.last_mut() {
                Some(vf) => {
                    vf.val_wrapper.set_leaves(is_leaves);
                    if v.is_null() {
                        vf.val_wrapper.replace(v);
                    } else if v.is_array() && v.as_array().unwrap().is_empty() {
                        vf.val_wrapper.replace(RefValue::Null.into());
                    } else if vf.val_wrapper.is_array() {
                        vf.val_wrapper.replace(v);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn into_value(&self) -> Value {
        match self.filter_stack.last() {
            Some(v) => v.val_wrapper.into_value(),
            _ => Value::Null
        }
    }

    pub fn take_value(&mut self) -> RefValueWrapper {
        match self.filter_stack.last_mut() {
            Some(v) => v.val_wrapper.get_val().clone(),
            _ => RefValue::Null.into()
        }
    }

    fn token_union<F: ArrayIndex>(&mut self, indices: Vec<F>) {
        self.token_stack.pop();

        match self.filter_stack.last_mut() {
            Some(ref mut vf) if vf.val_wrapper.is_array() && vf.val_wrapper.is_leaves() => {
                let mut ret = Vec::new();
                if let RefValue::Array(val) = vf.val_wrapper.get_val().deref() {
                    for mut v in val {
                        for i in &indices {
                            let v = i.take_value(v);
                            if !v.is_null() {
                                ret.push(v.clone());
                            }
                        }
                    }
                }
                vf.val_wrapper.replace(RefValue::Array(ret).into());
            }
            Some(ref mut vf) if vf.val_wrapper.is_array() && !vf.val_wrapper.is_leaves() => {
                let mut ret = Vec::new();
                for i in indices {
                    let wrapper = i.take_value(&vf.val_wrapper.get_val());
                    if !wrapper.is_null() {
                        ret.push(wrapper.clone());
                    }
                }
                vf.val_wrapper.replace(RefValue::Array(ret).into());
            }
            _ => {}
        }
    }

    fn token_range(&mut self, from: Option<isize>, to: Option<isize>) {
        self.token_stack.pop();

        fn _from_to<F: ArrayIndex>(from: Option<F>, to: Option<F>, val: &RefValueWrapper) -> (usize, usize) {
            let from = match from {
                Some(v) => v.index(val),
                _ => 0
            };
            let to = match to {
                Some(v) => v.index(val),
                _ => {
                    if let RefValue::Array(v) = val.deref() {
                        v.len()
                    } else {
                        0
                    }
                }
            };
            (from, to)
        }

        fn _range(from: usize, to: usize, v: &RefValueWrapper) -> Vec<RefValueWrapper> {
            trace!("range - {}:{}", from, to);

            (from..to).into_iter()
                .map(|i| i.take_value(v))
                .filter(|v| !v.is_null())
                .map(|v| v.clone())
                .collect()
        }

        match self.filter_stack.last_mut() {
            Some(ref mut vf) if vf.val_wrapper.is_array() && vf.val_wrapper.is_leaves() => {
                let mut buf = Vec::new();
                if let RefValue::Array(vec) = vf.val_wrapper.get_val().deref() {
                    for mut v in vec {
                        let (from, to) = _from_to(from, to, v);
                        let mut v: Vec<RefValueWrapper> = _range(from, to, v);
                        buf.append(&mut v);
                    }
                }
                vf.val_wrapper.replace(RefValue::Array(buf).into());
            }
            Some(ref mut vf) if vf.val_wrapper.is_array() && !vf.val_wrapper.is_leaves() => {
                let (from, to) = _from_to(from, to, &vf.val_wrapper.get_val());
                let vec: Vec<RefValueWrapper> = _range(from, to, vf.val_wrapper.get_val());
                vf.val_wrapper.replace(RefValue::Array(vec).into());
            }
            _ => {}
        }
    }

    fn token_key(&mut self, key: String) {
        match self.filter_stack.last_mut() {
            Some(vf) => {
                match self.token_stack.pop() {
                    Some(ParseToken::In) | Some(ParseToken::Array) => {
                        vf.step_in_string(&key);
                    }
                    Some(ParseToken::Leaves) => {
                        vf.step_leaves_string(&key);
                    }
                    _ => {
                        self.term_stack.push(TermContext::Constants(ExprTerm::String(key)));
                    }
                }
            }
            _ => {}
        }
    }

    fn token_all(&mut self) {
        match self.filter_stack.last_mut() {
            Some(vf) => {
                match self.token_stack.pop() {
                    Some(ParseToken::In) => {
                        vf.step_in_all();
                    }
                    Some(ParseToken::Leaves) => {
                        vf.step_leaves_all();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn token_end_array(&mut self) {
        trace!("array_eof - term_stack: {:?}", self.term_stack);
        trace!("array_eof - filter_stack: {:?}", self.filter_stack);

        match self.term_stack.pop() {
            Some(TermContext::Constants(ExprTerm::Number(v))) => {
                match self.filter_stack.last_mut() {
                    Some(vf) => {
                        vf.step_in_num(&v);
                    }
                    _ => {}
                }
            }
            Some(TermContext::Constants(ExprTerm::Bool(false))) => {
                self.replace_filter_stack(RefValue::Null.into(), false);
            }
            Some(TermContext::Json(_, vw)) => {
                self.replace_filter_stack(vw.get_val().clone(), vw.is_leaves());
            }
            _ => {
                match self.filter_stack.pop() {
                    Some(mut vf) => {
                        let is_leaves = vf.val_wrapper.is_leaves();
                        match vf.val_wrapper.get_val().deref() {
                            RefValue::Null | RefValue::Bool(false) => {
                                self.replace_filter_stack(RefValue::Null.into(), is_leaves);
                            }
                            _ => {
                                self.replace_filter_stack(vf.val_wrapper.get_val().clone(), is_leaves);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn token_op(&mut self, ft: &FilterToken) {
        let right = self.term_stack.pop();
        let left = self.term_stack.pop();

        trace!("left {:?}", left);
        trace!("right {:?}", right);

        if left.is_some() && right.is_some() {
            let left = left.unwrap();
            let right = right.unwrap();

            let tc = match ft {
                FilterToken::Equal => left.eq(&right),
                FilterToken::NotEqual => left.ne(&right),
                FilterToken::Greater => left.gt(&right),
                FilterToken::GreaterOrEqual => left.ge(&right),
                FilterToken::Little => left.lt(&right),
                FilterToken::LittleOrEqual => left.le(&right),
                FilterToken::And => left.and(&right),
                FilterToken::Or => left.or(&right),
            };
            self.term_stack.push(tc);
        }

        trace!("filter - {:?}", self.term_stack)
    }
}

impl NodeVisitor for JsonValueFilter {
    fn visit_token(&mut self, token: ParseToken) {
        debug!("visit_token: {:?}", token);

        match token {
            ParseToken::Absolute
            | ParseToken::Relative => {
                if self.is_peek_token_array() {
                    self.token_stack.pop();
                }
                self.push_value_filter(ParseToken::Relative == token);
            }
            ParseToken::In
            | ParseToken::Leaves => {
                self.token_stack.push(token);
            }
            ParseToken::Array => {
                if let Some(ParseToken::Leaves) = self.token_stack.last() {
                    self.token_all();
                }
                self.token_stack.push(token);
            }
            ParseToken::ArrayEof => {
                self.token_end_array();
            }
            ParseToken::All => {
                self.token_all();
            }
            ParseToken::Key(key) => {
                self.token_key(key);
            }
            ParseToken::Filter(ref ft) => {
                self.token_op(ft);
            }
            ParseToken::Number(v) => {
                self.term_stack.push(TermContext::Constants(ExprTerm::Number(v)))
            }
            ParseToken::Range(from, to) => {
                self.token_range(from, to);
            }
            ParseToken::Union(v) => {
                self.token_union(v);
            }
            ParseToken::Eof => {
                debug!("visit_token eof");
            }
        }
    }

    fn end_term(&mut self) {
        debug!("end_term");

        if let Some(ParseToken::Array) = self.token_stack.last() {
            self.token_stack.pop();
        }

        trace!("end_term - term_stack {:?}", self.term_stack);
        trace!("end_term - token_stack {:?}", self.token_stack);
        trace!("end_term - filter_stack {:?}", self.filter_stack);

        if self.token_stack.is_empty() && self.filter_stack.len() > 1 {
            match self.filter_stack.pop() {
                Some(vf) => {
                    self.term_stack.push(TermContext::Json(vf.last_key, vf.val_wrapper));
                }
                _ => {}
            }
        }

        if match self.token_stack.last() {
            Some(ParseToken::Key(_))
            | Some(ParseToken::Number(_)) => true,
            _ => false
        } {
            match self.token_stack.pop() {
                Some(ParseToken::Key(ref v)) if v.eq_ignore_ascii_case("true") => {
                    self.term_stack.push(TermContext::Constants(ExprTerm::Bool(true)))
                }
                Some(ParseToken::Key(ref v)) if v.eq_ignore_ascii_case("false") => {
                    self.term_stack.push(TermContext::Constants(ExprTerm::Bool(false)))
                }
                Some(ParseToken::Key(v)) => {
                    self.term_stack.push(TermContext::Constants(ExprTerm::String(v)))
                }
                Some(ParseToken::Number(v)) => {
                    self.term_stack.push(TermContext::Constants(ExprTerm::Number(v)))
                }
                _ => {}
            }
        }
    }
}
