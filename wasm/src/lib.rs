extern crate cfg_if;
extern crate wasm_bindgen;

extern crate serde_json;
extern crate jsonpath_lib as jsonpath;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use std::result::Result;
use std::rc::Rc;
use serde_json::Value;

use jsonpath::parser::parser::*;
use jsonpath::filter::value_filter::*;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

fn filter_value(json: Value, node: Node) -> JsValue {
    let mut jf = JsonValueFilter::new_from_value(Rc::new(Box::new(json)));
    jf.visit(node);
    let taken = jf.take_value();
    match JsValue::from_serde(&taken) {
        Ok(js_value) => js_value,
        Err(e) => JsValue::from_str(&format!("Json deserialize error: {:?}", e))
    }
}

fn into_value(js_value: &JsValue) -> Result<Value, String> {
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

fn into_js_value(js_value: &JsValue, node: Node) -> JsValue {
    match into_value(js_value) {
        Ok(json) => filter_value(json, node),
        Err(e) => JsValue::from_str(&format!("Json serialize error: {}", e))
    }
}

#[wasm_bindgen]
pub fn compile(path: &str) -> JsValue {
    let mut parser = Parser::new(path);
    let node = parser.compile();
    let cb = Closure::wrap(Box::new(move |js_value: JsValue| {
        match &node {
            Ok(node) => into_js_value(&js_value, node.clone()),
            Err(e) => JsValue::from_str(&format!("Json path error: {:?}", e))
        }
    }) as Box<Fn(JsValue) -> JsValue>);

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn reader(js_value: JsValue) -> JsValue {
    let cb = Closure::wrap(Box::new(move |path: String| {
        let mut parser = Parser::new(path.as_str());
        match parser.compile() {
            Ok(node) => into_js_value(&js_value, node),
            Err(e) => return JsValue::from_str(e.as_str())
        }
    }) as Box<Fn(String) -> JsValue>);

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn read(js_value: JsValue, path: &str) -> JsValue {
    let mut parser = Parser::new(path);
    match parser.compile() {
        Ok(node) => into_js_value(&js_value, node),
        Err(e) => return JsValue::from_str(e.as_str())
    }
}

#[wasm_bindgen]
pub fn testa() {
}