use core::borrow::Borrow;
use std::error::Error;
use std::rc::Rc;
use std::result;

use serde_json::Value;

use jsonpath::parser::*;

use super::term::*;
use super::value_wrapper::*;

trait ArrayIndex {
    fn index(&self, v: &Value) -> usize;

    fn take_value(&self, v: &mut Value) -> Value {
        let idx = self.index(v);
        match v.get_mut(idx) {
            Some(v) => v.take(),
            _ => Value::Null
        }
    }
}

impl ArrayIndex for f64 {
    fn index(&self, v: &Value) -> usize {
        if v.is_array() && self < &0_f64 {
            (v.as_array().unwrap().len() as f64 + self) as usize
        } else {
            *self as usize
        }
    }
}

impl ArrayIndex for isize {
    fn index(&self, v: &Value) -> usize {
        if v.is_array() && self < &0_isize {
            (v.as_array().unwrap().len() as isize + self) as usize
        } else {
            *self as usize
        }
    }
}

impl ArrayIndex for usize {
    fn index(&self, _: &Value) -> usize {
        *self as usize
    }
}

#[derive(Debug)]
pub enum ValueFilterKey {
    Num(usize),
    String(String),
    All,
}

#[derive(Debug)]
pub struct ValueFilter {
    vw: ValueWrapper,
    last_key: Option<ValueFilterKey>,
    filter_mode: bool,
}

impl ValueFilter {
    pub fn new(v: Value, is_leaves: bool, filter_mode: bool) -> Self {
        ValueFilter { vw: ValueWrapper::new(v, is_leaves), last_key: None, filter_mode }
    }

    fn iter_to_value_vec<'a, I: Iterator<Item=&'a mut Value>>(iter: I) -> Vec<Value> {
        iter.map(|v| v.take())
            .filter(|v| !v.is_null())
            .collect()
    }

    fn get_nested_array<F: ArrayIndex>(v: &mut Value, key: F, filter_mode: bool) -> Value {
        if v.is_array() && v.as_array().unwrap().get(key.index(v)).is_some() {
            if filter_mode {
                v.take()
            } else {
                let idx = key.index(v);
                v.get_mut(idx).unwrap().take()
            }
        } else {
            key.take_value(v)
        }
    }

    fn get_nested_object(v: &mut Value, key: &String, filter_mode: bool) -> Value {
        if v.is_object() && v.as_object().unwrap().contains_key(key) {
            if filter_mode {
                v.take()
            } else {
                v.get_mut(key).unwrap().take()
            }
        } else {
            Value::Null
        }
    }

    fn collect_all(key: Option<&String>, v: &Value, buf: &mut Vec<Value>) {
        match v {
            Value::Array(vec) => {
                if key.is_none() {
                    for v in vec {
                        buf.push(v.clone());
                    }
                }
                for i in vec {
                    Self::collect_all(key, &i, buf);
                }
            }
            Value::Object(v) => {
                for (k, v) in v.into_iter() {
                    if match key {
                        Some(map_key) => map_key == k,
                        _ => true
                    } {
                        buf.push(v.clone());
                    }
                }
                for (_, v) in v.into_iter() {
                    Self::collect_all(key, &v, buf);
                }
            }
            _ => {}
        }
    }

    pub fn step_leaves_all(&mut self) -> &ValueWrapper {
        debug!("step_leaves_all");
        let mut buf = Vec::new();
        Self::collect_all(None, &self.vw.get_val(), &mut buf);
        trace!("step_leaves_all - {:?}", buf);
        self.last_key = Some(ValueFilterKey::All);
        self.vw = ValueWrapper::new(Value::Array(buf), true);
        &self.vw
    }

    pub fn step_leaves_str(&mut self, key: &str) -> &ValueWrapper {
        self.step_leaves_string(&key.to_string())
    }

    pub fn step_leaves_string(&mut self, key: &String) -> &ValueWrapper {
        debug!("step_leaves_string");
        let mut buf: Vec<Value> = Vec::new();
        Self::collect_all(Some(key), &self.vw.get_val(), &mut buf);
        trace!("step_leaves_string - {:?}", buf);
        self.last_key = Some(ValueFilterKey::String(key.clone()));
        self.vw = ValueWrapper::new(Value::Array(buf), true);
        &self.vw
    }

    pub fn step_in_all(&mut self) -> &ValueWrapper {
        debug!("step_in_all");

        let vec = match &mut self.vw.get_val_mut() {
            Value::Object(map) => Self::iter_to_value_vec(map.values_mut()),
            Value::Array(list) => Self::iter_to_value_vec(list.iter_mut()),
            Value::Null => Vec::new(),
            other => vec![other.take()]
        };

        self.last_key = Some(ValueFilterKey::All);
        self.vw.replace(Value::Array(vec));
        trace!("step_in_all - {:?}", self.vw.get_val());
        &self.vw
    }

    pub fn step_in_num(&mut self, key: &f64) -> &ValueWrapper {
        debug!("step_in_num");
        trace!("step_in_num - before: {} - {:?}", self.filter_mode, self.vw.get_val());

        let v = if self.vw.is_leaves() {
            let filter_mode = self.filter_mode;
            match &mut self.vw.get_val_mut() {
                Value::Array(v) => {
                    let vec: Vec<Value> = v.iter_mut()
                        .map(|v| Self::get_nested_array(v, *key, filter_mode))
                        .filter(|v| !v.is_null())
                        .collect();
                    Value::Array(vec)
                }
                other => key.take_value(other)
            }
        } else {
            key.take_value(self.vw.get_val_mut())
        };

        self.last_key = Some(ValueFilterKey::Num(key.index(&v)));
        self.vw.replace(v);
        trace!("step_in_num - after: {:?}", self.vw.get_val());
        &self.vw
    }

    pub fn step_in_str(&mut self, key: &str) -> &ValueWrapper {
        self.step_in_string(&key.to_string())
    }

    pub fn step_in_string(&mut self, key: &String) -> &ValueWrapper {
        debug!("step_in_string");
        trace!("step_in_string - before: {},{},{:?}", self.vw.is_leaves(), self.filter_mode, self.vw.get_val());

        let filter_mode = self.filter_mode;
        let is_leaves = self.vw.is_leaves();
        let v = match &mut self.vw.get_val_mut() {
            Value::Array(ref mut vec) if is_leaves => {
                let mut buf = Vec::new();
                for mut item in vec {
                    if let Value::Array(v) = item {
                        let mut ret: Vec<Value> = v.iter_mut()
                            .map(|v| Self::get_nested_object(v, key, filter_mode))
                            .filter(|v| !v.is_null())
                            .collect();
                        buf.append(&mut ret);
                    }
                }

                Value::Array(buf)
            }
            Value::Array(v) if !is_leaves => {
                let vec: Vec<Value> = v.iter_mut()
                    .map(|v| Self::get_nested_object(v, key, filter_mode))
                    .filter(|v| !v.is_null())
                    .collect();

                Value::Array(vec)
            }
            other => {
                match other.get_mut(key) {
                    Some(v) => v.take(),
                    _ => Value::Null
                }
            }
        };

        self.last_key = Some(ValueFilterKey::String(key.clone()));
        self.vw.replace(v);
        trace!("step_in_string - after: {:?}", self.vw.get_val());
        &self.vw
    }
}

pub struct JsonValueFilter {
    json: Rc<Box<Value>>,
    filter_stack: Vec<ValueFilter>,
    token_stack: Vec<ParseToken>,
    term_stack: Vec<TermContext>,
}

impl JsonValueFilter {
    pub fn new(json: &str) -> result::Result<Self, String> {
        let json: Value = serde_json::from_str(json)
            .map_err(|e| e.description().to_string())?;
        Ok(JsonValueFilter {
            json: Rc::new(Box::new(json)),
            filter_stack: Vec::new(),
            token_stack: Vec::new(),
            term_stack: Vec::new(),
        })
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
                    ValueFilter::new(vf.vw.clone_val(), vf.vw.is_leaves(), from_current)
                })
                .and_then(|vf| {
                    Some(self.filter_stack.push(vf))
                });
        } else {
            let v: &Value = self.json.as_ref().borrow();
            self.filter_stack.push({
                ValueFilter::new(v.clone(), false, from_current)
            });
        }
    }

    fn replace_filter_stack(&mut self, v: Value) {
        if self.filter_stack.is_empty() {
            self.filter_stack.push(ValueFilter::new(v, false, false));
        } else {
            match self.filter_stack.last_mut() {
                Some(vf) => {
                    if v.is_null() {
                        vf.vw.replace(v);
                    } else if vf.vw.is_array() {
                        vf.vw.replace(v);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn current_value(&self) -> &Value {
        match self.filter_stack.last() {
            Some(v) => &v.vw.get_val(),
            _ => &Value::Null
        }
    }

    fn token_union<F: ArrayIndex>(&mut self, indices: Vec<F>) {
        self.token_stack.pop();

        match self.filter_stack.last_mut() {
            Some(ref mut vf) if vf.vw.is_array() && vf.vw.is_leaves() => {
                if let Value::Array(mut val) = vf.vw.get_val_mut().take() {
                    let mut ret = Vec::new();
                    for mut v in &mut val {
                        for i in &indices {
                            let v = i.take_value(v);
                            if !v.is_null() {
                                ret.push(v);
                            }
                        }
                    }
                    vf.vw.replace(Value::Array(ret));
                }
            }
            Some(ref mut vf) if vf.vw.is_array() && !vf.vw.is_leaves() => {
                let ret = indices.into_iter()
                    .map(|i| i.take_value(vf.vw.get_val_mut()))
                    .filter(|v| !v.is_null())
                    .collect();
                vf.vw.replace(Value::Array(ret));
            }
            _ => {}
        }
    }

    fn token_range(&mut self, from: Option<isize>, to: Option<isize>) {
        self.token_stack.pop();

        fn _from_to<F: ArrayIndex>(from: Option<F>, to: Option<F>, val: &Value) -> (usize, usize) {
            let from = match from {
                Some(v) => v.index(val),
                _ => 0
            };
            let to = match to {
                Some(v) => v.index(val),
                _ => if let Value::Array(v) = val { v.len() } else { 0 }
            };
            (from, to)
        }

        fn _range(from: usize, to: usize, v: &mut Value) -> Vec<Value> {
            trace!("range - {}:{}", from, to);

            (from..to).into_iter()
                .map(|i| i.take_value(v))
                .filter(|v| !v.is_null())
                .collect()
        }

        match self.filter_stack.last_mut() {
            Some(ref mut vf) if vf.vw.is_array() && vf.vw.is_leaves() => {
                if let Value::Array(mut vec) = vf.vw.get_val_mut().take() {
                    let mut buf = Vec::new();
                    for mut item in &mut vec {
                        let (from, to) = _from_to(from, to, item);
                        let mut v: Vec<Value> = _range(from, to, item);
                        buf.append(&mut v);
                    }
                    vf.vw.replace(Value::Array(buf));
                }
            }
            Some(ref mut vf) if vf.vw.is_array() && !vf.vw.is_leaves() => {
                let (from, to) = _from_to(from, to, vf.vw.get_val());
                let v: Vec<Value> = _range(from, to, vf.vw.get_val_mut());
                vf.vw.replace(Value::Array(v));
            }
            _ => {}
        }
    }

    fn token_key(&mut self, key: String) {
        match self.filter_stack.last_mut() {
            Some(vf) => {
                match self.token_stack.pop() {
                    Some(ParseToken::In) => {
                        vf.step_in_string(&key);
                    }
                    Some(ParseToken::Leaves) => {
                        vf.step_leaves_string(&key);
                    }
                    _ => {}
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
            Some(TermContext::Json(_, mut vw)) => {
                self.replace_filter_stack(vw.get_val_mut().take());
            }
            _ => {
                match self.filter_stack.pop() {
                    Some(mut vf) => {
                        match vf.vw.get_val_mut() {
                            Value::Null | Value::Bool(false) => {
                                self.replace_filter_stack(Value::Null);
                            }
                            other => {
                                self.replace_filter_stack(other.take());
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
            let mut left = left.unwrap();
            let mut right = right.unwrap();

            let tc = match ft {
                FilterToken::Equal => left.eq(&mut right),
                FilterToken::NotEqual => left.ne(&mut right),
                FilterToken::Greater => left.gt(&mut right),
                FilterToken::GreaterOrEqual => left.ge(&mut right),
                FilterToken::Little => left.lt(&mut right),
                FilterToken::LittleOrEqual => left.le(&mut right),
                FilterToken::And => left.and(&mut right),
                FilterToken::Or => left.or(&mut right),
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
            | ParseToken::Leaves
            | ParseToken::Array => {
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
                    self.term_stack.push(TermContext::Json(vf.last_key, vf.vw));
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
