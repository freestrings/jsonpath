extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate neon;
extern crate neon_serde;
extern crate serde_json;

use jsonpath::{JsonPathError, Node, Parser, Selector};
use neon::prelude::*;
use serde_json::Value;

///
/// `neon_serde::from_value` has very poor performance.
///
fn select(mut ctx: FunctionContext) -> JsResult<JsValue> {
    let json_val = ctx.argument::<JsValue>(0)?;
    let json: Value = neon_serde::from_value(&mut ctx, json_val)?;
    let path = ctx.argument::<JsString>(1)?.value();

    match jsonpath::select(&json, path.as_str()) {
        Ok(value) => Ok(neon_serde::to_value(&mut ctx, &value)?),
        Err(e) => panic!("{:?}", e)
    }
}

fn select_str(mut ctx: FunctionContext) -> JsResult<JsValue> {
    let json_val = ctx.argument::<JsString>(0)?.value();
    let path = ctx.argument::<JsString>(1)?.value();
    match jsonpath::select_as_str(&json_val, path.as_str()) {
        Ok(value) => Ok(JsString::new(&mut ctx, &value).upcast()),
        Err(e) => panic!("{:?}", e)
    }
}


pub struct SelectorCls {
    node: Option<Node>,
    value: Option<Value>,
}

impl SelectorCls {
    fn path(&mut self, path: &str) {
        let mut parser = Parser::new(path);
        let node = match parser.compile() {
            Ok(node) => node,
            Err(e) => panic!("{:?}", e)
        };

        self.node = Some(node);
    }

    fn value(&mut self, json_str: &str) {
        let value: Value = match serde_json::from_str(&json_str) {
            Ok(value) => value,
            Err(e) => panic!("{:?}", JsonPathError::Serde(e.to_string()))
        };

        self.value = Some(value);
    }

    fn select(&self) -> String {
        let node = match &self.node {
            Some(node) => node.clone(),
            None => panic!("{:?}", JsonPathError::EmptyPath)
        };

        let value = match &self.value {
            Some(value) => value,
            None => panic!("{:?}", JsonPathError::EmptyValue)
        };

        let mut selector = Selector::new();
        selector.compiled_path(node.clone());
        selector.value(&value);
        match selector.select_as_str() {
            Ok(ret) => ret,
            Err(e) => panic!("{:?}", e)
        }
    }
}

declare_types! {
    pub class JsCompileFn for SelectorCls {
        init(mut ctx) {
            let path = ctx.argument::<JsString>(0)?.value();
            let mut parser = Parser::new(path.as_str());
            let node = match parser.compile() {
                Ok(node) => node,
                Err(e) => panic!("{:?}", e)
            };

            Ok(SelectorCls { node: Some(node), value: None })
        }

        method template(mut ctx) {
            let mut this = ctx.this();

            let json_str = ctx.argument::<JsString>(0)?.value();
            {
                let guard = ctx.lock();
                let mut this = this.borrow_mut(&guard);
                let value: Value = match serde_json::from_str(&json_str) {
                    Ok(value) => value,
                    Err(e) => panic!("{:?}", JsonPathError::Serde(e.to_string()))
                };
                this.value = Some(value);
            };

            let result_str = {
                let guard = ctx.lock();
                let this = this.borrow(&guard);
                this.select()
            };

            Ok(JsString::new(&mut ctx, &result_str).upcast())
        }
    }

    pub class JsSelectorFn for SelectorCls {
        init(mut ctx) {
            let json_str = ctx.argument::<JsString>(0)?.value();
            let value: Value = match serde_json::from_str(&json_str) {
                Ok(value) => value,
                Err(e) => panic!("{:?}", JsonPathError::Serde(e.to_string()))
            };

            Ok(SelectorCls { node: None, value: Some(value) })
        }

        method select(mut ctx) {
            let mut this = ctx.this();

            let path = ctx.argument::<JsString>(0)?.value();
            {
                let guard = ctx.lock();
                let mut this = this.borrow_mut(&guard);
                this.path(&path);
            }

            let result_str = {
                let guard = ctx.lock();
                let this = this.borrow(&guard);
                this.select()
            };

            Ok(JsString::new(&mut ctx, &result_str).upcast())
        }
    }

    pub class JsSelector for SelectorCls {
        init(mut _ctx) {
            Ok(SelectorCls { node: None, value: None })
        }

        method path(mut ctx) {
            let mut this = ctx.this();

            let path = ctx.argument::<JsString>(0)?.value();
            {
                let guard = ctx.lock();
                let mut this = this.borrow_mut(&guard);
                let _ = this.path(&path);
            }

            Ok(JsUndefined::new().upcast())
        }

        method value(mut ctx) {
            let mut this = ctx.this();

            let json_str = ctx.argument::<JsString>(0)?.value();
            {
                let guard = ctx.lock();
                let mut this = this.borrow_mut(&guard);
                let _ = this.value(&json_str);
            }

            Ok(JsUndefined::new().upcast())
        }

        method select(mut ctx) {
             let this = ctx.this();

             let result_str = {
                let guard = ctx.lock();
                let this = this.borrow(&guard);
                this.select()
             };

             Ok(JsString::new(&mut ctx, &result_str).upcast())
        }
    }
}
register_module!(mut m, {
    m.export_class::<JsCompileFn>("CompileFn").expect("CompileFn class error");
    m.export_class::<JsSelectorFn>("SelectorFn").expect("SelectorFn class error");
    m.export_class::<JsSelector>("Selector").expect("Selector class error");
    m.export_function("select", select)?;
    m.export_function("selectStr", select_str)?;
    Ok(())
});