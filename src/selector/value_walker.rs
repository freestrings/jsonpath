use std::collections::HashSet;
use std::collections::HashMap;

use serde_json::Value;
use super::utils;
use selector::utils::PathKey;

pub(super) struct ValueWalker;

impl<'a> ValueWalker {
    pub fn next_all(parents: &mut HashMap<*const Value, &'a Value>, vec: &[&'a Value]) -> Vec<&'a Value> {
        vec.iter().fold(Vec::new(), |mut acc, v| {
            match v {
                Value::Object(map) => {
                    map.values().into_iter().for_each(|v_el| {
                        let ptr_el = v_el as *const Value;
                        if !parents.contains_key(&ptr_el) {
                            parents.insert(ptr_el, v);
                        }
                        acc.push(v_el)
                    })
                },
                Value::Array(vec) => {
                    vec.iter().for_each(|v_el| {
                        let ptr_el = v_el as *const Value;
                        if !parents.contains_key(&ptr_el) {
                            parents.insert(ptr_el, v);
                        }
                        acc.push(v_el)
                    })
                }
                _ => {}
            }
            acc
        })
    }

    pub fn next_with_str(parents: &mut HashMap<*const Value, &'a Value>, vec: &[&'a Value], key: &'a str) -> Vec<&'a Value> {
        vec.iter().fold(Vec::new(), |mut acc, v| {
            if let Value::Object(map) = v {
                if let Some(v_el) = map.get(key) {
                    let ptr_el = v_el as *const Value;
                    if !parents.contains_key(&ptr_el) {
                        parents.insert(ptr_el, v);
                    }
                    acc.push(v_el);
                }
            }
            acc
        })
    }

    pub fn next_with_num(parents: &mut HashMap<*const Value, &'a Value>, vec: &[&'a Value], index: f64) -> Vec<&'a Value> {
        vec.iter().fold(Vec::new(), |mut acc, v| {
            if let Value::Array(vec) = v {
                if let Some(v_el) = vec.get(utils::abs_index(index as isize, vec.len())) {
                    let ptr_el = v_el as *const Value;
                    if !parents.contains_key(&ptr_el) {
                        parents.insert(ptr_el, v);
                    }
                    acc.push(v_el);
                }
            }
            acc
        })
    }

    pub fn all_with_num(parents: &mut HashMap<*const Value, &'a Value>, vec: &[&'a Value], index: f64) -> Vec<&'a Value> {
        Self::walk(parents, vec, &|v, acc| {
            if v.is_array() {
                if let Some(v) = v.get(index as usize) {
                    acc.push(v);
                }
            }
        })
    }

    pub fn all_with_str(parents: &mut HashMap<*const Value, &'a Value>, vec: &[&'a Value], key: &'a str) -> Vec<&'a Value> {
        let path_key = utils::to_path_str(key);
        Self::walk(parents, vec, &|v, acc| if let Value::Object(map) = v {
            if let Some(v) = map.get(path_key.get_key()) {
                acc.push(v);
            }
        })
    }

    pub fn all_with_strs(parents: &mut HashMap<*const Value, &'a Value>, vec: &[&'a Value], keys: &[&'a str]) -> Vec<&'a Value> {
        let path_keys: &Vec<PathKey> = &keys.iter().map(|key| { utils::to_path_str(key) }).collect();
        vec.iter().fold(Vec::new(), |mut acc, v| {
            if let Value::Object(map) = v {
                path_keys.iter().for_each(|pk| if let Some(v_el) = map.get(pk.get_key()) {
                    let ptr_el = v_el as *const Value;
                    if !parents.contains_key(&ptr_el) {
                        parents.insert(ptr_el, v);
                    }
                    acc.push(v_el)
                });
            }
            acc
        })
    }

    pub fn all(parents: &mut HashMap<*const Value, &'a Value>, vec: &[&'a Value]) -> Vec<&'a Value> {
        Self::walk(parents, vec, &|v, acc| {
            match v {
                Value::Array(ay) => acc.extend(ay),
                Value::Object(map) => {
                    acc.extend(map.values());
                }
                _ => {}
            }
        })
    }

    fn walk<F>(parents: &mut HashMap<*const Value, &'a Value>, vec: &[&'a Value], fun: &F) -> Vec<&'a Value>
        where
            F: Fn(&'a Value, &mut Vec<&'a Value>),
    {
        vec.iter().fold(Vec::new(), |mut acc, v| {
            Self::_walk(parents, v, &mut acc, fun);
            acc
        })
    }

    fn _walk<F>(parents: &mut HashMap<*const Value, &'a Value>, v: &'a Value, acc: &mut Vec<&'a Value>, fun: &F)
        where
            F: Fn(&'a Value, &mut Vec<&'a Value>),
    {
        fun(v, acc);

        match v {
            Value::Array(vec) => {
                vec.iter().for_each(|v_el| {
                    let ptr_el = v_el as *const Value;
                    if !parents.contains_key(&ptr_el) {
                        parents.insert(ptr_el, v);
                    }
                    Self::_walk(parents, v_el, acc, fun)
                });
            }
            Value::Object(map) => {
                map.values().into_iter().for_each(|v_el| {
                    let ptr_el = v_el as *const Value;
                    if !parents.contains_key(&ptr_el) {
                        parents.insert(ptr_el, v);
                    }
                    Self::_walk(parents, v_el, acc, fun)
                });
            }
            _ => {}
        }
    }

    pub fn walk_dedup_all<F1, F2>(parents: &mut HashMap<*const Value, &'a Value>,
                                  vec: &[&'a Value],
                                  key: &str,
                                  visited: &mut HashSet<*const Value>,
                                  is_contain: &mut F1,
                                  is_not_contain: &mut F2,
                                  depth: usize)
        where
            F1: FnMut(&'a Value),
            F2: FnMut(usize),
    {
        vec.iter().enumerate().for_each(|(index, v)| Self::walk_dedup(parents,
                                                                      v,
                                                                      key,
                                                                      visited,
                                                                      index,
                                                                      is_contain,
                                                                      is_not_contain,
                                                                      depth));
    }

    fn walk_dedup<F1, F2>(parents: &mut HashMap<*const Value, &'a Value>,
                          v: &'a Value,
                          key: &str,
                          visited: &mut HashSet<*const Value>,
                          index: usize,
                          is_contain: &mut F1,
                          is_not_contain: &mut F2,
                          depth: usize)
        where
            F1: FnMut(&'a Value),
            F2: FnMut(usize),
    {
        let ptr = v as *const Value;
        if visited.contains(&ptr) {
            return;
        }

        match v {
            Value::Object(map) => {
                if map.get(key).is_some() {
                    let ptr = v as *const Value;
                    if !visited.contains(&ptr) {
                        visited.insert(ptr);
                        is_contain(v);
                    }
                } else if depth == 0 {
                    is_not_contain(index);
                }
            }
            Value::Array(vec) => {
                if depth == 0 {
                    is_not_contain(index);
                }
                vec.iter().for_each(|v_el| {
                    let ptr_el = v_el as *const Value;
                    if !parents.contains_key(&ptr_el) {
                        parents.insert(ptr_el, v);
                    }
                    Self::walk_dedup(parents, v_el, key, visited, index, is_contain, is_not_contain, depth + 1);
                })
            }
            _ => {
                if depth == 0 {
                    is_not_contain(index);
                }
            }
        }
    }
}

