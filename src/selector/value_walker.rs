use std::collections::HashSet;

use serde_json::Value;
use super::utils;

pub(super) struct ValueWalker;

impl<'a> ValueWalker {
    pub fn all_with_num(vec: Vec<&'a Value>, index: f64) -> Vec<&'a Value> {
        Self::walk(vec, &|v, acc| {
            if v.is_array() {
                if let Some(vv) = v.get(index as usize) {
                    acc.push(vv);
                }
            }
        })
    }

    pub fn all_with_str(vec: Vec<&'a Value>, key: &'a str, is_filter: bool) -> Vec<&'a Value> {
        let (key, opt) = utils::to_path_str(key);
        let k = if let Some(opt) = opt.as_ref() { opt } else { key };
        Self::walk(vec, &|v, acc| {
            if is_filter {
                match v {
                    Value::Object(map) => {
                        if let Some(v) = map.get(k) {
                            acc.push(v);
                        }
                    }
                    _ => {}
                }
            } else {
                match v {
                    Value::Object(map) => {
                        if let Some(v) = map.get(k) {
                            acc.push(v);
                        }
                    }
                    _ => {}
                }
            }
        })
    }

    pub fn all_with_strs(vec: Vec<&'a Value>, keys: &[&'a str]) -> Vec<&'a Value> {
        let mut acc = Vec::new();
        let mut new_keys = Vec::new();
        for key in keys {
            new_keys.push(utils::to_path_str(key));
        }

        for v in vec {
            if let Value::Object(map) = v {
                for (key, opt) in &new_keys {
                    let k = if let Some(opt) = opt.as_ref() { opt } else { *key };
                    if let Some(v) = map.get(k) {
                        acc.push(v)
                    }
                }
            }
        }
        acc
    }

    pub fn all(vec: Vec<&'a Value>) -> Vec<&'a Value> {
        Self::walk(vec, &|v, acc| {
            match v {
                Value::Array(ay) => acc.extend(ay),
                Value::Object(map) => {
                    acc.extend(map.values());
                }
                _ => {}
            }
        })
    }

    fn walk<F>(vec: Vec<&'a Value>, fun: &F) -> Vec<&'a Value>
        where
            F: Fn(&'a Value, &mut Vec<&'a Value>),
    {
        let mut acc = Vec::new();
        vec.iter().for_each(|v| {
            Self::_walk(v, &mut acc, fun);
        });
        acc
    }

    fn _walk<F>(v: &'a Value, acc: &mut Vec<&'a Value>, fun: &F)
        where
            F: Fn(&'a Value, &mut Vec<&'a Value>),
    {
        fun(v, acc);

        match v {
            Value::Array(vec) => {
                vec.iter().for_each(|v| Self::_walk(v, acc, fun));
            }
            Value::Object(map) => {
                map.values().into_iter().for_each(|v| Self::_walk(&v, acc, fun));
            }
            _ => {}
        }
    }

    pub fn walk_dedup(v: &'a Value,
                      acc: &mut Vec<&'a Value>,
                      key: &str,
                      visited: &mut HashSet<*const Value>, ) {
        match v {
            Value::Object(map) => {
                if map.contains_key(key) {
                    let ptr = v as *const Value;
                    if !visited.contains(&ptr) {
                        visited.insert(ptr);
                        acc.push(v)
                    }
                }
            }
            Value::Array(vec) => {
                for v in vec {
                    Self::walk_dedup(v, acc, key, visited);
                }
            }
            _ => {}
        }
    }
}

