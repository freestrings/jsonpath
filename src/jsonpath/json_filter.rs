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

#[derive(Debug)]
enum TermContext {
    Constants(ExprTerm),
    Json(Vec<Value>),
}

impl TermContext {
    fn cmp_value_term<'a, F: PrivCmp>(et: &'a ExprTerm, cmp_fn: F, default: bool)
                                      -> impl FnMut(&&'a mut Value) -> bool {
        move |v| match v {
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

    fn cmp_values_term<F: PrivCmp>(v: &mut Vec<Value>, et: &ExprTerm, cmp: F) -> TermContext {
        let ret = v.iter_mut()
            .filter(Self::cmp_value_term(et, cmp, false))
            .map(|v| v.take())
            .collect();
        TermContext::Json(ret)
    }

    fn cmp_term_term<F: PrivCmp>(v1: &ExprTerm, v2: &ExprTerm, cmp_fn: F, default: bool) -> bool {
        match v1 {
            ExprTerm::Bool(vv1) => match v2 {
                ExprTerm::Bool(vv2) => cmp_fn.cmp_bool(vv1, vv2),
                _ => default
            }
            ExprTerm::Number(vv1) => match v2 {
                ExprTerm::Number(vv2) => cmp_fn.cmp_f64(vv1, vv2),
                _ => default
            }
            ExprTerm::String(vv1) => match v2 {
                ExprTerm::String(vv2) => cmp_fn.cmp_string(vv1, vv2),
                _ => default
            }
        }
    }

    fn cmp_value_value(v1: &mut Vec<Value>, v2: &mut Vec<Value>, cmp_type: CmpType) -> TermContext {
        match cmp_type {
            CmpType::Eq => {
                let mut map: HashMap<String, Value> = HashMap::new();
                for v in v1 {
                    map.insert(format!("{:?}", v), v.take());
                }

                let mut ret: HashMap<String, Value> = HashMap::new();
                for v in v2 {
                    let key = format!("{:?}", v);
                    if map.contains_key(&key) {
                        ret.insert(key, v.take());
                    }
                }

                let v = ret.values_mut().into_iter().map(|v| v.take()).collect();
                TermContext::Json(v)
            }
            CmpType::Ne => {
                let mut map: HashMap<String, Value> = HashMap::new();
                for v in v1 {
                    map.insert(format!("{:?}", v), v.take());
                }

                let mut ret: HashMap<String, Value> = HashMap::new();
                for v in v2 {
                    let key = format!("{:?}", v);
                    if !map.contains_key(&key) {
                        ret.insert(key, v.take());
                    }
                }

                let v = ret.values_mut().into_iter().map(|v| v.take()).collect();
                TermContext::Json(v)
            }
            CmpType::Gt | CmpType::Ge | CmpType::Lt | CmpType::Le => {
                TermContext::Constants(ExprTerm::Bool(false))
            }
        }
    }

    fn cmp<F: PrivCmp + IntoType>(&mut self, other: &mut TermContext, cmp_fn: F, default: bool) -> TermContext {
        match self {
            TermContext::Constants(et) => {
                match other {
                    TermContext::Constants(oet) => {
                        let b = Self::cmp_term_term(et, oet, cmp_fn, default);
                        TermContext::Constants(ExprTerm::Bool(b))
                    }
                    TermContext::Json(v) => {
                        Self::cmp_values_term(v, et, cmp_fn)
                    }
                }
            }
            TermContext::Json(v) => {
                match other {
                    TermContext::Json(ov) => {
                        Self::cmp_value_value(v, ov, cmp_fn.into_type())
                    }
                    TermContext::Constants(et) => {
                        Self::cmp_values_term(v, et, cmp_fn)
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

    fn cmp_cond<F: PrivCmp>(&mut self, other: &mut TermContext, cmp_fn: F) -> TermContext {
        match self {
            TermContext::Constants(et) => {
                match other {
                    TermContext::Constants(oet) => {
                        let b = Self::cmp_term_term(et, oet, cmp_fn, false);
                        TermContext::Constants(ExprTerm::Bool(b))
                    }
                    TermContext::Json(v) => {
                        let list = v.iter_mut().map(|v| v.take()).collect();
                        TermContext::Json(list)
                    }
                }
            }
            TermContext::Json(v) => {
                match other {
                    TermContext::Json(ov) => {
                        let mut map: HashMap<String, Value> = HashMap::new();
                        for val in v {
                            map.insert(format!("{:?}", val), val.take());
                        }
                        for val in ov {
                            map.insert(format!("{:?}", val), val.take());
                        }
                        let list: Vec<Value> = map.values_mut().into_iter().map(|val| val.take()).collect();
                        TermContext::Json(list)
                    }
                    TermContext::Constants(et) => {
                        let list = v.iter_mut().map(|v| v.take()).collect();
                        TermContext::Json(list)
                    }
                }
            }
        }
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
    current: Vec<Value>,
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
                    Some(TermContext::Json(v)) => self.current = v,
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
            current: vec![root],
            stack: Vec::new(),
            filter_stack: Vec::new(),
            in_array: false,
        })
    }

    fn fork(&self, from_current: bool) -> Self {
        JsonValueFilter {
            json: self.json.clone(),
            current: if from_current {
                self.current.clone()
            } else {
                let v: &Value = self.json.as_ref().borrow();
                vec![v.clone()]
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

    fn step_leaves_all(&mut self) -> &Vec<Value> {
        debug!("step_leaves_all");

        let mut buf = Vec::new();
        loop {
            self.step_in_all().iter().map(|v| buf.push(v.clone()));
            if self.current.len() == 0 {
                break;
            }
        }
        self.current = buf;
        &self.current
    }

    fn step_leaves(&mut self, key: String) -> &Vec<Value> {
        debug!("step_leaves");

        let mut buf = Vec::new();
        loop {
            self.step_in(key.clone()).iter().map(|v| buf.push(v.clone()));
            if self.current.len() == 0 {
                break;
            }
        }
        self.current = buf;
        &self.current
    }

    fn step_in_all(&mut self) -> &Vec<Value> {
        debug!("step_in_all");

        fn to_vec<'a, I: Iterator<Item=&'a mut Value>>(iter: I) -> Vec<Value> {
            iter.map(|v| v.take())
                .filter(|v| !v.is_null())
                .collect()
        }

        self.current = self.current.iter_mut()
            .flat_map(|v| {
                match v {
                    Value::Object(map) => to_vec(map.values_mut()),
                    Value::Array(list) => to_vec(list.iter_mut()),
                    Value::Null => Vec::new(),
                    _ => vec![v.take()]
                }
            }).collect();

        &self.current
    }

    fn step_in<I: Index>(&mut self, key: I) -> &Vec<Value> {
        debug!("step_in");
        self.current = self.current.iter_mut()
            .map(|v| {
                trace!("step_in - map: {:?}", v);
                match v.get_mut(&key) {
                    Some(value) => value.take(),
                    _ => Value::Null
                }
            })
            .filter(|v| !v.is_null())
            .collect();

        &self.current
    }

    fn current(&self) -> &Vec<Value> {
        &self.current
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

    fn read_json(path: &str) -> String {
        let mut f = std::fs::File::open(path).unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();
        contents
    }

    #[test]
    fn step_in() {
        setup();

        let string = read_json("./benches/data_obj.json");
        let mut jf = JsonValueFilter::new(string.as_str()).unwrap();
        {
            let current = jf.step_in("friends");
            assert_eq!(current[0].is_array(), true);
        }

        let string = read_json("./benches/data_array.json");
        let mut jf = JsonValueFilter::new(string.as_str()).unwrap();
        {
            let current = jf.step_in(1);
            assert_eq!(current[0].is_object(), true);
        }
        {
            let current = jf.step_in("friends");
            assert_eq!(current[0].is_array(), true);
        }
    }

    #[test]
    fn fork() {
        setup();

        let string = read_json("./benches/data_obj.json");
        let mut jf = JsonValueFilter::new(string.as_str()).unwrap();
        {
            let current = jf.step_in("friends");
            assert_eq!(current[0].is_array(), true);
        }

        let jf_from_current = jf.fork(true);
        {
            let current = jf_from_current.current();
            assert_eq!(current[0].is_array(), true);
        }

        let mut jf_from_root = jf_from_current.fork(false);
        {
            let current = jf_from_root.step_in("age");
            assert_eq!(current[0].is_number(), true);
        }
    }

    #[test]
    fn filter() {
        setup();

        let string = read_json("./benches/data_obj.json");
        let mut jf = JsonValueFilter::new(string.as_str()).unwrap();
        let mut parser = Parser::new("$.school[?(@.friends==@.friends)]");
        parser.parse(&mut jf).unwrap();
        let v = json!([
          {"id": 0,"name": "Millicent Norman"},
          {"id": 1,"name": "Vincent Cannon" },
          {"id": 2,"name": "Gray Berry"}
        ]);
        assert_eq!(v, jf.current());
    }
}