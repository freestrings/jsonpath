use std::collections::HashSet;
use std::rc::Rc;

use serde_json::{Number, Value};
use serde_json::map::Entry;

use ::{JsonPathError, PathParser};
use paths::{
    _ParserTokenHandler,
    tokens::{
        _ParserToken,
        _TokenType,
        _TokenValue,
        constants::*,
    }
};

use super::terms::*;
use super::utils;

#[derive(Debug, Default)]
pub struct JsonSelector<'a, 'b> {
    parser: Option<Rc<PathParser<'a, 'b>>>,
    value: Option<&'a Value>,
    tokens: Vec<_ParserToken<'b>>,
    current: Option<Vec<&'a Value>>,
    selectors: Vec<JsonSelector<'a, 'b>>,
    selector_filter: FilterTerms<'a>,
}

impl<'a, 'b> JsonSelector<'a, 'b> {
    pub fn new(parser: PathParser<'a, 'b>) -> Self {
        JsonSelector {
            parser: Some(Rc::new(parser)),
            value: None,
            tokens: Vec::new(),
            current: None,
            selectors: Vec::new(),
            selector_filter: FilterTerms(Vec::new()),
        }
    }

    pub fn new_ref(parser: Rc<PathParser<'a, 'b>>) -> Self {
        JsonSelector {
            parser: Some(parser),
            value: None,
            tokens: Vec::new(),
            current: None,
            selectors: Vec::new(),
            selector_filter: FilterTerms(Vec::new()),
        }
    }

    pub fn reset_parser(&mut self, parser: PathParser<'a, 'b>) -> &mut Self {
        self.parser = Some(Rc::new(parser));
        self
    }

    pub fn reset_parser_ref(&mut self, parser: Rc<PathParser<'a, 'b>>) -> &mut Self {
        self.parser = Some(parser);
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
        let parser = self.parser.take();
        if let Some(parser) = parser.as_ref() {
            let _ = parser.parse(self);
        }
        self.parser = parser;

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

    fn compute_absolute_path_filter<F>(&mut self, token: &_ParserToken<'b>, parse_value_reader: &F) -> bool
        where
            F: Fn(&'_ _TokenType) -> _TokenValue<'a>
    {
        if !self.selectors.is_empty() {
            match &token.key {
                &P_TOK_ABSOLUTE
                | &P_TOK_RELATIVE
                | &P_TOK_FILTER_OR
                | &P_TOK_FILTER_AND
                | &P_TOK_FILTER_GREATER
                | &P_TOK_FILTER_EQUAL
                | &P_TOK_FILTER_LITTLE
                | &P_TOK_FILTER_GREATER_OR_EQUAL
                | &P_TOK_FILTER_LITTLE_OR_EQUAL
                | &P_TOK_FILTER_NOT_EQUAL
                => {
                    let selector = self.selectors.pop().unwrap();

                    if let Some(current) = &selector.current {
                        let term = current.into();

                        if let Some(s) = self.selectors.last_mut() {
                            s.selector_filter.push_term(Some(term));
                        } else {
                            self.selector_filter.push_term(Some(term));
                        }
                    } else {
                        unreachable!()
                    }
                }
                _ => {}
            }
        }

        if self.selectors.is_empty() {
            return false;
        }

        self.selectors.last_mut().unwrap().handle(token, parse_value_reader);
        true
    }
}

impl<'a, 'b> _ParserTokenHandler<'a, 'b> for JsonSelector<'a, 'b> {
    fn handle<F>(&mut self, token: &_ParserToken<'b>, parse_value_reader: &F) where F: Fn(&_TokenType) -> _TokenValue<'a> {
        debug!("token: {:?}, stack: {:?}", token, self.tokens);

        match &token {
            _ParserToken { key: P_TOK_ABSOLUTE, .. } => {
                if self.current.is_some() {
                    if let Some(value) = self.value {
                        let selector = JsonSelector {
                            parser: None,
                            value: Some(value),
                            tokens: Vec::new(),
                            current: Some(vec![value]),
                            selectors: Vec::new(),
                            selector_filter: FilterTerms(Vec::new()),
                        };
                        self.selectors.push(selector);
                    }
                    return;
                }

                if let Some(v) = &self.value {
                    self.current = Some(vec![v]);
                }
            }
            _ParserToken { key: P_TOK_RELATIVE, .. } => {
                if let Some(_ParserToken { key: P_TOK_ARRAY, .. }) = self.tokens.last() {
                    let array_token = self.tokens.pop();
                    if let Some(_ParserToken { key: P_TOK_LEAVES, .. }) = self.tokens.last() {
                        self.tokens.pop();
                        self.current = self.selector_filter.collect_all(self.current.take());
                    }
                    self.tokens.push(array_token.unwrap());
                }
                self.selector_filter.new_filter_context();
            }
            _ParserToken { key: P_TOK_IN, .. }
            | _ParserToken { key: P_TOK_LEAVES, .. }
            | _ParserToken { key: P_TOK_ARRAY, .. } => {
                self.tokens.push(token.clone());
            }
            _ParserToken { key: P_TOK_ARRAY_END, .. } => {
                fn get_last_before<'a>(tokens: &'a Vec<_ParserToken<'a>>) -> Option<&'a _ParserToken<'a>> {
                    if tokens.len() > 1 {
                        tokens.get(tokens.len() - 2)
                    } else {
                        None
                    }
                }
                if let Some(&_ParserToken { key: P_TOK_ARRAY, .. }) = get_last_before(&self.tokens) {
                    if let Some(Some(e)) = self.selector_filter.pop_term() {
                        if let ExprTerm::String(key) = e {
                            self.current = self.selector_filter.filter_next_with_str(self.current.take(), key);
                            self.tokens.pop();
                            return;
                        }

                        self.selector_filter.push_term(Some(e));
                    }
                }

                if let Some(&_ParserToken { key: P_TOK_LEAVES, .. }) = get_last_before(&self.tokens) {
                    self.tokens.pop();
                    self.tokens.pop();
                    if let Some(Some(e)) = self.selector_filter.pop_term() {
                        let selector_filter_consumed = match e {
                            ExprTerm::Number(n) => {
                                self.current = self.selector_filter.collect_all_with_num(self.current.take(), utils::to_f64(&n));
                                self.selector_filter.pop_term();
                                true
                            }
                            ExprTerm::String(key) => {
                                self.current = self.selector_filter.collect_all_with_str(self.current.take(), key);
                                self.selector_filter.pop_term();
                                true
                            }
                            _ => {
                                self.selector_filter.push_term(Some(e));
                                false
                            }
                        };

                        if selector_filter_consumed {
                            return;
                        }
                    }
                }

                if let Some(Some(e)) = self.selector_filter.pop_term() {
                    match e {
                        ExprTerm::Number(n) => {
                            self.current = self.selector_filter.collect_next_with_num(self.current.take(), utils::to_f64(&n));
                        }
                        ExprTerm::String(key) => {
                            self.current = self.selector_filter.collect_next_with_str(self.current.take(), &[key]);
                        }
                        ExprTerm::Json(rel, _, v) => {
                            if v.is_empty() {
                                self.current = Some(Vec::new());
                            } else if let Some(vec) = rel {
                                self.current = Some(vec);
                            } else {
                                self.current = Some(v);
                            }
                        }
                        ExprTerm::Bool(false) => {
                            self.current = Some(vec![]);
                        }
                        _ => {}
                    }
                }

                self.tokens.pop();
            }
            _ParserToken { key: P_TOK_ALL, .. } => {
                if let Some(_ParserToken { key: P_TOK_ARRAY, .. }) = self.tokens.last() {
                    self.tokens.pop();
                }

                match self.tokens.last() {
                    Some(_ParserToken { key: P_TOK_LEAVES, .. }) => {
                        self.tokens.pop();
                        self.current = self.selector_filter.collect_all(self.current.take());
                    }
                    Some(_ParserToken { key: P_TOK_IN, .. }) => {
                        self.tokens.pop();
                        self.current = self.selector_filter.collect_next_all(self.current.take());
                    }
                    _ => {
                        self.current = self.selector_filter.collect_next_all(self.current.take());
                    }
                }
            }
            _ParserToken { key: P_TOK_BOOL, token_type } => {
                if let Some(types) = token_type {
                    assert_eq!(types.len(), 1, "Invalid bool token size: {}", types.len());
                    if let Some(b) = parse_value_reader(&types[0]).get_bool() {
                        self.selector_filter.push_term(Some(ExprTerm::Bool(b)));
                    } else {
                        panic!("Not a bool {:?}", token_type);
                    }
                } else {
                    panic!("Empty bool token value");
                }

                // self.selector_filter.push_term(Some(ExprTerm::Bool(*b)));
            }
            _ParserToken { key: P_TOK_KEY, token_type } => {
                if let Some(types) = token_type {
                    assert_eq!(types.len(), 1, "Invalid key token size: {}", types.len());

                    let key = if let Some(v) = parse_value_reader(&types[0]).get_str() {
                        v
                    } else {
                        panic!("Not a string {:?}", types)
                    };

                    if let Some(_ParserToken { key: P_TOK_ARRAY, .. }) = self.tokens.last() {
                        self.selector_filter.push_term(Some(ExprTerm::String(key)));
                        return;
                    }

                    if let Some(t) = self.tokens.pop() {
                        if self.selector_filter.is_term_empty() {
                            match t {
                                _ParserToken { key: P_TOK_LEAVES, .. } => {
                                    self.current = self.selector_filter.collect_all_with_str(self.current.take(), key)
                                }
                                _ParserToken { key: P_TOK_IN, .. } => {
                                    self.current = self.selector_filter.collect_next_with_str(self.current.take(), &[key])
                                }
                                _ => {}
                            }
                        } else {
                            match t {
                                _ParserToken { key: P_TOK_LEAVES, .. } => {
                                    self.current = self.selector_filter.filter_all_with_str(self.current.take(), key);
                                }
                                _ParserToken { key: P_TOK_IN, .. } => {
                                    self.current = self.selector_filter.filter_next_with_str(self.current.take(), key);
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    panic!("Empty key token value");
                }
            }
            _ParserToken { key: P_TOK_KEYS, token_type, .. } => {
                if !self.selector_filter.is_term_empty() {
                    debug!("TODO keys in filter");
                    return;
                }

                if let Some(ranges) = token_type {
                    let keys: Vec<&str> = ranges.iter().map(|r| parse_value_reader(r).get_str())
                        .filter(|v| v.is_some())
                        .map(|v| v.unwrap())
                        .collect();

                    assert_ne!(ranges.len(), 0, "Invalid keys token size: {}", ranges.len());
                    assert_eq!(ranges.len(), keys.len(), "Invalid keys token value: {:?} {:?}", ranges, keys);

                    if let Some(_ParserToken { key: P_TOK_ARRAY, .. }) = self.tokens.pop() {
                        self.current = self.selector_filter.collect_next_with_str(self.current.take(), &keys);
                    } else {
                        unreachable!();
                    }
                } else {
                    panic!("Empty keys token value");
                }
            }
            _ParserToken { key: P_TOK_NUMBER, token_type } => {
                if let Some(types) = token_type {
                    assert_eq!(types.len(), 1, "Invalid number token size: {}", types.len());
                    let v = parse_value_reader(&types[0]);
                    // FIXME
                    match v {
                        _TokenValue::Int(v) => {
                            self.selector_filter.push_term(Some(ExprTerm::Number(Number::from(v))));
                        }
                        _TokenValue::Float(v) => {
                            self.selector_filter.push_term(Some(ExprTerm::Number(Number::from_f64(v).unwrap())));
                        }
                        _ => panic!("Not a number")
                    }
                } else {
                    panic!("Empty number token value");
                }
            }
            _ParserToken { key: P_TOK_FILTER_AND, .. }
            | _ParserToken { key: P_TOK_FILTER_EQUAL, .. }
            | _ParserToken { key: P_TOK_FILTER_GREATER, .. }
            | _ParserToken { key: P_TOK_FILTER_GREATER_OR_EQUAL, .. }
            | _ParserToken { key: P_TOK_FILTER_LITTLE, .. }
            | _ParserToken { key: P_TOK_FILTER_LITTLE_OR_EQUAL, .. }
            | _ParserToken { key: P_TOK_FILTER_NOT_EQUAL, .. }
            | _ParserToken { key: P_TOK_FILTER_OR, .. } => {
                let right = match self.selector_filter.pop_term() {
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

                let mut left = match self.selector_filter.pop_term() {
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

                let expr = match &token.key {
                    &P_TOK_FILTER_AND => left.and(right),
                    &P_TOK_FILTER_EQUAL => left.eq_(right),
                    &P_TOK_FILTER_GREATER => left.gt(right),
                    &P_TOK_FILTER_GREATER_OR_EQUAL => left.ge(right),
                    &P_TOK_FILTER_LITTLE => left.lt(right),
                    &P_TOK_FILTER_LITTLE_OR_EQUAL => left.le(right),
                    &P_TOK_FILTER_NOT_EQUAL => left.le(right),
                    &P_TOK_FILTER_OR => left.or(right),
                    _ => panic!("Unexpected operator {}", &token.key),
                };

                self.selector_filter.push_term(Some(expr));
            }
            _ParserToken { key: P_TOK_RANGE, token_type }
            | _ParserToken { key: P_TOK_RANGE_TO, token_type }
            | _ParserToken { key: P_TOK_RANGE_FROM, token_type } => {
                if !self.selector_filter.is_term_empty() {
                    debug!("TODO range syntax in filter");
                    return;
                }

                let as_int = |t| -> isize {
                    if let _TokenValue::Int(v) = parse_value_reader(t) {
                        v
                    } else {
                        panic!("expected int {:?}", &t)
                    }
                };

                if let Some(_ParserToken { key: P_TOK_ARRAY, .. }) = self.tokens.pop() {
                    let mut tmp = Vec::new();
                    if let Some(current) = &self.current {
                        for v in current {
                            if let Value::Array(vec) = v {
                                let params = if let Some(types) = token_type {
                                    match &token.key {
                                        // step
                                        //  0. $[::]
                                        //  1. $[::2]
                                        //  2. $[:3:2]
                                        //  3. $[0:3:2]
                                        &P_TOK_RANGE => {
                                            match types.len() {
                                                0 => {
                                                    (0, vec.len(), 1)
                                                }
                                                1 => {
                                                    (
                                                        0,
                                                        vec.len(),
                                                        as_int(&types[0]) as usize // FIXME
                                                    )
                                                }
                                                2 => {
                                                    (
                                                        0,
                                                        utils::abs_index(as_int(&types[0]), vec.len()),
                                                        as_int(&types[1]) as usize // FIXME
                                                    )
                                                }
                                                3 => {
                                                    (
                                                        utils::abs_index(as_int(&types[0]), vec.len()),
                                                        utils::abs_index(as_int(&types[1]), vec.len()),
                                                        as_int(&types[2]) as usize // FIXME
                                                    )
                                                }
                                                _ => panic!("Unexpected range param types")
                                            }
                                        }
                                        // to
                                        //  0. $[:]
                                        //  1. $.a[:11]
                                        //  2. $.a[-12:13]
                                        &P_TOK_RANGE_TO => {
                                            match types.len() {
                                                0 => {
                                                    (0, vec.len(), 1)
                                                }
                                                1 => {
                                                    (
                                                        0,
                                                        utils::abs_index(as_int(&types[0]), vec.len()),
                                                        1
                                                    )
                                                }
                                                2 => {
                                                    (
                                                        utils::abs_index(as_int(&types[0]), vec.len()),
                                                        utils::abs_index(as_int(&types[1]), vec.len()),
                                                        1
                                                    )
                                                }
                                                _ => panic!("Unexpected range_to param types")
                                            }
                                        }
                                        // from
                                        //  1. $.a[10:]
                                        //  2. $[1::2]
                                        &P_TOK_RANGE_FROM => {
                                            match types.len() {
                                                1 => {
                                                    (
                                                        utils::abs_index(as_int(&types[0]), vec.len()),
                                                        vec.len(),
                                                        1
                                                    )
                                                }
                                                2 => {
                                                    (
                                                        utils::abs_index(as_int(&types[0]), vec.len()),
                                                        vec.len(),
                                                        as_int(&types[1]) as usize // FIXME
                                                    )
                                                }
                                                _ => panic!("Unexpected range_from param types")
                                            }
                                        }
                                        _ => {
                                            panic!("Unexpected range type {}", &token.key);
                                        }
                                    }
                                } else {
                                    (0, vec.len(), 1)
                                };

                                let (from, to, step) = params;

                                for i in (from..to).step_by(step) {
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
            _ParserToken { key: P_TOK_UNION, token_type } => {
                if !self.selector_filter.is_term_empty() {
                    debug!("TODO union syntax in filter");
                    return;
                }

                if token_type.is_none() {
                    panic!("Unexpected range params");
                }

                let mut indices: Option<Vec<isize>> = None;

                if let Some(_ParserToken { key: P_TOK_ARRAY, .. }) = self.tokens.pop() {
                    let mut tmp = Vec::new();
                    if let Some(current) = &self.current {
                        for v in current {
                            if let Value::Array(vec) = v {
                                if indices.is_none() {
                                    indices = Some(if let Some(types) = token_type {
                                        let values: Vec<isize> = types.iter()
                                            .map(|t| parse_value_reader(t).as_int())
                                            .filter(|v| v.is_some())
                                            .map(|v| v.unwrap())
                                            .collect();

                                        if values.len() != types.len() {
                                            // TODO
                                            debug!("Unexpected data types: {:?}", types);
                                        }
                                        values
                                    } else {
                                        Vec::new()
                                    });
                                }

                                if let Some(indices) = &indices {
                                    for i in indices {
                                        if let Some(v) = vec.get(utils::abs_index(*i, vec.len())) {
                                            tmp.push(v);
                                        }
                                    }
                                }
                            } else if let Value::Object(_) = v {
                                // TODO
                            }
                        }
                    }

                    self.current = Some(tmp);
                } else {
                    unreachable!();
                }
            }
            _ParserToken { key: P_TOK_END, .. } => {
                debug!("visit_token eof");
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct JsonSelectorMut<'a, 'b> {
    value: Option<Value>,
    parser: Option<Rc<PathParser<'a, 'b>>>,
}

impl<'a, 'b> JsonSelectorMut<'a, 'b> {
    pub fn new(parser: PathParser<'a, 'b>) -> Self {
        Self::new_ref(Rc::new(parser))
    }

    pub fn new_ref(parser: Rc<PathParser<'a, 'b>>) -> Self {
        JsonSelectorMut {
            value: None,
            parser: Some(parser),
        }
    }

    pub fn reset_parser(&mut self, parser: PathParser<'a, 'b>) -> &mut Self {
        self.parser = Some(Rc::new(parser));
        self
    }

    pub fn reset_parser_ref(&mut self, parser: Rc<PathParser<'a, 'b>>) -> &mut Self {
        self.parser = Some(parser);
        self
    }

    pub fn value(&mut self, value: Value) -> &mut Self {
        self.value = Some(value);
        self
    }

    pub fn take(&mut self) -> Option<Value> {
        self.value.take()
    }

    pub fn delete(&mut self) -> Result<&mut Self, JsonPathError> {
        self.replace_with(&mut |_| Some(Value::Null))
    }

    pub fn remove(&mut self) -> Result<&mut Self, JsonPathError> {
        self.replace_with(&mut |_| None)
    }

    fn select(&self) -> Result<Vec<&Value>, JsonPathError> {
        let mut selector = JsonSelector::default();

        if let Some(parser) = self.parser.as_ref() {
            selector.reset_parser_ref(Rc::clone(parser));
        } else {
            return Err(JsonPathError::EmptyPath);
        }

        if let Some(value) = self.value.as_ref() {
            selector.value(value);
        } else {
            return Err(JsonPathError::EmptyValue);
        }

        selector.select()
    }

    pub fn replace_with<F>(&mut self, fun: &mut F) -> Result<&mut Self, JsonPathError>
        where
            F: FnMut(Value) -> Option<Value>,
    {
        let result = self.select()?;
        let paths = self.compute_paths(result);

        if let Some(ref mut value) = &mut self.value {
            for tokens in paths {
                Self::replace_value(tokens, value, fun);
            }
        }

        Ok(self)
    }

    fn replace_value<F>(mut tokens: Vec<String>, value: &mut Value, fun: &mut F)
        where
            F: FnMut(Value) -> Option<Value>
    {
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
                            if x < vec.len() {
                                let v = std::mem::replace(&mut vec[x], Value::Null);
                                if let Some(res) = fun(v) {
                                    vec[x] = res;
                                } else {
                                    vec.remove(x);
                                }
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

    fn compute_paths(&self, mut result: Vec<&Value>) -> Vec<Vec<String>> {
        let mut visited = HashSet::new();
        let mut visited_order = Vec::new();

        if let Some(origin) = &self.value {
            let mut tokens = Vec::new();
            Self::walk(
                origin,
                &mut result,
                &mut tokens,
                &mut visited,
                &mut visited_order,
            );
        }

        visited_order
    }

    fn walk(
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
                    if Self::walk(v, target, tokens, visited, visited_order) {
                        return true;
                    }
                    tokens.pop();
                }
            }
            Value::Object(map) => {
                for (k, v) in map {
                    tokens.push(k.clone());
                    if Self::walk(v, target, tokens, visited, visited_order) {
                        return true;
                    }
                    tokens.pop();
                }
            }
            _ => {}
        }

        false
    }
}