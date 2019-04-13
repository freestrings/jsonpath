extern crate cfg_if;
extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate lazy_static;
extern crate serde_json;
extern crate wasm_bindgen;
extern crate web_sys;

use std::collections::HashMap;
use std::ops::Deref;
use std::result;
use std::result::Result;
use std::sync::Mutex;

use cfg_if::cfg_if;
use jsonpath::filter::value_filter::JsonValueFilter;
use jsonpath::parser::parser::{Node, NodeVisitor, Parser};
use jsonpath::ref_value::model::{RefValue, RefValueWrapper};
use jsonpath::Selector as _Selector;
use wasm_bindgen::prelude::*;
use web_sys::console;

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
    let taken = &jf.take_value();
    match JsValue::from_serde(taken.deref()) {
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

#[wasm_bindgen(js_name = allocJson)]
pub extern fn alloc_json(js_value: JsValue) -> usize {
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

#[wasm_bindgen(js_name = deallocJson)]
pub extern fn dealloc_json(ptr: usize) -> bool {
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

///
/// `wasm_bindgen` 제약으로 builder-pattern을 구사 할 수 없다.
///
#[wasm_bindgen]
pub struct Selector {
    selector: _Selector
}

#[wasm_bindgen]
impl Selector {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Selector { selector: _Selector::new() }
    }

    #[wasm_bindgen(catch)]
    pub fn path(&mut self, path: &str) -> result::Result<(), JsValue> {
        let _ = self.selector.path(path)?;
        Ok(())
    }

    #[wasm_bindgen(catch)]
    pub fn value(&mut self, value: JsValue) -> result::Result<(), JsValue> {
        let ref_value = into_serde_json(&value)?;
        let _ = self.selector.value(ref_value)?;
        Ok(())
    }

    #[wasm_bindgen(catch, js_name = selectToStr)]
    pub fn select_to_str(&mut self) -> result::Result<JsValue, JsValue> {
        let json_str = self.selector.select_to_str()?;
        Ok(JsValue::from_str(&json_str))
    }

    #[wasm_bindgen(catch, js_name = selectTo)]
    pub fn select_to(&mut self) -> result::Result<JsValue, JsValue> {
        let ref_value = self.selector.select_to::<RefValue>()
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(JsValue::from_serde(&ref_value)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?)
    }
}

#[wasm_bindgen(catch)]
pub fn testa(js_value: JsValue, path: &str, iter: usize) -> result::Result<(), JsValue> {
    for _ in 0..iter {
        let mut parser = Parser::new(path);
        let node = parser.compile().unwrap();
        into_ref_value(&js_value, node);
    }
    Ok(())
}