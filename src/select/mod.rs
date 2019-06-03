use array_tool::vec::{Intersect, Union};
use serde_json::{Number, Value};

use parser::parser::*;
use std::collections::HashSet;

fn to_f64(n: &Number) -> f64 {
    if n.is_i64() {
        n.as_i64().unwrap() as f64
    } else if n.is_f64() {
        n.as_f64().unwrap()
    } else {
        n.as_u64().unwrap() as f64
    }
}

trait Cmp {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool;

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool;

    fn cmp_string(&self, v1: &String, v2: &String) -> bool;

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value>;

    fn default(&self) -> bool { false }
}

struct CmpEq;

impl Cmp for CmpEq {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 == v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 == v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 == v2
    }

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value> {
        v1.intersect(v2.to_vec())
    }
}

struct CmpNe;

impl Cmp for CmpNe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 != v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 != v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 != v2
    }

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value> {
        v1.intersect_if(v2.to_vec(), |a, b| a != b)
    }
}

struct CmpGt;

impl Cmp for CmpGt {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 > v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 > v2
    }

    fn cmp_json<'a>(&self, _: &Vec<&'a Value>, _: &Vec<&'a Value>) -> Vec<&'a Value> {
        Vec::new()
    }
}

struct CmpGe;

impl Cmp for CmpGe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 >= v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 >= v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 >= v2
    }

    fn cmp_json<'a>(&self, _: &Vec<&'a Value>, _: &Vec<&'a Value>) -> Vec<&'a Value> {
        Vec::new()
    }
}

struct CmpLt;

impl Cmp for CmpLt {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 < v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 < v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 < v2
    }

    fn cmp_json<'a>(&self, _: &Vec<&'a Value>, _: &Vec<&'a Value>) -> Vec<&'a Value> {
        Vec::new()
    }
}

struct CmpLe;

impl Cmp for CmpLe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 <= v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 <= v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 <= v2
    }

    fn cmp_json<'a>(&self, _: &Vec<&'a Value>, _: &Vec<&'a Value>) -> Vec<&'a Value> {
        Vec::new()
    }
}

struct CmpAnd;

impl Cmp for CmpAnd {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        *v1 && *v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > &0_f64 && v2 > &0_f64
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        !v1.is_empty() && !v2.is_empty()
    }

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value> {
        v1.intersect(v2.to_vec())
    }
}

struct CmpOr;

impl Cmp for CmpOr {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        *v1 || *v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > &0_f64 || v2 > &0_f64
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        !v1.is_empty() || !v2.is_empty()
    }

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value> {
        v1.union(v2.to_vec())
    }
}

#[derive(Debug)]
enum ExprTerm<'a> {
    String(String),
    Number(Number),
    Bool(bool),
    Json(Option<FilterKey>, Vec<&'a Value>),
}

impl<'a> ExprTerm<'a> {
    fn is_string(&self) -> bool {
        match &self {
            ExprTerm::String(_) => true,
            _ => false
        }
    }

    fn is_number(&self) -> bool {
        match &self {
            ExprTerm::Number(_) => true,
            _ => false
        }
    }

    fn is_bool(&self) -> bool {
        match &self {
            ExprTerm::Bool(_) => true,
            _ => false
        }
    }

    fn is_json(&self) -> bool {
        match &self {
            ExprTerm::Json(_, _) => true,
            _ => false
        }
    }

    fn cmp<C1: Cmp, C2: Cmp>(&self, other: &Self, cmp_fn: &C1, reverse_cmp_fn: &C2) -> ExprTerm<'a> {
        match &self {
            ExprTerm::String(s1) => match &other {
                ExprTerm::String(s2) => ExprTerm::Bool(cmp_fn.cmp_string(s1, s2)),
                ExprTerm::Json(_, _) => {
                    other.cmp(&self, reverse_cmp_fn, cmp_fn)
                }
                _ => ExprTerm::Bool(cmp_fn.default())
            }
            ExprTerm::Number(n1) => match &other {
                ExprTerm::Number(n2) => ExprTerm::Bool(cmp_fn.cmp_f64(&to_f64(n1), &to_f64(n2))),
                ExprTerm::Json(_, _) => {
                    other.cmp(&self, reverse_cmp_fn, cmp_fn)
                }
                _ => ExprTerm::Bool(cmp_fn.default())
            }
            ExprTerm::Bool(b1) => match &other {
                ExprTerm::Bool(b2) => ExprTerm::Bool(cmp_fn.cmp_bool(b1, b2)),
                ExprTerm::Json(_, _) => {
                    other.cmp(&self, reverse_cmp_fn, cmp_fn)
                }
                _ => ExprTerm::Bool(cmp_fn.default())
            }
            ExprTerm::Json(fk1, vec1) if other.is_string() => {
                match &other {
                    ExprTerm::String(s2) => {
                        let ret: Vec<&Value> = vec1.iter().filter(|v1| {
                            match v1 {
                                Value::String(s1) => cmp_fn.cmp_string(s1, s2),
                                Value::Object(map1) => {
                                    if let Some(FilterKey::String(k)) = fk1 {
                                        if let Some(Value::String(s1)) = map1.get(k) {
                                            return cmp_fn.cmp_string(s1, s2);
                                        }
                                    }
                                    cmp_fn.default()
                                }
                                _ => cmp_fn.default()
                            }
                        }).map(|v| *v).collect();

                        if ret.is_empty() { ExprTerm::Bool(cmp_fn.default()) } else { ExprTerm::Json(None, ret) }
                    }
                    _ => unreachable!()
                }
            }
            ExprTerm::Json(fk1, vec1) if other.is_number() => {
                match &other {
                    ExprTerm::Number(n2) => {
                        let ret: Vec<&Value> = vec1.iter().filter(|v1| {
                            match v1 {
                                Value::Number(n1) => cmp_fn.cmp_f64(&to_f64(n1), &to_f64(n2)),
                                Value::Object(map1) => {
                                    if let Some(FilterKey::String(k)) = fk1 {
                                        if let Some(Value::Number(n1)) = map1.get(k) {
                                            return cmp_fn.cmp_f64(&to_f64(n1), &to_f64(n2));
                                        }
                                    }
                                    cmp_fn.default()
                                }
                                _ => cmp_fn.default()
                            }
                        }).map(|v| *v).collect();

                        if ret.is_empty() { ExprTerm::Bool(cmp_fn.default()) } else { ExprTerm::Json(None, ret) }
                    }
                    _ => unreachable!()
                }
            }
            ExprTerm::Json(fk1, vec1) if other.is_bool() => {
                match &other {
                    ExprTerm::Bool(b2) => {
                        let ret: Vec<&Value> = vec1.iter().filter(|v1| {
                            match v1 {
                                Value::Bool(b1) => cmp_fn.cmp_bool(b1, b2),
                                Value::Object(map1) => {
                                    if let Some(FilterKey::String(k)) = fk1 {
                                        if let Some(Value::Bool(b1)) = map1.get(k) {
                                            return cmp_fn.cmp_bool(b1, b2);
                                        }
                                    }
                                    cmp_fn.default()
                                }
                                _ => cmp_fn.default()
                            }
                        }).map(|v| *v).collect();

                        if ret.is_empty() { ExprTerm::Bool(cmp_fn.default()) } else { ExprTerm::Json(None, ret) }
                    }
                    _ => unreachable!()
                }
            }
            ExprTerm::Json(_, vec1) if other.is_json() => {
                match &other {
                    ExprTerm::Json(_, vec2) => {
                        let vec = cmp_fn.cmp_json(vec1, vec2);
                        if vec.is_empty() { ExprTerm::Bool(cmp_fn.default()) } else { ExprTerm::Json(None, vec) }
                    }
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        }
    }

    fn eq(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("eq - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpEq, &CmpEq);
        debug!("eq = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn ne(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("ne - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpNe, &CmpNe);
        debug!("ne = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn gt(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("gt - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpGt, &CmpLt);
        debug!("gt = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn ge(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("ge - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpGe, &CmpLe);
        debug!("ge = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn lt(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("lt - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpLt, &CmpGt);
        debug!("lt = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn le(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("le - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpLe, &CmpGe);
        debug!("le = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn and(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("and - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpAnd, &CmpAnd);
        debug!("and = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn or(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("or - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpOr, &CmpOr);
        debug!("or = {:?}", tmp);
        *ret = Some(tmp);
    }
}

fn walk_all_with_str<'a>(vec: &Vec<&'a Value>, tmp: &mut Vec<&'a Value>, key: &str, is_filter: bool) {
    if is_filter {
        walk(vec, tmp, &|v| {
            match v {
                Value::Object(map) if map.contains_key(key) => {
                    Some(vec![v])
                }
                _ => None
            }
        });
    } else {
        walk(vec, tmp, &|v| {
            match v {
                Value::Object(map) => match map.get(key) {
                    Some(v) => Some(vec![v]),
                    _ => None
                }
                _ => None
            }
        });
    }
}

fn walk_all<'a>(vec: &Vec<&'a Value>, tmp: &mut Vec<&'a Value>) {
    walk(vec, tmp, &|v| match v {
        Value::Array(vec) => {
            Some(vec.iter().collect())
        }
        Value::Object(map) => {
            let mut tmp = Vec::new();
            for (_, v) in map {
                tmp.push(v);
            }
            Some(tmp)
        }
        _ => None
    });
}

fn walk<'a, F>(vec: &Vec<&'a Value>, tmp: &mut Vec<&'a Value>, fun: &F)
    where F: Fn(&Value) -> Option<Vec<&Value>>
{
    fn _walk<'a, F>(v: &'a Value, tmp: &mut Vec<&'a Value>, fun: &F)
        where F: Fn(&Value) -> Option<Vec<&Value>>
    {
        if let Some(mut ret) = fun(v) {
            tmp.append(&mut ret);
        }

        match v {
            Value::Array(vec) => {
                for v in vec {
                    _walk(v, tmp, fun);
                }
            }
            Value::Object(map) => {
                for (_, v) in map {
                    _walk(&v, tmp, fun);
                }
            }
            _ => {}
        }
    }

    for v in vec {
        _walk(v, tmp, fun);
    }
}

fn abs_index(n: &isize, len: usize) -> usize {
    if n < &0_isize {
        (n + len as isize) as usize
    } else {
        *n as usize
    }
}

#[derive(Debug)]
enum FilterKey {
    String(String),
    All,
}

#[derive(Debug)]
pub enum JsonPathError {
    EmptyPath,
    EmptyValue,
    Path(String),
    Serde(String),
}

#[derive(Debug)]
pub struct Selector<'a> {
    node: Option<Node>,
    value: Option<&'a Value>,
    tokens: Vec<ParseToken>,
    terms: Vec<Option<ExprTerm<'a>>>,
    current: Option<Vec<&'a Value>>,
    selectors: Vec<Selector<'a>>,
}

impl<'a> Selector<'a> {
    pub fn new() -> Self {
        Selector {
            node: None,
            value: None,
            tokens: Vec::new(),
            terms: Vec::new(),
            current: None,
            selectors: Vec::new(),
        }
    }

    pub fn path(&mut self, path: &str) -> Result<&mut Self, JsonPathError> {
        debug!("path : {}", path);
        self.node = Some(Parser::compile(path).map_err(|e| JsonPathError::Path(e))?);
        Ok(self)
    }

    pub(crate) fn reset_value(&mut self) -> &mut Self {
        self.current = None;
        self
    }

    pub fn compiled_path(&mut self, node: Node) -> &mut Self {
        self.node = Some(node);
        self
    }

    pub fn value(&mut self, v: &'a Value) -> &mut Self {
        self.value = Some(v);
        self
    }

    fn _select(&mut self) -> Result<(), JsonPathError> {
        match self.node.take() {
            Some(node) => {
                self.visit(&node);
                self.node = Some(node);
                Ok(())
            }
            _ => Err(JsonPathError::EmptyPath)
        }
    }

    pub fn select_as<T: serde::de::DeserializeOwned>(&mut self) -> Result<Vec<T>, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(vec) => {
                let mut ret = Vec::new();
                for v in vec {
                    match T::deserialize(*v) {
                        Ok(v) => ret.push(v),
                        Err(e) => return Err(JsonPathError::Serde(e.to_string()))
                    }
                }
                Ok(ret)
            }
            _ => Err(JsonPathError::EmptyValue)
        }
    }

    pub fn select_as_str(&mut self) -> Result<String, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(r) => {
                Ok(serde_json::to_string(r)
                    .map_err(|e| JsonPathError::Serde(e.to_string()))?)
            }
            _ => Err(JsonPathError::EmptyValue)
        }
    }

    pub fn select(&mut self) -> Result<Vec<&'a Value>, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(r) => Ok(r.to_vec()),
            _ => Err(JsonPathError::EmptyValue)
        }
    }

    fn new_filter_context(&mut self) {
        self.terms.push(None);
        debug!("new_filter_context: {:?}", self.terms);
    }

    fn in_filter<F: Fn(&Vec<&'a Value>, &mut Vec<&'a Value>) -> FilterKey>(&mut self, fun: F) {
        match self.terms.pop() {
            Some(peek) => {
                match peek {
                    Some(v) => {
                        debug!("in_filter 1.: {:?}", v);

                        match v {
                            ExprTerm::Json(_, vec) => {
                                let mut tmp = Vec::new();
                                let filter_key = fun(&vec, &mut tmp);
                                self.terms.push(Some(ExprTerm::Json(Some(filter_key), tmp)));
                            }
                            _ => unreachable!()
                        };
                    }
                    _ => {
                        debug!("in_filter 2.: {:?}", &self.current);

                        if let Some(current) = &self.current {
                            let mut tmp = Vec::new();
                            let filter_key = fun(current, &mut tmp);
                            self.terms.push(Some(ExprTerm::Json(Some(filter_key), tmp)));
                        }
                    }
                };
            }
            _ => {}
        }
    }

    fn all_in_filter_with_str(&mut self, key: &str) {
        self.in_filter(|vec, tmp| {
            walk_all_with_str(&vec, tmp, key, true);
            FilterKey::All
        });

        debug!("all_in_filter_with_str : {}, {:?}", key, self.terms);
    }

    fn next_in_filter_with_str(&mut self, key: &str) {
        fn _collect<'a>(v: &'a Value, tmp: &mut Vec<&'a Value>, key: &str, visited: &mut HashSet<*const Value>) {
            match v {
                Value::Object(map) => if map.contains_key(key) {
                    let ptr = v as *const Value;
                    if !visited.contains(&ptr) {
                        visited.insert(ptr);
                        tmp.push(v)
                    }
                },
                Value::Array(vec) => for v in vec {
                    _collect(v, tmp, key, visited);
                }
                _ => {}
            }
        }

        self.in_filter(|vec, tmp| {
            let mut visited = HashSet::new();
            for v in vec {
                _collect(v, tmp, key, &mut visited);
            }
            FilterKey::String(key.to_owned())
        });

        debug!("next_in_filter_with_str : {}, {:?}", key, self.terms);
    }

    fn next_from_current_with_num(&mut self, index: f64) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            for c in current {
                if let Value::Array(vec) = c {
                    let index = abs_index(&(index as isize), vec.len());
                    if let Some(v) = c.get(index) {
                        tmp.push(v);
                    }
                }
            }
            self.current = Some(tmp);
        }

        debug!("next_from_current_with_num : {:?}, {:?}", &index, self.current);
    }

    fn next_from_current_with_str(&mut self, key: &str) {
        fn _collect<'a>(v: &'a Value, tmp: &mut Vec<&'a Value>, key: &str, visited: &mut HashSet<*const Value>) {
            match v {
                Value::Object(map) => {
                    if let Some(v) = map.get(key) {
                        let ptr = v as *const Value;
                        if !visited.contains(&ptr) {
                            visited.insert(ptr);
                            tmp.push(v)
                        }
                    }
                }
                Value::Array(vec) => for v in vec {
                    _collect(v, tmp, key, visited);
                }
                _ => {}
            }
        }

        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            let mut visited = HashSet::new();
            for c in current {
                _collect(c, &mut tmp, key, &mut visited);
            }
            self.current = Some(tmp);
        }

        debug!("next_from_current_with_str : {}, {:?}", key, self.current);
    }

    fn next_all_from_current(&mut self) {
        fn _collect<'a>(v: &'a Value, tmp: &mut Vec<&'a Value>) {
            match v {
                Value::Object(map) => {
                    for (_, v) in map {
                        tmp.push(v)
                    }
                }
                Value::Array(vec) => for v in vec {
                    _collect(v, tmp);
                }
                _ => {}
            }
        }

        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            for c in current {
                _collect(c, &mut tmp);
            }
            self.current = Some(tmp);
        }

        debug!("next_all_from_current : {:?}", self.current);
    }

    fn all_from_current(&mut self) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            walk_all(&current, &mut tmp);
            self.current = Some(tmp);
        }
        debug!("all_from_current: {:?}", self.current);
    }

    fn all_from_current_with_str(&mut self, key: &str) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            walk_all_with_str(&current, &mut tmp, key, false);
            self.current = Some(tmp);
        }
        debug!("all_from_current_with_str: {}, {:?}", key, self.current);
    }
}

impl<'a> NodeVisitor for Selector<'a> {
    fn visit_token(&mut self, token: &ParseToken) {
        debug!("token: {:?}, stack: {:?}", token, self.tokens);

        if !self.selectors.is_empty() {
            match token {
                ParseToken::Absolute | ParseToken::Relative | ParseToken::Filter(_) => {
                    let s = self.selectors.pop().unwrap();

                    if let Some(current) = &s.current {
                        let term = if current.len() == 1 {
                            match current[0] {
                                Value::Number(v) => ExprTerm::Number(v.clone()),
                                Value::String(v) => ExprTerm::String(v.clone()),
                                Value::Bool(v) => ExprTerm::Bool(*v),
                                _ => ExprTerm::Json(None, current.to_vec())
                            }
                        } else {
                            ExprTerm::Json(None, current.to_vec())
                        };

                        if let Some(s) = self.selectors.last_mut() {
                            s.terms.push(Some(term));
                        } else {
                            self.terms.push(Some(term));
                        }
                    } else {
                        unreachable!()
                    }
                }
                _ => {}
            }
        }

        if let Some(s) = self.selectors.last_mut() {
            s.visit_token(token);
            return;
        }

        match token {
            ParseToken::Absolute => {
                if self.current.is_some() {
                    let mut s = Selector::new();
                    if let Some(value) = self.value {
                        s.value = Some(value);
                        s.current = Some(vec![value]);
                        self.selectors.push(s);
                    }
                    return;
                }

                match &self.value {
                    Some(v) => self.current = Some(vec![v]),
                    _ => {}
                }
            }
            ParseToken::Relative => {
                self.new_filter_context();
            }
            ParseToken::In | ParseToken::Leaves => {
                self.tokens.push(token.clone());
            }
            ParseToken::Array => {
                if let Some(ParseToken::Leaves) = self.tokens.last() {
                    self.tokens.pop();
                    self.all_from_current();
                }

                self.tokens.push(token.clone());
            }
            ParseToken::ArrayEof => {
                if let Some(Some(e)) = self.terms.pop() {
                    match e {
                        ExprTerm::Number(n) => {
                            self.next_from_current_with_num(to_f64(&n));
                        }
                        ExprTerm::String(key) => {
                            self.next_from_current_with_str(&key);
                        }
                        ExprTerm::Json(_, v) => {
                            if v.is_empty() {
                                self.current = Some(vec![&Value::Null]);
                            } else {
                                self.current = Some(v);
                            }
                        }
                        ExprTerm::Bool(false) => {
                            self.current = Some(vec![&Value::Null]);
                        }
                        _ => {}
                    }
                }

                self.tokens.pop();
            }
            ParseToken::All => {
                match self.tokens.last() {
                    Some(ParseToken::Leaves) => {
                        self.tokens.pop();
                        self.all_from_current();
                    }
                    Some(ParseToken::In) => {
                        self.tokens.pop();
                        self.next_all_from_current();
                    }
                    _ => {}
                }
            }
            ParseToken::Bool(b) => {
                self.terms.push(Some(ExprTerm::Bool(*b)));
            }
            ParseToken::Key(key) => {
                if let Some(ParseToken::Array) = self.tokens.last() {
                    self.terms.push(Some(ExprTerm::String(key.clone())));
                    return;
                }

                match self.tokens.pop() {
                    Some(t) => {
                        if self.terms.is_empty() {
                            match t {
                                ParseToken::Leaves => {
                                    self.all_from_current_with_str(key.as_str())
                                }
                                ParseToken::In => {
                                    self.next_from_current_with_str(key.as_str())
                                }
                                _ => {}
                            }
                        } else {
                            match t {
                                ParseToken::Leaves => {
                                    self.all_in_filter_with_str(key.as_str());
                                }
                                ParseToken::In => {
                                    self.next_in_filter_with_str(key.as_str());
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            ParseToken::Number(v) => {
                self.terms.push(Some(ExprTerm::Number(Number::from_f64(*v).unwrap())));
            }
            ParseToken::Filter(ref ft) => {
                if let Some(Some(ref right)) = self.terms.pop() {
                    if let Some(Some(left)) = self.terms.pop() {
                        let mut ret = None;
                        match ft {
                            FilterToken::Equal => left.eq(right, &mut ret),
                            FilterToken::NotEqual => left.ne(right, &mut ret),
                            FilterToken::Greater => left.gt(right, &mut ret),
                            FilterToken::GreaterOrEqual => left.ge(right, &mut ret),
                            FilterToken::Little => left.lt(right, &mut ret),
                            FilterToken::LittleOrEqual => left.le(right, &mut ret),
                            FilterToken::And => left.and(right, &mut ret),
                            FilterToken::Or => left.or(right, &mut ret),
                        };

                        if let Some(e) = ret {
                            self.terms.push(Some(e));
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            ParseToken::Range(from, to) => {
                if !self.terms.is_empty() {
                    unimplemented!("range syntax in filter");
                }

                if let Some(ParseToken::Array) = self.tokens.pop() {
                    let mut tmp = Vec::new();
                    for vec in &self.current {
                        for v in vec {
                            if let Value::Array(vec) = v {
                                let from = if let Some(from) = from {
                                    abs_index(from, vec.len())
                                } else {
                                    0
                                };

                                let to = if let Some(to) = to {
                                    abs_index(to, vec.len())
                                } else {
                                    vec.len()
                                };

                                for i in from..to {
                                    if let Some(v) = vec.get(i) {
                                        tmp.push(v);
                                    }
                                }
                            }
                        }
                    }

                    self.current = Some(tmp);
                } else {
                    unreachable!();
                }
            }
            ParseToken::Union(indices) => {
                if !self.terms.is_empty() {
                    unimplemented!("union syntax in filter");
                }

                if let Some(ParseToken::Array) = self.tokens.pop() {
                    let mut tmp = Vec::new();
                    for vec in &self.current {
                        for v in vec {
                            if let Value::Array(vec) = v {
                                for i in indices {
                                    if let Some(v) = vec.get(abs_index(i, vec.len())) {
                                        tmp.push(v);
                                    }
                                }
                            }
                        }
                    }

                    self.current = Some(tmp);
                } else {
                    unreachable!();
                }
            }
            ParseToken::Eof => {
                debug!("visit_token eof");
            }
        }
    }
}


//pub trait Transform<'a> {
//    fn map<F>(&mut self, mut fun: F) -> &mut Self where F: FnMut(&'a mut Value);
//}
//
//impl<'a> Transform<'a> for Selector<'a> {
//    fn map<F>(&mut self, mut fun: F) -> &mut Self where F: FnMut(&'a mut Value) {
//        match &mut self.current {
//            Some(current) => {
//                current.iter_mut().for_each(|ref mut v| fun(v))
//            }
//            _ => {}
//        }
//
//        self
//    }
//}