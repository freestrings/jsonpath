extern crate cfg_if;
extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate lazy_static;
extern crate serde_json;
extern crate wasm_bindgen;
extern crate web_sys;

use std::collections::HashMap;
use std::result::Result;
use std::sync::Mutex;

use cfg_if::cfg_if;
use serde_json::Value;
use wasm_bindgen::prelude::*;
use web_sys::console;

use jsonpath::filter::value_filter::JsonValueFilter;
use jsonpath::parser::parser::{Node, NodeVisitor, Parser};
use jsonpath::ref_value::model::{RefValue, RefValueWrapper};

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

cfg_if! {
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

fn filter_ref_value(json: RefValueWrapper, node: Node) -> JsValue {
    let mut jf = JsonValueFilter::new_from_value(json);
    jf.visit(node);
    let taken: Value = (&jf.take_value()).into();
    match JsValue::from_serde(&taken) {
        Ok(js_value) => js_value,
        Err(e) => JsValue::from_str(&format!("Json deserialize error: {:?}", e))
    }
}

fn into_serde_json(js_value: &JsValue) -> Result<RefValue, String> {
    if js_value.is_string() {
        match serde_json::from_str(js_value.as_string().unwrap().as_str()) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("{:?}", e))
        }
    } else {
        match js_value.into_serde() {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("{:?}", e))
        }
    }
}

fn into_ref_value(js_value: &JsValue, node: Node) -> JsValue {
    match into_serde_json(js_value) {
        Ok(json) => filter_ref_value(json.into(), node),
        Err(e) => JsValue::from_str(&format!("Json serialize error: {}", e))
    }
}

fn get_ref_value(js_value: JsValue, node: Node) -> JsValue {
    match js_value.as_f64() {
        Some(val) => {
            match CACHE_JSON.lock().unwrap().get(&(val as usize)) {
                Some(json) => filter_ref_value(json.clone(), node),
                _ => JsValue::from_str("Invalid pointer")
            }
        }
        _ => into_ref_value(&js_value, node)
    }
}

lazy_static! {
    static ref CACHE_JSON: Mutex<HashMap<usize, RefValueWrapper>> = Mutex::new(HashMap::new());
    static ref CACHE_JSON_IDX: Mutex<usize> = Mutex::new(0);
}

#[wasm_bindgen]
pub fn alloc_json(js_value: JsValue) -> usize {
    match into_serde_json(&js_value) {
        Ok(json) => {
            let mut map = CACHE_JSON.lock().unwrap();
            if map.len() >= std::u8::MAX as usize {
                return 0;
            }

            let mut idx = CACHE_JSON_IDX.lock().unwrap();
            *idx += 1;
            map.insert(*idx, json.into());
            *idx
        }
        Err(e) => {
            console::log_1(&e.into());
            0
        }
    }
}

#[wasm_bindgen]
pub fn dealloc_json(ptr: usize) -> bool {
    let mut map = CACHE_JSON.lock().unwrap();
    map.remove(&ptr).is_some()
}

#[wasm_bindgen]
pub fn compile(path: &str) -> JsValue {
    let mut parser = Parser::new(path);
    let node = parser.compile();
    let cb = Closure::wrap(Box::new(move |js_value: JsValue| {
        match &node {
            Ok(node) => get_ref_value(js_value, node.clone()),
            Err(e) => JsValue::from_str(&format!("Json path error: {:?}", e))
        }
    }) as Box<Fn(JsValue) -> JsValue>);

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn selector(js_value: JsValue) -> JsValue {
    let json = match js_value.as_f64() {
        Some(val) => {
            match CACHE_JSON.lock().unwrap().get(&(val as usize)) {
                Some(json) => json.clone(),
                _ => return JsValue::from_str("Invalid pointer")
            }
        }
        _ => {
            match into_serde_json(&js_value) {
                Ok(json) => json.into(),
                Err(e) => return JsValue::from_str(e.as_str())
            }
        }
    };

    let cb = Closure::wrap(Box::new(move |path: String| {
        let mut parser = Parser::new(path.as_str());
        match parser.compile() {
            Ok(node) => filter_ref_value(json.clone(), node),
            Err(e) => return JsValue::from_str(e.as_str())
        }
    }) as Box<Fn(String) -> JsValue>);

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn select(js_value: JsValue, path: &str) -> JsValue {
    let mut parser = Parser::new(path);
    match parser.compile() {
        Ok(node) => get_ref_value(js_value, node),
        Err(e) => return JsValue::from_str(e.as_str())
    }
}

#[wasm_bindgen]
pub fn testa() {}