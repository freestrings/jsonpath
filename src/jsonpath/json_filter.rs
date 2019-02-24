use core::borrow::Borrow;
use std::error::Error;
use std::rc::Rc;
use std::result;

use serde_json::Value;

use jsonpath::parser::{
    FilterToken,
    NodeVisitor,
    ParseToken,
};

use std::collections::HashMap;

enum CmpType {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

enum CmpCondType {
    And,
    Or,
}

trait PrivCmp {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool;

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool;

    fn cmp_string(&self, v1: &String, v2: &String) -> bool;
}

trait IntoType {
    fn into_type(&self) -> CmpType;
}

struct CmpEq;

impl PrivCmp for CmpEq {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 == v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 == v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 == v2
    }
}

impl IntoType for CmpEq {
    fn into_type(&self) -> CmpType {
        CmpType::Eq
    }
}

struct CmpNe;

impl PrivCmp for CmpNe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 != v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 != v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 != v2
    }
}

impl IntoType for CmpNe {
    fn into_type(&self) -> CmpType {
        CmpType::Ne
    }
}

struct CmpGt;

impl PrivCmp for CmpGt {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 > v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 > v2
    }
}

impl IntoType for CmpGt {
    fn into_type(&self) -> CmpType {
        CmpType::Gt
    }
}

struct CmpGe;

impl PrivCmp for CmpGe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 >= v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 >= v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 >= v2
    }
}

impl IntoType for CmpGe {
    fn into_type(&self) -> CmpType {
        CmpType::Ge
    }
}

struct CmpLt;

impl PrivCmp for CmpLt {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 < v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 < v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 < v2
    }
}

impl IntoType for CmpLt {
    fn into_type(&self) -> CmpType {
        CmpType::Lt
    }
}

struct CmpLe;

impl PrivCmp for CmpLe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 <= v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 <= v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 <= v2
    }
}

impl IntoType for CmpLe {
    fn into_type(&self) -> CmpType {
        CmpType::Le
    }
}

struct CmpAnd;

impl PrivCmp for CmpAnd {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        *v1 && *v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > &0_f64 && v2 > &0_f64
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        !v1.is_empty() && !v2.is_empty()
    }
}

struct CmpOr;

impl PrivCmp for CmpOr {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        *v1 || *v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > &0_f64 || v2 > &0_f64
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        !v1.is_empty() || !v2.is_empty()
    }
}

#[derive(Debug)]
enum ExprTerm {
    String(String),
    Number(f64),
    Bool(bool),
}

impl ExprTerm {
    fn cmp<F: PrivCmp>(&self, other: &ExprTerm, cmp_fn: F, default: bool) -> bool {
        match self {
            ExprTerm::Bool(v1) => match other {
                ExprTerm::Bool(v2) => cmp_fn.cmp_bool(v1, v2),
                _ => default
            }
            ExprTerm::Number(v1) => match other {
                ExprTerm::Number(v2) => cmp_fn.cmp_f64(v1, v2),
                _ => default
            }
            ExprTerm::String(v1) => match other {
                ExprTerm::String(v2) => cmp_fn.cmp_string(v1, v2),
                _ => default
            }
        }
    }
}

#[derive(Debug)]
enum TermContext {
    Constants(ExprTerm),
    Json(Option<ValueFilterKey>, ValueWrapper),
}

impl TermContext {
    fn cmp<F: PrivCmp + IntoType>(&mut self, other: &mut TermContext, cmp_fn: F, default: bool) -> TermContext {
        match self {
            TermContext::Constants(et) => {
                match other {
                    TermContext::Constants(oet) => {
                        TermContext::Constants(ExprTerm::Bool(et.cmp(oet, cmp_fn, default)))
                    }
                    TermContext::Json(key, v) => {
                        TermContext::Json(None, v.take_with(key, et, cmp_fn))
                    }
                }
            }
            TermContext::Json(key, v) => {
                match other {
                    TermContext::Json(_, ov) => {
                        v.cmp(ov, cmp_fn.into_type())
                    }
                    TermContext::Constants(et) => {
                        TermContext::Json(None, v.take_with(key, et, cmp_fn))
                    }
                }
            }
        }
    }

    fn cmp_cond(&mut self, other: &mut TermContext, cmp_cond_type: CmpCondType) -> TermContext {
        match self {
            TermContext::Constants(et) => {
                match other {
                    TermContext::Constants(oet) => {
                        match cmp_cond_type {
                            CmpCondType::Or => {
                                TermContext::Constants(ExprTerm::Bool(et.cmp(oet, CmpOr, false)))
                            }
                            CmpCondType::And => {
                                TermContext::Constants(ExprTerm::Bool(et.cmp(oet, CmpAnd, false)))
                            }
                        }
                    }
                    TermContext::Json(_, v) => {
                        TermContext::Json(None, ValueWrapper::new(v.clone_val()))
                    }
                }
            }
            TermContext::Json(_, v) => {
                match other {
                    TermContext::Json(_, ov) => {
                        match cmp_cond_type {
                            CmpCondType::Or => TermContext::Json(None, v.union(ov)),
                            CmpCondType::And => TermContext::Json(None, v.intersect(ov)),
                        }
                    }
                    _ => {
                        TermContext::Json(None, ValueWrapper::new(v.clone_val()))
                    }
                }
            }
        }
    }

    fn eq(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp(other, CmpEq, false)
    }

    fn ne(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp(other, CmpNe, true)
    }

    fn gt(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp(other, CmpGt, false)
    }

    fn ge(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp(other, CmpGe, false)
    }

    fn lt(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp(other, CmpLt, false)
    }

    fn le(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp(other, CmpLe, false)
    }

    fn and(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp_cond(other, CmpCondType::And)
    }

    fn or(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp_cond(other, CmpCondType::Or)
    }
}

#[derive(Debug)]
enum ValueFilterKey {
    Num(usize),
    String(String),
    All,
}

#[derive(Debug)]
struct ValueFilter {
    vw: ValueWrapper,
    last_key: Option<ValueFilterKey>,
}

impl ValueFilter {
    fn new(v: Value) -> Self {
        ValueFilter { vw: ValueWrapper::new(v), last_key: None }
    }

    fn iter_to_value_vec<'a, I: Iterator<Item=&'a mut Value>>(iter: I) -> Vec<Value> {
        iter.map(|v| v.take())
            .filter(|v| !v.is_null())
            .collect()
    }

    fn step_leaves_all(&mut self) -> &ValueWrapper {
        debug!("step_leaves_all");

        let mut vw = ValueWrapper::new(Value::Null);
        loop {
            vw.push(self.step_in_all().clone_val());
            if let Value::Null = self.vw.val {
                break;
            }
        }
        self.last_key = Some(ValueFilterKey::All);
        self.vw = vw;
        &self.vw
    }

    fn step_leaves(&mut self, key: &String) -> &ValueWrapper {
        debug!("step_leaves");

        let mut vw = ValueWrapper::new(Value::Null);
        loop {
            vw.push(self.step_in_string(key).clone_val());
            if let Value::Null = self.vw.val() {
                break;
            }
        }
        self.last_key = Some(ValueFilterKey::String(key.clone()));
        self.vw = vw;
        &self.vw
    }

    fn step_in_all(&mut self) -> &ValueWrapper {
        debug!("step_in_all");

        let vec = match &mut self.vw.val {
            Value::Object(map) => Self::iter_to_value_vec(map.values_mut()),
            Value::Array(list) => Self::iter_to_value_vec(list.iter_mut()),
            Value::Null => Vec::new(),
            other => vec![other.take()]
        };

        self.last_key = Some(ValueFilterKey::All);
        self.vw.replace(Value::Array(vec));
        trace!("step_in_all - {:?}", self.vw.val);
        &self.vw
    }

    fn step_in_num(&mut self, key: &usize) -> &ValueWrapper {
        debug!("step_in_num");

        trace!("step_in_num - before: {:?}", self.vw.val);
        let v = match self.vw.val.get_mut(&key) {
            Some(value) => value.take(),
            _ => Value::Null
        };

        self.last_key = Some(ValueFilterKey::Num(key.clone()));
        self.vw.replace(v);
        trace!("step_in_num - after: {:?}", self.vw.val);
        &self.vw
    }

    fn step_in_str(&mut self, key: &str) -> &ValueWrapper {
        self.step_in_string(&key.to_string())
    }

    fn step_in_string(&mut self, key: &String) -> &ValueWrapper {
        debug!("step_in_string");

        trace!("step_in_string - before: {:?}", self.vw.val);
        let v = match &mut self.vw.val {
            Value::Array(v) => {
                let vec: Vec<Value> = v.iter_mut()
                    .map(|v| {
                        if v.is_object() && v.as_object().unwrap().contains_key(key) {
                            v.take()
                        } else {
                            Value::Null
                        }
                    })
                    .filter(|v| !v.is_null())
                    .collect();
                Value::Array(vec)
            }
            other => match other.get_mut(key) {
                Some(v) => v.take(),
                _ => Value::Null
            }
        };

        self.last_key = Some(ValueFilterKey::String(key.clone()));
        self.vw.replace(v);
        trace!("step_in_string - after: {:?}", self.vw.val);
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
                .map(|vf| ValueFilter::new(vf.vw.clone_val()))
                .and_then(|vf| Some(self.filter_stack.push(vf)));
        } else {
            let v: &Value = self.json.as_ref().borrow();
            self.filter_stack.push(ValueFilter::new(v.clone()));
        }
    }

    fn replace_filter_stack(&mut self, v: Value) {
        if self.filter_stack.is_empty() {
            self.filter_stack.push(ValueFilter::new(v));
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

    fn current_value(&self) -> &Value {
        match self.filter_stack.last() {
            Some(v) => &v.vw.val(),
            _ => &Value::Null
        }
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
                trace!("array_eof - term_stack: {:?}", self.term_stack);
                trace!("array_eof - filter_stack: {:?}", self.filter_stack);

                match self.term_stack.pop() {
                    Some(TermContext::Constants(ExprTerm::Number(v))) => {
                        let v = v as usize;
                        match self.filter_stack.last_mut() {
                            Some(vf) => {
                                vf.step_in_num(&v);
                            }
                            _ => {}
                        }
                    }
                    Some(TermContext::Json(_, mut vw)) => {
                        self.replace_filter_stack(vw.val.take())
                    }
                    _ => {
                        match self.filter_stack.pop() {
                            Some(vf) => {
                                match vf.vw.val {
                                    Value::Null | Value::Bool(false) => {
                                        self.replace_filter_stack(Value::Null)
                                    }
                                    other => {
                                        self.replace_filter_stack(other)
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ParseToken::All => {
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
            ParseToken::Key(key) => {
                match self.filter_stack.last_mut() {
                    Some(vf) => {
                        match self.token_stack.pop() {
                            Some(ParseToken::In) => {
                                vf.step_in_string(&key);
                            }
                            Some(ParseToken::Leaves) => {
                                vf.step_leaves(&key);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            ParseToken::Filter(ref ft) => {
                let right = self.term_stack.pop();
                let left = self.term_stack.pop();

                trace!("left {:?}", left);
                trace!("right {:?}", right);

                if let Some(mut left) = left {
                    if let Some(mut right) = right {
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
                }

                trace!("filter - {:?}", self.term_stack)
            }

            ParseToken::Number(v) => {
                self.term_stack.push(TermContext::Constants(ExprTerm::Number(v)))
            }

            ParseToken::Range(from, to) => {
                self.token_stack.pop();
                match self.filter_stack.last_mut() {
                    Some(vf) => {
                        if !vf.vw.is_array() {
                            return;
                        }

                        let len = if let Some(v) = vf.vw.val.as_array() { v.len() } else { 0 };
                        let from = match from {
                            Some(v) => if v < 0 { 0 } else { v as usize },
                            _ => 0
                        };
                        let to = match to {
                            Some(v) => if v < 0 { len - v.abs() as usize } else { v as usize }
                            _ => len
                        };

                        trace!("range - {}:{}", from, to);

                        let v: Vec<Value> = (from..to).into_iter()
                            .map(|i| match vf.vw.val.get_mut(i) {
                                Some(v) => v.take(),
                                _ => Value::Null
                            })
                            .filter(|v| !v.is_null())
                            .collect();

                        vf.vw.replace(Value::Array(v));
                    }
                    _ => {}
                }
            }

            ParseToken::Union(v) => {
                self.token_stack.pop();
                match self.filter_stack.last_mut() {
                    Some(vf) => {
                        if !vf.vw.is_array() {
                            return;
                        }

                        let v: Vec<Value> = v.into_iter()
                            .map(|i| match vf.vw.val.get_mut(i as usize) {
                                Some(v) => v.take(),
                                _ => Value::Null
                            })
                            .filter(|v| !v.is_null())
                            .collect();

                        trace!("union - {:?}", v);

                        vf.vw.replace(Value::Array(v));
                    }
                    _ => {}
                }
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

#[derive(Debug)]
struct ValueWrapper {
    val: Value,
}

impl ValueWrapper {
    fn new(v: Value) -> Self {
        ValueWrapper { val: v }
    }

    fn cmp(&mut self, other: &mut ValueWrapper, cmp_type: CmpType) -> TermContext {
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

    fn cmp_with_term<F: PrivCmp>(val: &Value, et: &ExprTerm, cmp_fn: &F, default: bool) -> bool {
        match val {
            Value::Bool(ref v1) => {
                match et {
                    ExprTerm::Bool(v2) => cmp_fn.cmp_bool(v1, v2),
                    _ => default
                }
            }
            Value::Number(ref v1) => match v1.as_f64() {
                Some(ref v1) => {
                    match et {
                        ExprTerm::Number(v2) => cmp_fn.cmp_f64(v1, v2),
                        _ => default
                    }
                }
                _ => default
            },
            Value::String(ref v1) => {
                match et {
                    ExprTerm::String(v2) => cmp_fn.cmp_string(v1, v2),
                    _ => default
                }
            }
            _ => default
        }
    }

    fn take_object_in_array<F: PrivCmp>(&mut self, key: &String, et: &ExprTerm, cmp: &F) -> Option<Self> {
        match self.val.take() {
            Value::Array(mut vec) => {
                let mut ret: Vec<Value> = vec.iter_mut()
                    .filter(|v| {
                        match &v {
                            Value::Object(map) => {
                                match map.get(key) {
                                    Some(vv) => Self::cmp_with_term(vv, et, cmp, false),
                                    _ => false
                                }
                            }
                            _ => false
                        }
                    })
                    .map(|v| v.take())
                    .collect();
                Some(ValueWrapper::new(Value::Array(ret)))
            }
            _ => None
        }
    }

    fn take_with_key_type<F: PrivCmp>(&mut self, key: &Option<ValueFilterKey>, et: &ExprTerm, cmp: &F) -> Option<Self> {
        match key {
            Some(ValueFilterKey::String(key)) => {
                self.take_object_in_array(key, et, cmp)
            }
            _ => None
        }
    }

    fn take_with<F: PrivCmp>(&mut self, key: &Option<ValueFilterKey>, et: &ExprTerm, cmp: F) -> Self {
        match self.take_with_key_type(key, et, &cmp) {
            Some(vw) => vw,
            _ => {
                match self.val.take() {
                    Value::Array(mut vec) => {
                        let mut ret = vec.iter_mut()
                            .filter(|v| Self::cmp_with_term(&v, et, &cmp, false))
                            .map(|v| v.take())
                            .collect();
                        ValueWrapper::new(Value::Array(ret))
                    }
                    other => {
                        if Self::cmp_with_term(&other, et, &cmp, false) {
                            ValueWrapper::new(other)
                        } else {
                            ValueWrapper::new(Value::Null)
                        }
                    }
                }
            }
        }
    }

    fn replace(&mut self, val: Value) {
        let is_null = match &val {
            Value::Array(v) => if v.is_empty() { true } else { false },
            Value::Object(m) => if m.is_empty() { true } else { false },
            _ => val.is_null()
        };
        self.val = if is_null { Value::Null } else { val };
    }

    fn push(&mut self, v: Value) {
        if let Value::Array(values) = &mut self.val {
            values.push(v);
            return;
        }

        let data = self.val.take();
        if data.is_null() {
            self.val = v;
        } else {
            let mut values = Vec::new();
            values.push(data);
            values.push(v);
            self.val = Value::Array(values);
        }
    }

    fn clone_val(&self) -> Value {
        self.val.clone()
    }

    fn is_array(&self) -> bool {
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

    fn into_map(&mut self) -> HashMap<String, Value> {
        let mut map: HashMap<String, Value> = HashMap::new();
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

    fn except(&mut self, other: &mut Self) -> Self {
        let map = self.into_map();
        let mut ret: HashMap<String, Value> = HashMap::new();
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
        ValueWrapper::new(v)
    }

    fn intersect(&mut self, other: &mut Self) -> Self {
        let map = self.into_map();
        let mut ret: HashMap<String, Value> = HashMap::new();
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
        ValueWrapper::new(v)
    }

    fn union(&mut self, other: &mut Self) -> Self {
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

        let mut vw = ValueWrapper::new(Value::Null);
        let list: Vec<Value> = map.values_mut().into_iter().map(|val| val.take()).collect();
        vw.replace(Value::Array(list));
        vw
    }

    fn val(&self) -> &Value {
        &self.val
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::sync::{Once, ONCE_INIT};
    use jsonpath::parser::Parser;
    use std::io::Read;
    use super::*;

    static INIT: Once = ONCE_INIT;

    fn setup() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    fn new_value_filter(file: &str) -> ValueFilter {
        let string = read_json(file);
        let json: Value = serde_json::from_str(string.as_str()).unwrap();
        ValueFilter::new(json)
    }

    fn do_filter(path: &str, file: &str) -> JsonValueFilter {
        let string = read_json(file);
        let mut jf = JsonValueFilter::new(string.as_str()).unwrap();
        let mut parser = Parser::new(path);
        parser.parse(&mut jf).unwrap();
        jf
    }

    fn read_json(path: &str) -> String {
        let mut f = std::fs::File::open(path).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();
        contents
    }

    #[test]
    fn step_in() {
        setup();

        let mut jf = new_value_filter("./benches/data_obj.json");
        {
            let current = jf.step_in_str("friends");
            assert_eq!(current.is_array(), true);
        }

        let mut jf = new_value_filter("./benches/data_array.json");
        {
            let current = jf.step_in_num(&1);
            assert_eq!(current.val.is_object(), true);
        }
        {
            let current = jf.step_in_str("friends");
            assert_eq!(current.is_array(), true);
        }
    }

    #[test]
    fn array() {
        setup();

        let friends = json!([
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]);

        let jf = do_filter("$.school.friends[1, 2]", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school.friends[1:]", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school.friends[:-2]", "./benches/data_obj.json");
        let friends = json!([
            {"id": 0, "name": "Millicent Norman"}
        ]);
        assert_eq!(&friends, jf.current_value());
    }

    #[test]
    fn return_type() {
        setup();

        let friends = json!({
            "friends": [
                {"id": 0, "name": "Millicent Norman"},
                {"id": 1, "name": "Vincent Cannon" },
                {"id": 2, "name": "Gray Berry"}
            ]
        });

        let jf = do_filter("$.school", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school[?(@.friends[0])]", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school[?(@.friends[10])]", "./benches/data_obj.json");
        assert_eq!(&Value::Null, jf.current_value());

        let jf = do_filter("$.school[?(1==1)]", "./benches/data_obj.json");
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.school.friends[?(1==1)]", "./benches/data_obj.json");
        let friends = json!([
            {"id": 0, "name": "Millicent Norman"},
            {"id": 1, "name": "Vincent Cannon" },
            {"id": 2, "name": "Gray Berry"}
        ]);
        assert_eq!(&friends, jf.current_value());
    }

    #[test]
    fn op() {
        setup();

        let jf = do_filter("$.school[?(@.friends == @.friends)]", "./benches/data_obj.json");
        let friends = json!({
            "friends": [
                {"id": 0, "name": "Millicent Norman"},
                {"id": 1, "name": "Vincent Cannon" },
                {"id": 2, "name": "Gray Berry"}
            ]
        });
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.friends[?(@.name)]", "./benches/data_obj.json");
        let friends = json!([
            { "id" : 1, "name" : "Vincent Cannon" },
            { "id" : 2, "name" : "Gray Berry" }
        ]);
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.friends[?(@.id >= 2)]", "./benches/data_obj.json");
        let friends = json!([
            { "id" : 2, "name" : "Gray Berry" }
        ]);
        assert_eq!(&friends, jf.current_value());

        //
        // TODO 원본 json 순서여야 하나?
        //
        let jf = do_filter("$.friends[?(@.id >= 2 || @.id == 1)]", "./benches/data_obj.json");
        let friends = json!([
            { "id" : 2, "name" : "Gray Berry" },
            { "id" : 1, "name" : "Vincent Cannon" }
        ]);
        assert_eq!(&friends, jf.current_value());

        let jf = do_filter("$.friends[?( (@.id >= 2 || @.id == 1) && @.id == 0)]", "./benches/data_obj.json");
        assert_eq!(&Value::Null, jf.current_value());
    }
}