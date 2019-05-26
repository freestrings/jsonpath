//use std::collections::HashMap;
use std::ops::Deref;

use ref_value::model::{RefValue, RefValueWrapper};
use indexmap::IndexMap;

#[derive(Debug)]
pub struct PathMap {
    map: IndexMap<RefValueWrapper, String>
}

impl PathMap {
    pub(in select) fn new() -> Self {
        PathMap { map: IndexMap::new() }
    }

    pub fn get_path(&self, v: &RefValueWrapper) -> Option<&String> {
        self.map.get(v)
    }

    pub(in select) fn replace(&mut self, v: &RefValueWrapper) {
        self.map.clear();
        self.walk("".to_string(), v);
    }

    fn walk(&mut self, parent_path: String, v: &RefValueWrapper) {
        if &parent_path == "" {
            self.map.insert(v.clone(), "/".to_string());
        } else {
            self.map.insert(v.clone(), parent_path.clone());
        }

        match v.deref() {
            RefValue::Object(map) => {
                for (key, value) in map {
                    self.walk(format!("{}/{}", &parent_path, key), value);
                }
            }
            RefValue::Array(vec) => {
                for (index, value) in vec.iter().enumerate() {
                    self.walk(format!("{}/{}", &parent_path, index), value);
                }
            }
            _ => {}
        };
    }

    pub fn print(&self) {
        for (k, v) in &self.map {
            println!("{:?} : {}", k, v);
        }
    }
}