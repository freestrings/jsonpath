use std::collections::HashSet;

use serde_json::{Number, Value};

use super::cmp::*;
use super::utils;
use super::value_walker::ValueWalker;

#[derive(Debug, PartialEq)]
pub enum ExprTerm<'a> {
    String(&'a str),
    Number(Number),
    Bool(bool),
    Json(Option<Vec<&'a Value>>, Option<FilterKey<'a>>, Vec<&'a Value>),
}

impl<'a> ExprTerm<'a> {
    fn cmp_string<C1, C2>(&self, s1: &str, other: &ExprTerm<'a>, cmp_fn: &C1, rev_cmp_fn: &C2) -> ExprTerm<'a>
        where
            C1: Cmp,
            C2: Cmp
    {
        match other {
            ExprTerm::String(s2) => {
                let (s1, opt1) = utils::to_path_str(s1);
                let (s2, opt2) = utils::to_path_str(s2);
                let k1 = if let Some(opt) = opt1.as_ref() { opt } else { s1 };
                let k2 = if let Some(opt) = opt2.as_ref() { opt } else { s2 };
                ExprTerm::Bool(cmp_fn.cmp_string(k1, k2))
            }
            ExprTerm::Json(_, _, _) => other.cmp(self, rev_cmp_fn, cmp_fn),
            _ => ExprTerm::Bool(cmp_fn.default()),
        }
    }

    fn cmp_number<C1, C2>(&self, n1: &Number, other: &ExprTerm<'a>, cmp_fn: &C1, rev_cmp_fn: &C2) -> ExprTerm<'a>
        where
            C1: Cmp,
            C2: Cmp
    {
        match other {
            ExprTerm::Number(n2) => ExprTerm::Bool(cmp_fn.cmp_f64(utils::to_f64(n1), utils::to_f64(n2))),
            ExprTerm::Json(_, _, _) => other.cmp(self, rev_cmp_fn, cmp_fn),
            _ => ExprTerm::Bool(cmp_fn.default()),
        }
    }

    fn cmp_bool<C1, C2>(&self, b1: &bool, other: &ExprTerm<'a>, cmp_fn: &C1, rev_cmp_fn: &C2) -> ExprTerm<'a>
        where
            C1: Cmp,
            C2: Cmp
    {
        match other {
            ExprTerm::Bool(b2) => ExprTerm::Bool(cmp_fn.cmp_bool(*b1, *b2)),
            ExprTerm::Json(_, _, _) => other.cmp(self, rev_cmp_fn, cmp_fn),
            _ => ExprTerm::Bool(cmp_fn.default()),
        }
    }

    fn cmp_json_string<C1>(&self, s2: &str, fk1: &Option<FilterKey>, vec1: &Vec<&'a Value>, cmp_fn: &C1) -> Vec<&'a Value>
        where
            C1: Cmp
    {
        let (s2, opt2) = utils::to_path_str(s2);
        vec1.iter().filter(|v1| match v1 {
            Value::String(s1) => {
                if let Some(opt) = opt2.as_ref() {
                    cmp_fn.cmp_string(s1, opt)
                } else {
                    cmp_fn.cmp_string(s1, s2)
                }
            }
            Value::Object(map1) => {
                if let Some(FilterKey::String(k)) = fk1 {
                    if let Some(Value::String(s1)) = map1.get(*k) {
                        return if let Some(opt) = opt2.as_ref() {
                            cmp_fn.cmp_string(s1, opt)
                        } else {
                            cmp_fn.cmp_string(s1, s2)
                        };
                    }
                }
                cmp_fn.default()
            }
            _ => cmp_fn.default(),
        }).copied().collect()
    }

    fn cmp_json_number<C1>(&self, n2: &Number, fk1: &Option<FilterKey>, vec1: &Vec<&'a Value>, cmp_fn: &C1) -> Vec<&'a Value>
        where
            C1: Cmp
    {
        vec1.iter().filter(|v1| match v1 {
            Value::Number(n1) => cmp_fn.cmp_f64(utils::to_f64(n1), utils::to_f64(n2)),
            Value::Object(map1) => {
                if let Some(FilterKey::String(k)) = fk1 {
                    if let Some(Value::Number(n1)) = map1.get(*k) {
                        return cmp_fn.cmp_f64(utils::to_f64(n1), utils::to_f64(n2));
                    }
                }
                cmp_fn.default()
            }
            _ => cmp_fn.default(),
        }).copied().collect()
    }

    fn cmp_json_bool<C1>(&self, b2: &bool, fk1: &Option<FilterKey>, vec1: &Vec<&'a Value>, cmp_fn: &C1) -> Vec<&'a Value>
        where
            C1: Cmp
    {
        vec1.iter().filter(|v1| match v1 {
            Value::Bool(b1) => cmp_fn.cmp_bool(*b1, *b2),
            Value::Object(map1) => {
                if let Some(FilterKey::String(k)) = fk1 {
                    if let Some(Value::Bool(b1)) = map1.get(*k) {
                        return cmp_fn.cmp_bool(*b1, *b2);
                    }
                }
                cmp_fn.default()
            }
            _ => cmp_fn.default(),
        }).copied().collect()
    }

    fn cmp_json_json<C1>(&self, rel: &Option<Vec<&'a Value>>, parent: &Option<Vec<&'a Value>>, vec1: &Vec<&'a Value>, vec2: &Vec<&'a Value>, cmp_fn: &C1) -> Vec<&'a Value>
        where
            C1: Cmp
    {
        if let Some(vec1) = rel {
            cmp_fn.cmp_json(vec1, vec2)
        } else if let Some(vec2) = parent {
            cmp_fn.cmp_json(vec1, vec2)
        } else {
            cmp_fn.cmp_json(vec1, vec2)
        }
    }

    fn cmp_json<C1>(&self, rel: &Option<Vec<&'a Value>>, fk1: &Option<FilterKey>, vec1: &Vec<&'a Value>, other: &ExprTerm<'a>, cmp_fn: &C1) -> ExprTerm<'a>
        where
            C1: Cmp
    {
        let ret: Vec<&Value> = match other {
            ExprTerm::String(s2) => self.cmp_json_string(s2, fk1, vec1, cmp_fn),
            ExprTerm::Number(n2) => self.cmp_json_number(n2, fk1, vec1, cmp_fn),
            ExprTerm::Bool(b2) => self.cmp_json_bool(b2, fk1, vec1, cmp_fn),
            ExprTerm::Json(parent, _, vec2) => self.cmp_json_json(rel, parent, vec1, vec2, cmp_fn)
        };

        if ret.is_empty() {
            return ExprTerm::Bool(cmp_fn.default());
        }

        if let Some(rel) = rel {
            if let ExprTerm::Json(_, _, _) = &other {
                return ExprTerm::Json(Some(rel.to_vec()), None, ret);
            } else {
                let mut object_exist = false;
                for v in rel {
                    if v.is_object() {
                        object_exist = true;
                        break;
                    }
                }

                if !object_exist {
                    return ExprTerm::Json(Some(Vec::new()), None, ret);
                }

                let ret_set: HashSet<*const Value> = ret.iter()
                    .fold(HashSet::new(), |mut acc, v| {
                        let ptr = *v as *const Value;
                        acc.insert(ptr);
                        acc
                    });

                let mut tmp = Vec::new();
                for rv in rel {
                    if let Value::Object(map) = rv {
                        for map_value in map.values() {
                            let ptr = map_value as *const Value;
                            if ret_set.contains(&ptr) {
                                tmp.push(*rv);
                            }
                        }
                    }
                }

                return ExprTerm::Json(Some(tmp), None, ret);
            }
        }

        ExprTerm::Json(None, None, ret)
    }

    fn cmp<C1, C2>(&self, other: &Self, cmp_fn: &C1, rev_cmp_fn: &C2) -> ExprTerm<'a>
        where
            C1: Cmp,
            C2: Cmp
    {
        match &self {
            ExprTerm::String(s1) => self.cmp_string(s1, other, cmp_fn, rev_cmp_fn),
            ExprTerm::Number(n1) => self.cmp_number(n1, other, cmp_fn, rev_cmp_fn),
            ExprTerm::Bool(b1) => self.cmp_bool(b1, other, cmp_fn, rev_cmp_fn),
            ExprTerm::Json(rel, fk1, vec1) =>
                self.cmp_json(rel, fk1, vec1, other, cmp_fn)
        }
    }

    pub fn eq(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("eq - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpEq, &CmpEq);
        debug!("eq = {:?}", tmp);
        *ret = Some(tmp);
    }

    pub fn ne(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("ne - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpNe, &CmpNe);
        debug!("ne = {:?}", tmp);
        *ret = Some(tmp);
    }

    pub fn gt(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("gt - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpGt, &CmpLt);
        debug!("gt = {:?}", tmp);
        *ret = Some(tmp);
    }

    pub fn ge(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("ge - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpGe, &CmpLe);
        debug!("ge = {:?}", tmp);
        *ret = Some(tmp);
    }

    pub fn lt(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("lt - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpLt, &CmpGt);
        debug!("lt = {:?}", tmp);
        *ret = Some(tmp);
    }

    pub fn le(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("le - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpLe, &CmpGe);
        debug!("le = {:?}", tmp);
        *ret = Some(tmp);
    }

    pub fn and(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("and - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpAnd, &CmpAnd);
        debug!("and = {:?}", tmp);
        *ret = Some(tmp);
    }

    pub fn or(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("or - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpOr, &CmpOr);
        debug!("or = {:?}", tmp);
        *ret = Some(tmp);
    }
}

impl<'a> From<&Vec<&'a Value>> for ExprTerm<'a> {
    fn from(vec: &Vec<&'a Value>) -> Self {
        if vec.len() == 1 {
            match &vec[0] {
                Value::Number(v) => return ExprTerm::Number(v.clone()),
                Value::String(v) => return ExprTerm::String(v.as_str()),
                Value::Bool(v) => return ExprTerm::Bool(*v),
                _ => {}
            }
        }

        ExprTerm::Json(None, None, vec.to_vec())
    }
}

#[derive(Debug, PartialEq)]
pub enum FilterKey<'a> {
    String(&'a str),
    All,
}

#[derive(Debug, Default)]
pub struct FilterTerms<'a>(pub Vec<Option<ExprTerm<'a>>>);

impl<'a> FilterTerms<'a> {
    pub fn new_filter_context(&mut self) {
        self.0.push(None);
        debug!("new_filter_context: {:?}", self.0);
    }

    pub fn is_term_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push_term(&mut self, term: Option<ExprTerm<'a>>) {
        self.0.push(term);
    }

    #[allow(clippy::option_option)]
    pub fn pop_term(&mut self) -> Option<Option<ExprTerm<'a>>> {
        self.0.pop()
    }

    pub fn filter_json_term<F>(&mut self, e: ExprTerm<'a>, fun: F)
        where
            F: Fn(Vec<&'a Value>, &mut Option<HashSet<usize>>) -> (FilterKey<'a>, Vec<&'a Value>),
    {
        debug!("filter_json_term: {:?}", e);

        if let ExprTerm::Json(rel, fk, vec) = e {
            let mut not_matched = Some(HashSet::new());
            let (filter_key, collected) = if let Some(FilterKey::String(key)) = fk {
                let tmp = vec.iter().map(|v| match v {
                    Value::Object(map) if map.contains_key(key) => map.get(key).unwrap(),
                    _ => v
                }).collect();
                fun(tmp, &mut not_matched)
            } else {
                fun(vec.to_vec(), &mut not_matched)
            };

            if rel.is_some() {
                self.push_term(Some(ExprTerm::Json(rel, Some(filter_key), collected)));
            } else {
                let not_matched = not_matched.unwrap();
                let filtered = vec.iter().enumerate()
                    .filter(|(idx, _)| !not_matched.contains(idx))
                    .map(|(_, v)| *v).collect();
                self.push_term(Some(ExprTerm::Json(Some(filtered), Some(filter_key), collected)));
            }
        } else {
            unreachable!("unexpected: ExprTerm: {:?}", e);
        }
    }

    pub fn push_json_term<F>(&mut self, current: Option<Vec<&'a Value>>, fun: F) -> Option<Vec<&'a Value>>
        where
            F: Fn(Vec<&'a Value>, &mut Option<HashSet<usize>>) -> (FilterKey<'a>, Vec<&'a Value>),
    {
        debug!("push_json_term: {:?}", &current);

        if let Some(current) = &current {
            let (filter_key, collected) = fun(current.to_vec(), &mut None);
            self.push_term(Some(ExprTerm::Json(None, Some(filter_key), collected)));
        }

        current
    }

    pub fn filter<F>(&mut self, current: Option<Vec<&'a Value>>, fun: F) -> Option<Vec<&'a Value>>
        where
            F: Fn(Vec<&'a Value>, &mut Option<HashSet<usize>>) -> (FilterKey<'a>, Vec<&'a Value>),
    {
        let peek = self.pop_term();

        if let Some(None) = peek {
            return self.push_json_term(current, fun);
        }

        if let Some(Some(e)) = peek {
            self.filter_json_term(e, fun);
        }

        current
    }

    pub fn filter_all_with_str(&mut self, current: Option<Vec<&'a Value>>, key: &'a str) -> Option<Vec<&'a Value>> {
        let current = self.filter(current, |vec, _| {
            (FilterKey::All, ValueWalker::all_with_str(vec, key))
        });

        debug!("filter_all_with_str : {}, {:?}", key, self.0);
        current
    }

    pub fn filter_next_with_str(&mut self, current: Option<Vec<&'a Value>>, key: &'a str) -> Option<Vec<&'a Value>> {
        let current = self.filter(current, |vec, not_matched| {
            let mut visited = HashSet::new();
            let mut acc = Vec::new();
            let (key, opt) = utils::to_path_str(key);
            let k = if let Some(opt) = opt.as_ref() { opt } else { key };
            vec.iter().enumerate().for_each(|(idx, v)| {
                match v {
                    Value::Object(map) => {
                        if map.contains_key(k) {
                            let ptr = *v as *const Value;
                            if !visited.contains(&ptr) {
                                visited.insert(ptr);
                                acc.push(*v)
                            }
                        } else if let Some(set) = not_matched {
                            set.insert(idx);
                        }
                    }
                    Value::Array(ay) => {
                        if let Some(set) = not_matched { set.insert(idx); }
                        for v in ay {
                            ValueWalker::walk_dedup(v, &mut acc, k, &mut visited);
                        }
                    }
                    _ => {
                        if let Some(set) = not_matched { set.insert(idx); }
                    }
                }
            });

            (FilterKey::String(key), acc)
        });

        debug!("filter_next_with_str : {}, {:?}", key, self.0);
        current
    }

    pub fn collect_next_with_num(&mut self, current: Option<Vec<&'a Value>>, index: f64) -> Option<Vec<&'a Value>> {
        if current.is_none() {
            debug!("collect_next_with_num : {:?}, {:?}", &index, &current);
            return current;
        }

        let mut acc = Vec::new();
        current.unwrap().iter().for_each(|v| {
            match v {
                Value::Object(map) => {
                    for k in map.keys() {
                        if let Some(Value::Array(vec)) = map.get(k) {
                            if let Some(v) = vec.get(utils::abs_index(index as isize, vec.len())) {
                                acc.push(v);
                            }
                        }
                    }
                }
                Value::Array(vec) => {
                    if let Some(v) = vec.get(utils::abs_index(index as isize, vec.len())) {
                        acc.push(v);
                    }
                }
                _ => {}
            }
        });

        if acc.is_empty() {
            self.pop_term();
        }

        Some(acc)
    }

    pub fn collect_next_all(&mut self, current: Option<Vec<&'a Value>>) -> Option<Vec<&'a Value>> {
        if current.is_none() {
            debug!("collect_next_all : {:?}", &current);
            return current;
        }

        let mut acc = Vec::new();
        current.unwrap().iter().for_each(|v| {
            match v {
                Value::Object(map) => acc.extend(map.values()),
                Value::Array(vec) => acc.extend(vec),
                _ => {}
            }
        });

        Some(acc)
    }

    pub fn collect_next_with_str(&mut self, current: Option<Vec<&'a Value>>, keys: &[&'a str]) -> Option<Vec<&'a Value>> {
        if current.is_none() {
            debug!(
                "collect_next_with_str : {:?}, {:?}",
                keys, &current
            );
            return current;
        }

        trace!("#1. {:?}", keys);
        let acc = ValueWalker::all_with_strs(current.unwrap(), keys);

        if acc.is_empty() {
            self.pop_term();
        }

        Some(acc)
    }

    pub fn collect_all(&mut self, current: Option<Vec<&'a Value>>) -> Option<Vec<&'a Value>> {
        if current.is_none() {
            debug!("collect_all: {:?}", &current);
            return current;
        }

        Some(ValueWalker::all(current.unwrap()))
    }

    pub fn collect_all_with_str(&mut self, current: Option<Vec<&'a Value>>, key: &'a str) -> Option<Vec<&'a Value>> {
        if current.is_none() {
            debug!("collect_all_with_str: {}, {:?}", key, &current);
            return current;
        }

        let ret = ValueWalker::all_with_str(current.unwrap(), key);
        Some(ret)
    }

    pub fn collect_all_with_num(&mut self, mut current: Option<Vec<&'a Value>>, index: f64) -> Option<Vec<&'a Value>> {
        if let Some(current) = current.take() {
            let ret = ValueWalker::all_with_num(current, index);
            if !ret.is_empty() {
                return Some(ret);
            }
        }

        debug!("collect_all_with_num: {}, {:?}", index, &current);
        None
    }
}

#[cfg(test)]
mod expr_term_inner_tests {
    use serde_json::{Number, Value};

    use selector::terms::ExprTerm;

    #[test]
    fn value_vec_into() {
        let v = Value::Bool(true);
        let vec = &vec![&v];
        let term: ExprTerm = vec.into();
        assert_eq!(term, ExprTerm::Bool(true));

        let v = Value::String("a".to_string());
        let vec = &vec![&v];
        let term: ExprTerm = vec.into();
        assert_eq!(term, ExprTerm::String("a"));

        let v = serde_json::from_str("1.0").unwrap();
        let vec = &vec![&v];
        let term: ExprTerm = vec.into();
        assert_eq!(term, ExprTerm::Number(Number::from_f64(1.0).unwrap()));
    }
}
