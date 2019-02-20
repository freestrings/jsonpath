use core::borrow::Borrow;
use std::cmp::Ordering;
use std::error::Error;
use std::io::Read;
use std::rc::Rc;
use std::result;

use serde_json::Value;
use serde_json::value::Index;

use jsonpath::parser::{
    FilterToken,
    NodeVisitor,
    Parser,
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
    Json(ValueWrapper),
}

impl TermContext {
    fn cmp<F: PrivCmp + IntoType>(&mut self, other: &mut TermContext, cmp_fn: F, default: bool) -> TermContext {
        match self {
            TermContext::Constants(et) => {
                match other {
                    TermContext::Constants(oet) => {
                        TermContext::Constants(ExprTerm::Bool(et.cmp(oet, cmp_fn, default)))
                    }
                    TermContext::Json(v) => {
                        TermContext::Json(v.take_with(et, cmp_fn))
                    }
                }
            }
            TermContext::Json(v) => {
                match other {
                    TermContext::Json(ov) => {
                        v.cmp(ov, cmp_fn.into_type())
                    }
                    TermContext::Constants(et) => {
                        TermContext::Json(v.take_with(et, cmp_fn))
                    }
                }
            }
        }
    }

    fn cmp_cond<F: PrivCmp>(&mut self, other: &mut TermContext, cmp_fn: F) -> TermContext {
        match self {
            TermContext::Constants(et) => {
                match other {
                    TermContext::Constants(oet) => {
                        TermContext::Constants(ExprTerm::Bool(et.cmp(oet, cmp_fn, false)))
                    }
                    TermContext::Json(v) => {
                        TermContext::Json(ValueWrapper::new(v.clone_data()))
                    }
                }
            }
            TermContext::Json(v) => {
                match other {
                    TermContext::Json(ov) => {
                        TermContext::Json(v.union(ov))
                    }
                    _ => {
                        TermContext::Json(ValueWrapper::new(v.clone_data()))
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
        self.cmp_cond(other, CmpAnd)
    }

    fn or(&mut self, other: &mut TermContext) -> TermContext {
        self.cmp_cond(other, CmpOr)
    }
}

pub struct JsonValueFilter {
    json: Rc<Box<Value>>,
    current: ValueWrapper,
    stack: Vec<ParseToken>,
    filter_stack: Vec<TermContext>,
    in_array: bool,
}

impl NodeVisitor for JsonValueFilter {
    fn visit_token(&mut self, token: ParseToken) {
        debug!("visit_token: {:?}", token);
        match token {
            ParseToken::Absolute
            | ParseToken::Relative
            | ParseToken::In
            | ParseToken::Leaves => {
                self.stack.push(token);
            }
            ParseToken::Array => {
                self.in_array = true;
            }
            ParseToken::ArrayEof => {
                self.in_array = false;
                match self.filter_stack.pop() {
                    Some(TermContext::Constants(_)) => unreachable!(),
                    Some(TermContext::Json(v)) => self.current.replace(vec![v.clone_data()]),
                    _ => {}
                }

                if !self.filter_stack.is_empty() {
                    panic!()
                }
            }
            ParseToken::All => {
                if self.in_array {
                    self.stack.push(token);
                } else {
                    match self.stack.pop() {
                        Some(ParseToken::In) => {
                            self.step_in_all();
                        }
                        Some(ParseToken::Leaves) => {
                            self.step_leaves_all();
                        }
                        _ => {}
                    }
                }
            }
            ParseToken::Key(key) => {
                if self.in_array {
                    self.stack.push(ParseToken::Key(key));
                } else {
                    match self.stack.pop() {
                        Some(ParseToken::In) => {
                            self.step_in(key);
                        }
                        Some(ParseToken::Leaves) => {
                            self.step_leaves(key);
                        }
                        _ => {}
                    }

                    if match self.stack.last() {
                        Some(ParseToken::Absolute) | Some(ParseToken::Relative) => true,
                        _ => false
                    } {
                        self.stack.pop();
                    }
                }
            }
            ParseToken::Number(_) => {
                self.stack.push(token);
            }
            ParseToken::Filter(ref ft) => {
                let left = self.filter_stack.pop();
                let right = self.filter_stack.pop();

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
                        self.filter_stack.push(tc);
                    }
                }
            }

            other => {
                debug!("visit_token other: {:?}", other);
            }
        }
    }

    fn clean_filter_context(&mut self) {
        debug!("clean_filter_context");
        self.clean_filter_path();
        self.clean_filter_constants();
    }
}

impl JsonValueFilter {
    pub fn new(json: &str) -> result::Result<Self, String> {
        let json: Value = serde_json::from_str(json)
            .map_err(|e| e.description().to_string())?;
        let root = json.clone();
        Ok(JsonValueFilter {
            json: Rc::new(Box::new(json)),
            current: ValueWrapper::new(root),
            stack: Vec::new(),
            filter_stack: Vec::new(),
            in_array: false,
        })
    }

    fn fork(&self, from_current: bool) -> Self {
        JsonValueFilter {
            json: self.json.clone(),
            current: if from_current {
                ValueWrapper::new(self.current.clone_data())
            } else {
                let v: &Value = self.json.as_ref().borrow();
                ValueWrapper::new(v.clone())
            },
            stack: Vec::new(),
            filter_stack: Vec::new(),
            in_array: false,
        }
    }

    fn clean_filter_path(&mut self) {
        let mut paths = Vec::new();

        loop {
            trace!("clean_filter_path - loop: {:?}", self.stack.last());

            if match self.stack.last() {
                Some(ParseToken::Absolute)
                | Some(ParseToken::Relative)
                | Some(ParseToken::In)
                | Some(ParseToken::Leaves)
                | Some(ParseToken::All)
                | Some(ParseToken::Key(_)) => true,
                _ => false
            } {
                self.stack.pop().map(|t| paths.push(t));
            } else {
                break;
            }
        }

        trace!("clean_filter_path: {:?}", paths);

        if let Some(forked) = match paths.pop() {
            Some(ParseToken::Absolute) => {
                Some(self.fork(false))
            }
            Some(ParseToken::Relative) => {
                Some(self.fork(true))
            }
            _ => None
        }.and_then(|mut forked| {
            while let Some(t) = paths.pop() {
                forked.visit_token(t);
            }
            Some(forked)
        }) {
            trace!("clean_filter_path -> {:?}", forked.current);
            self.filter_stack.push(TermContext::Json(forked.current));
        }
    }

    fn clean_filter_constants(&mut self) {
        trace!("clean_filter_constants: {:?}", self.stack.last());

        if match self.stack.last() {
            Some(ParseToken::Key(_))
            | Some(ParseToken::Number(_)) => true,
            _ => false
        } {
            match self.stack.pop() {
                Some(ParseToken::Key(ref v)) if v.eq_ignore_ascii_case("true") => {
                    self.filter_stack.push(TermContext::Constants(ExprTerm::Bool(true)))
                }
                Some(ParseToken::Key(ref v)) if v.eq_ignore_ascii_case("false") => {
                    self.filter_stack.push(TermContext::Constants(ExprTerm::Bool(false)))
                }
                Some(ParseToken::Key(v)) => {
                    self.filter_stack.push(TermContext::Constants(ExprTerm::String(v)))
                }
                Some(ParseToken::Number(v)) => {
                    self.filter_stack.push(TermContext::Constants(ExprTerm::Number(v)))
                }
                _ => {}
            }
        }
    }

    fn step_leaves_all(&mut self) -> &ValueWrapper {
        debug!("step_leaves_all");

        let mut vw = ValueWrapper::new(Value::Null);
        loop {
            vw.push(self.step_in_all().clone_data());
            if let Value::Null = self.current._val {
                break;
            }
        }
        self.current = vw;
        &self.current
    }

    fn step_leaves(&mut self, key: String) -> &ValueWrapper {
        debug!("step_leaves");

        let mut vw = ValueWrapper::new(Value::Null);
        loop {
            vw.push(self.step_in(key.clone()).clone_data());
            if let Value::Null = self.current._val {
                break;
            }
        }
        self.current = vw;
        &self.current
    }

    fn step_in_all(&mut self) -> &ValueWrapper {
        debug!("step_in_all");

        fn to_vec<'a, I: Iterator<Item=&'a mut Value>>(iter: I) -> Vec<Value> {
            iter.map(|v| v.take())
                .filter(|v| !v.is_null())
                .collect()
        }

        let vec = match &mut self.current._val {
            Value::Object(map) => to_vec(map.values_mut()),
            Value::Array(list) => to_vec(list.iter_mut()),
            Value::Null => Vec::new(),
            other => vec![other.take()]
        };

        self.current.replace(vec);
        &self.current
    }

    fn step_in<I: Index>(&mut self, key: I) -> &ValueWrapper {
        debug!("step_in");

        let v = match self.current._val.get_mut(&key) {
            Some(value) => value.take(),
            _ => Value::Null
        };

        trace!("{:?}", v);

        self.current.replace(vec![v]);

        &self.current
    }

    fn current(&self) -> &ValueWrapper {
        &self.current
    }
}

#[derive(Debug)]
struct ValueWrapper {
    _val: Value,
}

impl ValueWrapper {
    fn new(v: Value) -> Self {
        ValueWrapper { _val: v }
    }

    fn cmp(&mut self, other: &mut ValueWrapper, cmp_type: CmpType) -> TermContext {
        match cmp_type {
            CmpType::Eq => {
                TermContext::Json(self.intersect(other))
            }
            CmpType::Ne => {
                TermContext::Json(self.except(other))
            }
            CmpType::Gt | CmpType::Ge | CmpType::Lt | CmpType::Le => {
                TermContext::Constants(ExprTerm::Bool(false))
            }
        }
    }

    fn cmp_with_term<F: PrivCmp>(&self, et: &ExprTerm, cmp_fn: &F, default: bool) -> bool {
        match &self._val {
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

    fn take_with<F: PrivCmp>(&mut self, et: &ExprTerm, cmp: F) -> Self {
        match self._val.take() {
            Value::Array(vec) => {
                let mut vw = ValueWrapper::new(Value::Null);
                for v in vec {
                    if self.cmp_with_term(et, &cmp, false) {
                        vw.push(v);
                    }
                }
                vw
            }
            other => {
                if self.cmp_with_term(et, &cmp, false) {
                    ValueWrapper::new(other)
                } else {
                    ValueWrapper::new(Value::Null)
                }
            }
        }
    }

    fn replace(&mut self, values: Vec<Value>) {
        self._val.take();
        for v in values {
            self.push(v);
        }
    }

    fn push(&mut self, v: Value) {
        if let Value::Array(values) = &mut self._val {
            values.push(v);
            return;
        }

        let data = self._val.take();
        if data.is_null() {
            self._val = v;
        } else {
            let mut values = Vec::new();
            values.push(v);
            self._val = Value::Array(values);
        }
    }

    fn clone_data(&self) -> Value {
        self._val.clone()
    }

    fn is_array(&self) -> bool {
        self._val.is_array()
    }

    fn is_object(&self) -> bool {
        self.data().is_object()
    }

    fn is_number(&self) -> bool {
        self.data().is_number()
    }

    fn uuid(v: &Value) -> String {
        fn _fn(v: &Value) -> String {
            match v {
                Value::Null => "null".to_string(),
                Value::String(v) => v.to_string(),
                Value::Bool(v) => v.to_string(),
                Value::Number(v) => v.to_string(),
                Value::Array(v) => {
                    v.iter().enumerate().map(|(i, v)| {
                        format!("{}{}", i, _fn(v))
                    }).collect()
                }
                Value::Object(v) => {
                    v.into_iter().map(|(k, v)| {
                        format!("{}{}", k, _fn(v))
                    }).collect()
                }
            }
        }
        _fn(v)
    }

    fn into_map(&mut self) -> HashMap<String, Value> {
        let mut map: HashMap<String, Value> = HashMap::new();
        match &mut self._val {
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

    fn intersect(&mut self, other: &mut Self) -> Self {
        let map = self.into_map();
        let mut ret: HashMap<String, Value> = HashMap::new();
        match &mut other._val {
            Value::Array(vv2) => {
                for v in vv2 {
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

    fn except(&mut self, other: &mut Self) -> Self {
        let map = self.into_map();
        let mut ret: HashMap<String, Value> = HashMap::new();
        match &mut other._val {
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

    fn union(&mut self, other: &mut Self) -> Self {
        let mut map = self.into_map();
        match &mut other._val {
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
        vw.replace(list);
        vw
    }

    fn data(&self) -> &Value {
        match &self._val {
            Value::Array(v) if v.len() == 1 => {
                self._val.get(0).unwrap()
            }
            other => {
                other
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::sync::{Once, ONCE_INIT};

    use jsonpath::tokenizer::PreloadedTokenizer;

    use super::*;

    static INIT: Once = ONCE_INIT;

    fn setup() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    fn new_filter(file: &str) -> JsonValueFilter {
        let string = read_json(file);
        JsonValueFilter::new(string.as_str()).unwrap()
    }

    fn do_filter(path: &str, file: &str) -> JsonValueFilter {
        let mut jf = new_filter(file);
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

        let mut jf = new_filter("./benches/data_obj.json");
        {
            let current = jf.step_in("friends");
            assert_eq!(current.is_array(), true);
        }

        let mut jf = new_filter("./benches/data_array.json");
        {
            let current = jf.step_in(1);
            assert_eq!(current.is_object(), true);
        }
        {
            let current = jf.step_in("friends");
            assert_eq!(current.is_array(), true);
        }
    }

    #[test]
    fn fork() {
        setup();

        let mut jf = new_filter("./benches/data_obj.json");
        {
            let current = jf.step_in("friends");
            assert_eq!(current.is_array(), true);
        }

        let jf_from_current = jf.fork(true);
        {
            let current = jf_from_current.current();
            assert_eq!(current.is_array(), true);
        }

        let mut jf_from_root = jf_from_current.fork(false);
        {
            let current = jf_from_root.step_in("age");
            assert_eq!(current.is_number(), true);
        }
    }

    #[test]
    fn filter() {
        setup();

        let jf = do_filter("$.school[?(@.friends)]", "./benches/data_obj.json");
        let v = json!({
            "friends" : [
              {"id": 0,"name": "Millicent Norman"},
              {"id": 1,"name": "Vincent Cannon" },
              {"id": 2,"name": "Gray Berry"}
            ]
        });
        assert_eq!(&v, jf.current().data());
    }
}