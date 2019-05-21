extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate neon;
extern crate neon_serde;
extern crate serde_json;

use jsonpath::filter::value_filter::JsonValueFilter;
use jsonpath::parser::parser::{Node, NodeVisitor, Parser};
use jsonpath::ref_value::model::{RefValue, RefValueWrapper};
use jsonpath::Selector;
use neon::prelude::*;
use serde_json::Value;
use std::ops::Deref;

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

pub struct CompileFn {
    node: Node
}

pub struct SelectorFn {
    json: RefValueWrapper
}

pub struct SelectorCls {
    selector: Selector
}

declare_types! {
    pub class JsCompileFn for CompileFn {
        init(mut ctx) {
            let path = ctx.argument::<JsString>(0)?.value();
            let mut parser = Parser::new(path.as_str());

            let node = match parser.compile() {
                Ok(node) => node,
                Err(e) => panic!("{:?}", e)
            };

            Ok(CompileFn { node })
        }

        method template(mut ctx) {
            let this = ctx.this();

            let node = {
                let guard = ctx.lock();
                let this = this.borrow(&guard);
                this.node.clone()
            };

            let json_str = ctx.argument::<JsString>(0)?.value();
            let ref_value: RefValue = match serde_json::from_str(&json_str) {
                Ok(ref_value) => ref_value,
                Err(e) => panic!("{:?}", e)
            };

            let mut jf = JsonValueFilter::new_from_value(ref_value.into());
            jf.visit(node);
            match serde_json::to_string(&jf.take_value().deref()) {
                Ok(json_str) => Ok(JsString::new(&mut ctx, &json_str).upcast()),
                Err(e) => panic!("{:?}", e)
            }
        }
    }

    pub class JsSelectorFn for SelectorFn {
        init(mut ctx) {
            let json_str = ctx.argument::<JsString>(0)?.value();
            let ref_value: RefValue = match serde_json::from_str(&json_str) {
                Ok(ref_value) => ref_value,
                Err(e) => panic!("{:?}", e)
            };

            Ok(SelectorFn { json: ref_value.into() })
        }

        method select(mut ctx) {
            let this = ctx.this();

            let json = {
                let guard = ctx.lock();
                let this = this.borrow(&guard);
                this.json.clone()
            };

            let path = ctx.argument::<JsString>(0)?.value();
            let mut parser = Parser::new(path.as_str());

            let node = match parser.compile() {
                Ok(node) => node,
                Err(e) => panic!("{:?}", e)
            };

            let mut jf = JsonValueFilter::new_from_value(json);
            jf.visit(node);
            match serde_json::to_string(&jf.take_value().deref()) {
                Ok(json_str) => Ok(JsString::new(&mut ctx, &json_str).upcast()),
                Err(e) => panic!("{:?}", e)
            }
        }
    }

    pub class JsSelector for SelectorCls {
        init(mut _ctx) {
            Ok(SelectorCls { selector: Selector::new() })
        }

        method path(mut ctx) {
            let mut this = ctx.this();

            let path = ctx.argument::<JsString>(0)?.value();
            {
                let guard = ctx.lock();
                let mut this = this.borrow_mut(&guard);
                let _ = this.selector.path(&path);
            }
            Ok(JsUndefined::new().upcast())
        }

        method valueFromStr(mut ctx) {
            let mut this = ctx.this();

            let json_str = ctx.argument::<JsString>(0)?.value();
            {
                let guard = ctx.lock();
                let mut this = this.borrow_mut(&guard);
                let _ = this.selector.value_from_str(&json_str);
            }
            Ok(JsUndefined::new().upcast())
        }

        method selectAsStr(mut ctx) {
             let mut this = ctx.this();

             let result = {
                let guard = ctx.lock();
                let this = this.borrow_mut(&guard);
                this.selector.select_as_str()
             };

             match result {
                Ok(json_str) => Ok(JsString::new(&mut ctx, &json_str).upcast()),
                Err(e) => panic!("{:?}", e)
             }
        }

        method map(mut ctx) {
            let null = ctx.null();
            let mut this = ctx.this();

            let func = ctx.argument::<JsFunction>(0)?;

            let value = {
                let guard = ctx.lock();
                let this = this.borrow_mut(&guard);
                match this.selector.select_as_str() {
                    Ok(v) => v,
                    Err(e) => panic!("{:?}", e)
                }
            };

            let js_value = JsString::new(&mut ctx, &value);
            let json_str = func.call(&mut ctx, null, vec![js_value])?
                .downcast::<JsString>()
                .or_throw(&mut ctx)?
                .value();
            {
                let guard = ctx.lock();
                let mut this = this.borrow_mut(&guard);
                let _ = this.selector.value_from_str(&json_str);
            }

            Ok(JsUndefined::new().upcast())
        }

        method get(mut ctx) {
            let mut this = ctx.this();

            let result = {
                let guard = ctx.lock();
                let this = this.borrow_mut(&guard);
                match this.selector.get() {
                    Ok(v) => v,
                    Err(e) => panic!("{:?}", e)
                }
            };

            Ok(JsString::new(&mut ctx, &result.to_string()).upcast())
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