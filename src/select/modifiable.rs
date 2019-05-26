use std::collections::HashMap;
use std::ops::Deref;

use ref_value::model::{RefValue, RefValueWrapper};
use Selector;

pub trait Modifiable {
    fn delete(&mut self) -> Result<&mut Self, String>;
}

impl Modifiable for Selector {
    fn delete(&mut self) -> Result<&mut Self, String> {
        Ok(self)
    }
}

fn traverse(parent_path: String, v: &RefValueWrapper, buf: &mut HashMap<RefValueWrapper, String>, depth: usize, limit: usize) {
    if depth >= limit {
        return;
    }

    match v.deref() {
        RefValue::Array(vec) => {
            for (i, v) in vec.iter().enumerate() {
                buf.insert(v.clone(), format!("{}/{}", parent_path, i.to_string()));
            }
            for (i, v) in vec.iter().enumerate() {
                traverse(format!("{}/{}", parent_path, i.to_string()), v, buf, depth + 1, limit);
            }
        }
        RefValue::Object(map) => {
            for (k, v) in map.into_iter() {
                buf.insert(v.clone(), format!("{}/{}", parent_path, k.to_string()));
            }
            for (k, v) in map.into_iter() {
                traverse(format!("{}/{}", parent_path, k.to_string()), v, buf, depth + 1, limit);
            }
        }
        _ => {
            buf.insert(v.clone(), parent_path);
        }
    }
}

pub struct PathFinder {
    map: HashMap<RefValueWrapper, String>,
}

impl PathFinder {
    pub fn new(v: RefValueWrapper) -> Self {
        let mut map = HashMap::new();
        traverse("/".to_string(), &v, &mut map, 0, 1);
        debug!("map: {:?}", map);
        PathFinder { map }
    }

    pub fn get(&self, v: &RefValueWrapper) -> Option<&String> {
        self.map.get(v)
    }
}