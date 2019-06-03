extern crate cfg_if;
extern crate core;
extern crate js_sys;
extern crate jsonpath_lib as jsonpath;
extern crate serde;
extern crate serde_json;
extern crate wasm_bindgen;
extern crate web_sys;

use cfg_if::cfg_if;
use jsonpath::{JsonPathError, Parser};
use jsonpath::Selector as _Selector;
use serde_json::Value;
use wasm_bindgen::*;
use wasm_bindgen::prelude::*;

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

fn into_serde_json<D>(js_value: &JsValue) -> Result<D, String>
    where D: for<'a> serde::de::Deserialize<'a>
{
    if js_value.is_string() {
        match serde_json::from_str(js_value.as_string().unwrap().as_str()) {
            Ok(json) => Ok(json),
            Err(e) => Err(e.to_string())
        }
    } else {
        match js_value.into_serde() {
            Ok(json) => Ok(json),
            Err(e) => Err(e.to_string())
        }
    }
}

#[wasm_bindgen]
pub fn compile(path: &str) -> JsValue {
    let node = Parser::compile(path);

    let cb = Closure::wrap(Box::new(move |js_value: JsValue| {
        let mut selector = _Selector::new();
        match &node {
            Ok(node) => selector.compiled_path(node.clone()),
            Err(e) => return JsValue::from_str(&format!("{:?}", JsonPathError::Path(e.clone())))
        };
        let json = match into_serde_json(&js_value) {
            Ok(json) => json,
            Err(e) => return JsValue::from_str(&format!("{:?}", JsonPathError::Serde(e)))
        };
        match selector.value(&json).select() {
            Ok(ret) => match JsValue::from_serde(&ret) {
                Ok(ret) => ret,
                Err(e) => JsValue::from_str(&format!("{:?}", JsonPathError::Serde(e.to_string())))
            },
            Err(e) => JsValue::from_str(&format!("{:?}", e))
        }
    }) as Box<Fn(JsValue) -> JsValue>);

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn selector(js_value: JsValue) -> JsValue {
    let json: Value = match JsValue::into_serde(&js_value) {
        Ok(json) => json,
        Err(e) => return JsValue::from_str(&format!("{:?}", JsonPathError::Serde(e.to_string())))
    };

    let cb = Closure::wrap(Box::new(move |path: String| {
        match Parser::compile(path.as_str()) {
            Ok(node) => {
                let mut selector = _Selector::new();
                let _ = selector.compiled_path(node);
                match selector.value(&json).select() {
                    Ok(ret) => match JsValue::from_serde(&ret) {
                        Ok(ret) => ret,
                        Err(e) => JsValue::from_str(&format!("{:?}", JsonPathError::Serde(e.to_string())))
                    },
                    Err(e) => JsValue::from_str(&format!("{:?}", e))
                }
            }
            Err(e) => return JsValue::from_str(&format!("{:?}", JsonPathError::Path(e)))
        }
    }) as Box<Fn(String) -> JsValue>);

    let ret = cb.as_ref().clone();
    cb.forget();
    ret
}

#[wasm_bindgen]
pub fn select(js_value: JsValue, path: &str) -> JsValue {
    let mut selector = _Selector::new();
    let _ = selector.path(path);

    let json = match into_serde_json(&js_value) {
        Ok(json) => json,
        Err(e) => return JsValue::from_str(&format!("{:?}", JsonPathError::Serde(e)))
    };

    match selector.value(&json).select() {
        Ok(ret) => match JsValue::from_serde(&ret) {
            Ok(ret) => ret,
            Err(e) => JsValue::from_str(&format!("{:?}", JsonPathError::Serde(e.to_string())))
        },
        Err(e) => JsValue::from_str(&format!("{:?}", e))
    }
}

///
/// `wasm_bindgen` 제약으로 builder-pattern을 구사 할 수 없다.
/// lifetime 제약으로 Selector를 사용 할 수 없다.
///
#[wasm_bindgen]
pub struct Selector {
    path: Option<String>,
    value: Option<Value>,
}

#[wasm_bindgen]
impl Selector {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Selector { path: None, value: None }
    }

    #[wasm_bindgen(catch)]
    pub fn path(&mut self, path: &str) -> Result<(), JsValue> {
        self.path = Some(path.to_string());
        Ok(())
    }

    #[wasm_bindgen(catch)]
    pub fn value(&mut self, value: JsValue) -> Result<(), JsValue> {
        let json = into_serde_json(&value)
            .map_err(|e| JsValue::from_str(&format!("{:?}", JsonPathError::Serde(e))))?;
        self.value = Some(json);
        Ok(())
    }

    #[wasm_bindgen(catch, js_name = select)]
    pub fn select(&mut self) -> Result<JsValue, JsValue> {
        let mut selector = _Selector::new();

        if let Some(path) = &self.path {
            let _ = selector.path(&path).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        } else {
            return Err(JsValue::from_str(&format!("{:?}", JsonPathError::EmptyPath)));
        }

        if let Some(value) = &self.value {
            let _ = selector.value(value);
        } else {
            return Err(JsValue::from_str(&format!("{:?}", JsonPathError::EmptyValue)));
        }

        match selector.select() {
            Ok(ret) => match JsValue::from_serde(&ret) {
                Ok(ret) => Ok(ret),
                Err(e) => Err(JsValue::from_str(&format!("{:?}", JsonPathError::Serde(e.to_string()))))
            },
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e)))
        }
    }
}