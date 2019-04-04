extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate neon;
extern crate neon_serde;
extern crate serde_json;

use std::ops::Deref;

use jsonpath::filter::value_filter::JsonValueFilter;
use jsonpath::parser::parser::{Node, NodeVisitor, Parser};
use jsonpath::ref_value::model::{RefValue, RefValueWrapper};
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

pub struct Compile {
    node: Node
}

pub struct Selector {
    json: RefValueWrapper
}

declare_types! {
    pub class JsCompile for Compile {
        init(mut ctx) {
            let path = ctx.argument::<JsString>(0)?.value();
            let mut parser = Parser::new(path.as_str());

            let node = match parser.compile() {
                Ok(node) => node,
                Err(e) => panic!("{:?}", e)
            };

            Ok(Compile { node })
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

    pub class JsSelector for Selector {
        init(mut ctx) {
            let json_str = ctx.argument::<JsString>(0)?.value();
            let ref_value: RefValue = match serde_json::from_str(&json_str) {
                Ok(ref_value) => ref_value,
                Err(e) => panic!("{:?}", e)
            };

            Ok(Selector { json: ref_value.into() })
        }

        method selector(mut ctx) {
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
}
register_module!(mut m, {
    m.export_class::<JsCompile>("Compile").expect("Compile class error");
    m.export_class::<JsSelector>("Selector").expect("Selector class error");
    m.export_function("select", select)?;
    m.export_function("selectStr", select_str)?;
    Ok(())
});