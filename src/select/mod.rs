use std::collections::HashSet;
use std::fmt;

use serde_json::{Number, Value};
use serde_json::map::Entry;

use parser::*;

use self::expr_term::*;
use self::value_walker::ValueWalker;

mod cmp;
mod expr_term;
mod value_walker;

fn to_f64(n: &Number) -> f64 {
    if n.is_i64() {
        n.as_i64().unwrap() as f64
    } else if n.is_f64() {
        n.as_f64().unwrap()
    } else {
        n.as_u64().unwrap() as f64
    }
}

fn abs_index(n: isize, len: usize) -> usize {
    if n < 0_isize {
        (n + len as isize).max(0) as usize
    } else {
        n.min(len as isize) as usize
    }
}

#[derive(Debug, PartialEq)]
enum FilterKey {
    String(String),
    All,
}

pub enum JsonPathError {
    EmptyPath,
    EmptyValue,
    Path(String),
    Serde(String),
}

impl fmt::Debug for JsonPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for JsonPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonPathError::EmptyPath => f.write_str("path not set"),
            JsonPathError::EmptyValue => f.write_str("json value not set"),
            JsonPathError::Path(msg) => f.write_str(&format!("path error: \n{}\n", msg)),
            JsonPathError::Serde(msg) => f.write_str(&format!("serde error: \n{}\n", msg)),
        }
    }
}

#[derive(Debug, Default)]
pub struct Selector<'a, 'b> {
    node: Option<Node>,
    node_ref: Option<&'b Node>,
    value: Option<&'a Value>,
    tokens: Vec<ParseToken>,
    terms: Vec<Option<ExprTerm<'a>>>,
    current: Option<Vec<&'a Value>>,
    selectors: Vec<Selector<'a, 'b>>,
}

impl<'a, 'b> Selector<'a, 'b> {
    pub fn new() -> Self {
        Selector::default()
    }

    pub fn str_path(&mut self, path: &str) -> Result<&mut Self, JsonPathError> {
        debug!("path : {}", path);
        self.node_ref.take();
        self.node = Some(Parser::compile(path).map_err(JsonPathError::Path)?);
        Ok(self)
    }

    pub fn node_ref(&self) -> Option<&Node> {
        if let Some(node) = &self.node {
            return Some(node);
        }

        if let Some(node) = &self.node_ref {
            return Some(*node);
        }

        None
    }

    pub fn compiled_path(&mut self, node: &'b Node) -> &mut Self {
        self.node.take();
        self.node_ref = Some(node);
        self
    }

    pub fn reset_value(&mut self) -> &mut Self {
        self.current = None;
        self
    }

    pub fn value(&mut self, v: &'a Value) -> &mut Self {
        self.value = Some(v);
        self
    }

    fn _select(&mut self) -> Result<(), JsonPathError> {
        if self.node_ref.is_some() {
            let node_ref = self.node_ref.take().unwrap();
            self.visit(node_ref);
            return Ok(());
        }

        if self.node.is_none() {
            return Err(JsonPathError::EmptyPath);
        }

        let node = self.node.take().unwrap();
        self.visit(&node);
        self.node = Some(node);

        Ok(())
    }

    pub fn select_as<T: serde::de::DeserializeOwned>(&mut self) -> Result<Vec<T>, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(vec) => {
                let mut ret = Vec::new();
                for v in vec {
                    match T::deserialize(*v) {
                        Ok(v) => ret.push(v),
                        Err(e) => return Err(JsonPathError::Serde(e.to_string())),
                    }
                }
                Ok(ret)
            }
            _ => Err(JsonPathError::EmptyValue),
        }
    }

    pub fn select_as_str(&mut self) -> Result<String, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(r) => {
                Ok(serde_json::to_string(r).map_err(|e| JsonPathError::Serde(e.to_string()))?)
            }
            _ => Err(JsonPathError::EmptyValue),
        }
    }

    pub fn select(&mut self) -> Result<Vec<&'a Value>, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(r) => Ok(r.to_vec()),
            _ => Err(JsonPathError::EmptyValue),
        }
    }

    fn new_filter_context(&mut self) {
        self.terms.push(None);
        debug!("new_filter_context: {:?}", self.terms);
    }

    fn in_filter<F: Fn(&Vec<&'a Value>, &mut Vec<&'a Value>, &mut HashSet<usize>) -> FilterKey>(&mut self, fun: F) {
        fn get_parent<'a>(prev: Option<Vec<&'a Value>>, current_value: &[&'a Value], not_matched: HashSet<usize>) -> Option<Vec<&'a Value>> {
            if prev.is_some() {
                return prev;
            }

            let filtered: Vec<&Value> = current_value.iter().enumerate().filter(|(idx, _)| !not_matched.contains(idx))
                .map(|(_, v)| *v)
                .collect();

            Some(filtered)
        }


        if let Some(peek) = self.terms.pop() {
            match peek {
                Some(v) => {
                    debug!("in_filter 1.: {:?}", v);

                    match v {
                        ExprTerm::Json(rel, fk, vec) => {
                            let mut tmp = Vec::new();
                            let mut not_matched = HashSet::new();
                            let filter_key = if let Some(FilterKey::String(key)) = fk {
                                let key_contained = &vec.iter().map(|v| match v {
                                    Value::Object(map) if map.contains_key(&key) => map.get(&key).unwrap(),
                                    _ => v,
                                }).collect();
                                fun(key_contained, &mut tmp, &mut not_matched)
                            } else {
                                fun(&vec, &mut tmp, &mut not_matched)
                            };

                            let parent = get_parent(rel, &vec, not_matched);
                            self.terms.push(Some(ExprTerm::Json(parent, Some(filter_key), tmp)));
                        }
                        _ => unreachable!(),
                    };
                }
                _ => {
                    debug!("in_filter 2.: {:?}", &self.current);

                    if let Some(current) = &self.current {
                        let mut tmp = Vec::new();
                        let mut not_matched = HashSet::new();
                        let filter_key = fun(current, &mut tmp, &mut not_matched);
                        self.terms.push(Some(ExprTerm::Json(None, Some(filter_key), tmp)));
                    }
                }
            }
        }
    }

    fn all_in_filter_with_str(&mut self, key: &str) {
        self.in_filter(|vec, tmp, _| {
            ValueWalker::all_with_str(&vec, tmp, key, true);
            FilterKey::All
        });

        debug!("all_in_filter_with_str : {}, {:?}", key, self.terms);
    }

    fn next_in_filter_with_str(&mut self, key: &str) {
        fn _collect<'a>(
            v: &'a Value,
            tmp: &mut Vec<&'a Value>,
            key: &str,
            visited: &mut HashSet<*const Value>,
            not_matched: &mut HashSet<usize>,
        ) {
            match v {
                Value::Object(map) => {
                    if map.contains_key(key) {
                        let ptr = v as *const Value;
                        if !visited.contains(&ptr) {
                            visited.insert(ptr);
                            tmp.push(v)
                        }
                    }
                }
                Value::Array(vec) => {
                    for v in vec {
                        _collect(v, tmp, key, visited, not_matched);
                    }
                }
                _ => {}
            }
        }

        self.in_filter(|vec, tmp, not_matched| {
            let mut visited = HashSet::new();
            for (idx, v) in vec.iter().enumerate() {
                match v {
                    Value::Object(map) => {
                        if map.contains_key(key) {
                            let ptr = *v as *const Value;
                            if !visited.contains(&ptr) {
                                visited.insert(ptr);
                                tmp.push(v)
                            }
                        } else {
                            not_matched.insert(idx);
                        }
                    }
                    Value::Array(vec) => {
                        not_matched.insert(idx);
                        for v in vec {
                            _collect(v, tmp, key, &mut visited, not_matched);
                        }
                    }
                    _ => {
                        not_matched.insert(idx);
                    }
                }
            }

            FilterKey::String(key.to_owned())
        });

        debug!("next_in_filter_with_str : {}, {:?}", key, self.terms);
    }

    fn next_from_current_with_num(&mut self, index: f64) {
        fn _collect<'a>(tmp: &mut Vec<&'a Value>, vec: &'a [Value], index: f64) {
            let index = abs_index(index as isize, vec.len());
            if let Some(v) = vec.get(index) {
                tmp.push(v);
            }
        }

        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            for c in current {
                match c {
                    Value::Object(map) => {
                        for k in map.keys() {
                            if let Some(Value::Array(vec)) = map.get(k) {
                                _collect(&mut tmp, vec, index);
                            }
                        }
                    }
                    Value::Array(vec) => {
                        _collect(&mut tmp, vec, index);
                    }
                    _ => {}
                }
            }

            if tmp.is_empty() {
                self.terms.pop();
                self.current = Some(vec![&Value::Null]);
            } else {
                self.current = Some(tmp);
            }
        }

        debug!(
            "next_from_current_with_num : {:?}, {:?}",
            &index, self.current
        );
    }

    fn next_all_from_current(&mut self) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            for c in current {
                match c {
                    Value::Object(map) => {
                        for (_, v) in map {
                            tmp.push(v)
                        }
                    }
                    Value::Array(vec) => {
                        for v in vec {
                            tmp.push(v);
                        }
                    }
                    _ => {}
                }
            }
            self.current = Some(tmp);
        }

        debug!("next_all_from_current : {:?}", self.current);
    }

    fn next_from_current_with_str(&mut self, keys: &[String]) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            for c in current {
                if let Value::Object(map) = c {
                    for key in keys {
                        if let Some(v) = map.get(key) {
                            tmp.push(v)
                        }
                    }
                }
            }

            if tmp.is_empty() {
                self.current = Some(vec![&Value::Null]);
            } else {
                self.current = Some(tmp);
            }
        }

        debug!(
            "next_from_current_with_str : {:?}, {:?}",
            keys, self.current
        );
    }

    fn all_from_current(&mut self) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            ValueWalker::all(&current, &mut tmp);
            self.current = Some(tmp);
        }
        debug!("all_from_current: {:?}", self.current);
    }

    fn all_from_current_with_str(&mut self, key: &str) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            ValueWalker::all_with_str(&current, &mut tmp, key, false);
            self.current = Some(tmp);
        }
        debug!("all_from_current_with_str: {}, {:?}", key, self.current);
    }

    fn all_from_current_with_num(&mut self, index: f64) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            ValueWalker::all_with_num(&current, &mut tmp, index);
            self.current = Some(tmp);
        }
        debug!("all_from_current_with_num: {}, {:?}", index, self.current);
    }

    fn compute_absolute_path_filter(&mut self, token: &ParseToken) -> bool {
        if !self.selectors.is_empty() {
            match token {
                ParseToken::Absolute | ParseToken::Relative | ParseToken::Filter(_) => {
                    let selector = self.selectors.pop().unwrap();

                    if let Some(current) = &selector.current {
                        let term = current.into();

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

        if let Some(selector) = self.selectors.last_mut() {
            selector.visit_token(token);
            true
        } else {
            false
        }
    }
}

impl<'a, 'b> Selector<'a, 'b> {
    fn visit_absolute(&mut self) {
        if self.current.is_some() {
            let mut selector = Selector::default();

            if let Some(value) = self.value {
                selector.value = Some(value);
                selector.current = Some(vec![value]);
                self.selectors.push(selector);
            }
            return;
        }

        if let Some(v) = &self.value {
            self.current = Some(vec![v]);
        }
    }

    fn visit_relative(&mut self) {
        if let Some(ParseToken::Array) = self.tokens.last() {
            let array_token = self.tokens.pop();
            if let Some(ParseToken::Leaves) = self.tokens.last() {
                self.tokens.pop();
                self.all_from_current();
            }
            self.tokens.push(array_token.unwrap());
        }
        self.new_filter_context();
    }

    fn visit_array_eof(&mut self) {
        if self.is_last_before_token_match(ParseToken::Array) {
            if let Some(Some(e)) = self.terms.pop() {
                if let ExprTerm::String(key) = e {
                    self.next_in_filter_with_str(&key);
                    self.tokens.pop();
                    return;
                }

                self.terms.push(Some(e));
            }
        }

        if self.is_last_before_token_match(ParseToken::Leaves) {
            self.tokens.pop();
            self.tokens.pop();
            if let Some(Some(e)) = self.terms.pop() {
                if let ExprTerm::Number(n) = &e {
                    self.all_from_current_with_num(to_f64(n));
                    self.terms.pop();
                    return;
                }

                self.terms.push(Some(e));
            }
        }

        if let Some(Some(e)) = self.terms.pop() {
            match e {
                ExprTerm::Number(n) => {
                    self.next_from_current_with_num(to_f64(&n));
                }
                ExprTerm::String(key) => {
                    self.next_from_current_with_str(&[key]);
                }
                ExprTerm::Json(rel, _, v) => {
                    if v.is_empty() {
                        self.current = Some(vec![&Value::Null]);
                    } else if let Some(vec) = rel {
                        self.current = Some(vec);
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

    fn is_last_before_token_match(&mut self, token: ParseToken) -> bool {
        if self.tokens.len() > 1 {
            return token == self.tokens[self.tokens.len() - 2];
        }

        false
    }

    fn visit_all(&mut self) {
        if let Some(ParseToken::Array) = self.tokens.last() {
            self.tokens.pop();
        }

        match self.tokens.last() {
            Some(ParseToken::Leaves) => {
                self.tokens.pop();
                self.all_from_current();
            }
            Some(ParseToken::In) => {
                self.tokens.pop();
                self.next_all_from_current();
            }
            _ => {
                self.next_all_from_current();
            }
        }
    }

    fn visit_key(&mut self, key: &str) {
        if let Some(ParseToken::Array) = self.tokens.last() {
            self.terms.push(Some(ExprTerm::String(key.to_string())));
            return;
        }

        if let Some(t) = self.tokens.pop() {
            if self.terms.is_empty() {
                match t {
                    ParseToken::Leaves => self.all_from_current_with_str(key),
                    ParseToken::In => self.next_from_current_with_str(&[key.to_string()]),
                    _ => {}
                }
            } else {
                match t {
                    ParseToken::Leaves => {
                        self.all_in_filter_with_str(key);
                    }
                    ParseToken::In => {
                        self.next_in_filter_with_str(key);
                    }
                    _ => {}
                }
            }
        }
    }

    fn visit_keys(&mut self, keys: &[String]) {
        if !self.terms.is_empty() {
            unimplemented!("keys in filter");
        }

        if let Some(ParseToken::Array) = self.tokens.pop() {
            self.next_from_current_with_str(keys);
        } else {
            unreachable!();
        }
    }

    fn visit_filter(&mut self, ft: &FilterToken) {
        let right = match self.terms.pop() {
            Some(Some(right)) => right,
            Some(None) => ExprTerm::Json(
                None,
                None,
                match &self.current {
                    Some(current) => current.to_vec(),
                    _ => unreachable!(),
                },
            ),
            _ => panic!("empty term right"),
        };

        let left = match self.terms.pop() {
            Some(Some(left)) => left,
            Some(None) => ExprTerm::Json(
                None,
                None,
                match &self.current {
                    Some(current) => current.to_vec(),
                    _ => unreachable!(),
                },
            ),
            _ => panic!("empty term left"),
        };

        let mut ret = None;
        match ft {
            FilterToken::Equal => left.eq(&right, &mut ret),
            FilterToken::NotEqual => left.ne(&right, &mut ret),
            FilterToken::Greater => left.gt(&right, &mut ret),
            FilterToken::GreaterOrEqual => left.ge(&right, &mut ret),
            FilterToken::Little => left.lt(&right, &mut ret),
            FilterToken::LittleOrEqual => left.le(&right, &mut ret),
            FilterToken::And => left.and(&right, &mut ret),
            FilterToken::Or => left.or(&right, &mut ret),
        };

        if let Some(e) = ret {
            self.terms.push(Some(e));
        }
    }

    fn visit_range(&mut self, from: &Option<isize>, to: &Option<isize>, step: &Option<usize>) {
        if !self.terms.is_empty() {
            unimplemented!("range syntax in filter");
        }

        if let Some(ParseToken::Array) = self.tokens.pop() {
            let mut tmp = Vec::new();
            if let Some(current) = &self.current {
                for v in current {
                    if let Value::Array(vec) = v {
                        let from = if let Some(from) = from {
                            abs_index(*from, vec.len())
                        } else {
                            0
                        };

                        let to = if let Some(to) = to {
                            abs_index(*to, vec.len())
                        } else {
                            vec.len()
                        };

                        for i in (from..to).step_by(match step {
                            Some(step) => *step,
                            _ => 1,
                        }) {
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

    fn visit_union(&mut self, indices: &[isize]) {
        if !self.terms.is_empty() {
            unimplemented!("union syntax in filter");
        }

        if let Some(ParseToken::Array) = self.tokens.pop() {
            let mut tmp = Vec::new();
            if let Some(current) = &self.current {
                for v in current {
                    if let Value::Array(vec) = v {
                        for i in indices {
                            if let Some(v) = vec.get(abs_index(*i, vec.len())) {
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
}

impl<'a, 'b> NodeVisitor for Selector<'a, 'b> {
    fn visit_token(&mut self, token: &ParseToken) {
        debug!("token: {:?}, stack: {:?}", token, self.tokens);

        if self.compute_absolute_path_filter(token) {
            return;
        }

        match token {
            ParseToken::Absolute => self.visit_absolute(),
            ParseToken::Relative => self.visit_relative(),
            ParseToken::In | ParseToken::Leaves | ParseToken::Array => {
                self.tokens.push(token.clone());
            }
            ParseToken::ArrayEof => self.visit_array_eof(),
            ParseToken::All => self.visit_all(),
            ParseToken::Bool(b) => {
                self.terms.push(Some(ExprTerm::Bool(*b)));
            }
            ParseToken::Key(key) => self.visit_key(key),
            ParseToken::Keys(keys) => self.visit_keys(keys),
            ParseToken::Number(v) => {
                self.terms
                    .push(Some(ExprTerm::Number(Number::from_f64(*v).unwrap())));
            }
            ParseToken::Filter(ref ft) => self.visit_filter(ft),
            ParseToken::Range(from, to, step) => self.visit_range(from, to, step),
            ParseToken::Union(indices) => self.visit_union(indices),
            ParseToken::Eof => {
                debug!("visit_token eof");
            }
        }
    }
}

#[derive(Default)]
pub struct SelectorMut {
    path: Option<Node>,
    value: Option<Value>,
}

fn replace_value<F: FnMut(Value) -> Option<Value>>(
    mut tokens: Vec<String>,
    value: &mut Value,
    fun: &mut F,
) {
    let mut target = value;

    let last_index = tokens.len().saturating_sub(1);
    for (i, token) in tokens.drain(..).enumerate() {
        let target_once = target;
        let is_last = i == last_index;
        let target_opt = match *target_once {
            Value::Object(ref mut map) => {
                if is_last {
                    if let Entry::Occupied(mut e) = map.entry(token) {
                        let v = e.insert(Value::Null);
                        if let Some(res) = fun(v) {
                            e.insert(res);
                        } else {
                            e.remove();
                        }
                    }
                    return;
                }
                map.get_mut(&token)
            }
            Value::Array(ref mut vec) => {
                if let Ok(x) = token.parse::<usize>() {
                    if is_last {
                        let v = std::mem::replace(&mut vec[x], Value::Null);
                        if let Some(res) = fun(v) {
                            vec[x] = res;
                        } else {
                            vec.remove(x);
                        }
                        return;
                    }
                    vec.get_mut(x)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(t) = target_opt {
            target = t;
        } else {
            break;
        }
    }
}

impl SelectorMut {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn str_path(&mut self, path: &str) -> Result<&mut Self, JsonPathError> {
        self.path = Some(Parser::compile(path).map_err(JsonPathError::Path)?);
        Ok(self)
    }

    pub fn value(&mut self, value: Value) -> &mut Self {
        self.value = Some(value);
        self
    }

    pub fn take(&mut self) -> Option<Value> {
        self.value.take()
    }

    fn compute_paths(&self, mut result: Vec<&Value>) -> Vec<Vec<String>> {
        fn _walk(
            origin: &Value,
            target: &mut Vec<&Value>,
            tokens: &mut Vec<String>,
            visited: &mut HashSet<*const Value>,
            visited_order: &mut Vec<Vec<String>>,
        ) -> bool {
            trace!("{:?}, {:?}", target, tokens);

            if target.is_empty() {
                return true;
            }

            target.retain(|t| {
                if std::ptr::eq(origin, *t) {
                    if visited.insert(*t) {
                        visited_order.push(tokens.to_vec());
                    }
                    false
                } else {
                    true
                }
            });

            match origin {
                Value::Array(vec) => {
                    for (i, v) in vec.iter().enumerate() {
                        tokens.push(i.to_string());
                        if _walk(v, target, tokens, visited, visited_order) {
                            return true;
                        }
                        tokens.pop();
                    }
                }
                Value::Object(map) => {
                    for (k, v) in map {
                        tokens.push(k.clone());
                        if _walk(v, target, tokens, visited, visited_order) {
                            return true;
                        }
                        tokens.pop();
                    }
                }
                _ => {}
            }

            false
        }

        let mut visited = HashSet::new();
        let mut visited_order = Vec::new();

        if let Some(origin) = &self.value {
            let mut tokens = Vec::new();
            _walk(
                origin,
                &mut result,
                &mut tokens,
                &mut visited,
                &mut visited_order,
            );
        }

        visited_order
    }

    pub fn delete(&mut self) -> Result<&mut Self, JsonPathError> {
        self.replace_with(&mut |_| Some(Value::Null))
    }

    pub fn remove(&mut self) -> Result<&mut Self, JsonPathError> {
        self.replace_with(&mut |_| None)
    }

    fn select(&self) -> Result<Vec<&Value>, JsonPathError> {
        if let Some(node) = &self.path {
            let mut selector = Selector::default();
            selector.compiled_path(&node);

            if let Some(value) = &self.value {
                selector.value(value);
            }

            Ok(selector.select()?)
        } else {
            Err(JsonPathError::EmptyPath)
        }
    }

    pub fn replace_with<F: FnMut(Value) -> Option<Value>>(
        &mut self,
        fun: &mut F,
    ) -> Result<&mut Self, JsonPathError> {
        let paths = {
            let result = self.select()?;
            self.compute_paths(result)
        };

        if let Some(ref mut value) = &mut self.value {
            for tokens in paths {
                replace_value(tokens, value, fun);
            }
        }

        Ok(self)
    }
}


#[cfg(test)]
mod select_inner_tests {
    use serde_json::Value;

    #[test]
    fn to_f64_i64() {
        let number = 0_i64;
        let v: Value = serde_json::from_str(&format!("{}", number)).unwrap();
        if let Value::Number(n) = v {
            assert_eq!((super::to_f64(&n) - number as f64).abs() == 0_f64, true);
        } else {
            panic!();
        }
    }

    #[test]
    fn to_f64_f64() {
        let number = 0.1_f64;
        let v: Value = serde_json::from_str(&format!("{}", number)).unwrap();
        if let Value::Number(n) = v {
            assert_eq!((super::to_f64(&n) - number).abs() == 0_f64, true);
        } else {
            panic!();
        }
    }

    #[test]
    fn to_f64_u64() {
        let number = u64::max_value();
        let v: Value = serde_json::from_str(&format!("{}", number)).unwrap();
        if let Value::Number(n) = v {
            assert_eq!((super::to_f64(&n) - number as f64).abs() == 0_f64, true);
        } else {
            panic!();
        }
    }
}