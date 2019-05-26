use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;

use serde_json::Value;

use filter::term::*;
use filter::value_manager::*;
use parser::parser::{FilterToken, NodeVisitor, ParseToken};
use ref_value::model::*;
use select::path_map::PathMap;

#[derive(Debug, Clone)]
pub enum ValueFilterKey {
    Num(usize),
    String(String),
    All,
}

fn collect_all(key: Option<&String>, v: &RefValueWrapper, buf: &mut Vec<RefValueWrapper>) {
    match v.deref() {
        RefValue::Array(vec) => {
            if key.is_none() {
                for v in vec.iter() {
                    buf.push(v.clone());
                }
            }

            for v in vec {
                collect_all(key, v, buf);
            }
        }
        RefValue::Object(map) => {
            if let Some(k) = key {
                if let Some(val) = map.get(k) {
                    buf.push(val.clone());
                }
            } else {
                let mut c = map.values().map(|v| v.clone()).collect();
                buf.append(&mut c);
            }
            for (_, v) in map {
                collect_all(key, v, buf);
            }
        }
        _ => {}
    }
}

#[derive(Debug)]
pub struct ValueFilter {
    value_mgr: ValueManager,
    last_key: Option<ValueFilterKey>,
    is_relative: bool,
    path_map: Arc<RefCell<PathMap>>,
}

impl ValueFilter {
    pub fn new(v: RefValueWrapper, is_leaves: bool, is_relative: bool, path_map: Arc<RefCell<PathMap>>) -> Self {
        ValueFilter {
            value_mgr: ValueManager::new(v, is_leaves, path_map.clone()),
            last_key: None,
            is_relative,
            path_map,
        }
    }

    fn step_leaves(&mut self, key: Option<&String>) {
        let mut buf = Vec::new();
        collect_all(key, &self.value_mgr.get_val(), &mut buf);
        trace!("step_leaves - {:?}", buf);
        self.value_mgr = ValueManager::new(RefValue::Array(buf).into(), true, self.path_map.clone());
    }

    pub fn step_leaves_all(&mut self) -> &ValueManager {
        debug!("step_leaves_all");
        self.step_leaves(None);
        self.last_key = Some(ValueFilterKey::All);
        &self.value_mgr
    }

    pub fn step_leaves_str(&mut self, key: &str) -> &ValueManager {
        self.step_leaves_string(&key.to_string())
    }

    pub fn step_leaves_string(&mut self, key: &String) -> &ValueManager {
        debug!("step_leaves_string");
        self.step_leaves(Some(key));
        self.last_key = Some(ValueFilterKey::String(key.clone()));
        &self.value_mgr
    }

    pub fn step_in_all(&mut self) -> &ValueManager {
        debug!("step_in_all");
        self.last_key = Some(ValueFilterKey::All);
        self.value_mgr.replace(self.value_mgr.get_as_array());
        trace!("step_in_all - {:?}", self.value_mgr.get_val());
        &self.value_mgr
    }

    pub fn step_in_num(&mut self, key: &f64) -> &ValueManager {
        debug!("step_in_num");
        trace!("step_in_num - before: leaves {}, filterMode {} - {:?}"
               , self.value_mgr.is_leaves()
               , self.is_relative
               , self.value_mgr.get_val());


        self.last_key = Some(ValueFilterKey::Num(self.value_mgr.get_index(*key)));
        let v = self.value_mgr.get_with_num(key, self.is_relative);
        self.value_mgr.replace(v);
        trace!("step_in_num - after: {:?}", self.value_mgr.get_val());
        &self.value_mgr
    }

    pub fn step_in_str(&mut self, key: &str) -> &ValueManager {
        self.step_in_string(&key.to_string())
    }

    pub fn step_in_string(&mut self, key: &String) -> &ValueManager {
        debug!("step_in_string");
        trace!("step_in_string - before: {},{},{:?}"
               , self.value_mgr.is_leaves()
               , self.is_relative
               , self.value_mgr.get_val());

        self.last_key = Some(ValueFilterKey::String(key.clone()));
        self.value_mgr.replace(self.value_mgr.get_with_str(key, self.is_relative));
        trace!("step_in_string - after: {},{},{:?}"
               , self.value_mgr.is_leaves()
               , self.is_relative
               , self.value_mgr.get_val());
        &self.value_mgr
    }
}

pub struct JsonValueFilter {
    json: RefValueWrapper,
    path_map: Arc<RefCell<PathMap>>,
    filter_stack: Vec<ValueFilter>,
    token_stack: Vec<ParseToken>,
    term_stack: Vec<TermContext>,
}

impl JsonValueFilter {
    pub fn new(json: RefValueWrapper, path_map: Arc<RefCell<PathMap>>) -> Self {
        JsonValueFilter {
            json,
            path_map,
            filter_stack: Vec::new(),
            token_stack: Vec::new(),
            term_stack: Vec::new(),
        }
    }

    fn is_peek_token_array(&self) -> bool {
        if let Some(ParseToken::Array) = self.token_stack.last() { true } else { false }
    }

    fn create_new_filter(&mut self, is_relative: bool) {
        if is_relative {
            self.filter_stack.last()
                .map(|vf| {
                    ValueFilter::new(vf.value_mgr.get_val().clone(),
                                     vf.value_mgr.is_leaves(),
                                     is_relative,
                                     self.path_map.clone(),
                    )
                })
                .and_then(|vf| {
                    Some(self.filter_stack.push(vf))
                });
        } else {
            let vf = ValueFilter::new(
                self.json.clone(),
                false,
                is_relative,
                self.path_map.clone(),
            );
            self.filter_stack.push(vf);
        }
    }

    fn append_to_current_filter(&mut self, v: RefValueWrapper, is_leaves: bool) {
        if self.filter_stack.is_empty() {
            self.filter_stack.push(ValueFilter::new(
                v,
                is_leaves,
                false,
                self.path_map.clone(),
            ));
            return;
        }

        match self.filter_stack.last_mut() {
            Some(vf) => {
                vf.value_mgr.set_leaves(is_leaves);
                if v.is_null() || v.is_empty() {
                    vf.value_mgr.replace(RefValue::Null.into());
                } else if vf.value_mgr.is_array() {
                    vf.value_mgr.replace(v);
                } else {
                    // ignore. the current filter context is object that include v: RefValueWrapper as a child.
                }
            }
            _ => {}
        }
    }

    pub fn into_value(&self) -> Value {
        match self.filter_stack.last() {
            Some(v) => v.value_mgr.into_value(),
            _ => Value::Null
        }
    }

    #[deprecated(since = "0.1.14", note = "Please use the clone_value function instead")]
    pub fn take_value(&mut self) -> RefValueWrapper {
        self.clone_value()
    }

    pub fn clone_value(&mut self) -> RefValueWrapper {
        match self.filter_stack.last() {
            Some(v) => v.value_mgr.get_val().clone(),
            _ => RefValue::Null.into()
        }
    }

    fn token_union(&mut self, indices: Vec<isize>) {
        self.token_stack.pop();

        match self.filter_stack.last_mut() {
            Some(vf) => {
                if let Some(vec) = vf.value_mgr.pick_with_nums(indices) {
                    vf.value_mgr.replace(vec);
                }
            }
            _ => {}
        }
    }

    fn token_range(&mut self, from: Option<isize>, to: Option<isize>) {
        self.token_stack.pop();

        match self.filter_stack.last_mut() {
            Some(ref mut vf) => {
                if let Some(vec) = vf.value_mgr.range_with(from, to) {
                    vf.value_mgr.replace(vec);
                }
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
                self.append_to_current_filter(RefValue::Null.into(), false);
            }
            Some(TermContext::Json(_, vw)) => {
                self.append_to_current_filter(vw.get_val().clone(), vw.is_leaves());
            }
            _ => {

                //
                // None, TermContext::Constants(ExprTerm::Bool(true))
                //

                match self.filter_stack.pop() {
                    Some(vf) => {
                        match vf.value_mgr.get_val().deref() {
                            RefValue::Null | RefValue::Bool(false) => {
                                self.append_to_current_filter(RefValue::Null.into(), vf.value_mgr.is_leaves());
                            }
                            _ => {
                                self.append_to_current_filter(vf.value_mgr.get_val().clone(), vf.value_mgr.is_leaves());
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
                FilterToken::And => left.and(&mut right, self.path_map.clone()),
                FilterToken::Or => left.or(&mut right, self.path_map.clone()),
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
                self.create_new_filter(ParseToken::Relative == token);
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
                    self.term_stack.push(TermContext::Json(vf.last_key, vf.value_mgr));
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
