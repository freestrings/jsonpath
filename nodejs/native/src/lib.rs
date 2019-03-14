extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate neon;
extern crate neon_serde;
extern crate serde_json;

use jsonpath::prelude::*;
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
    let json: Value = match serde_json::from_str(json_val.as_str()) {
        Ok(json) => json,
        Err(e) => panic!("{:?}", e)
    };
    let path = ctx.argument::<JsString>(1)?.value();
    match jsonpath::select(&json, path.as_str()) {
        Ok(value) => Ok(neon_serde::to_value(&mut ctx, &value)?),
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

//            let o = ctx.argument::<JsValue>(0)?;
//            let json: Value = neon_serde::from_value(&mut ctx, o)?;
            let json_str = ctx.argument::<JsString>(0)?.value();
            let json: Value = match serde_json::from_str(&json_str) {
                Ok(json) => json,
                Err(e) => panic!("{:?}", e)
            };
            let mut jf = JsonValueFilter::new_from_value((&json).into());
            jf.visit(node);
            let v = jf.take_value().into_value();
            Ok(neon_serde::to_value(&mut ctx, &v)?)
        }
    }

    pub class JsSelector for Selector {
        init(mut ctx) {
//            let o = ctx.argument::<JsValue>(0)?;
//            let json: Value = neon_serde::from_value(&mut ctx, o)?;
            let json_str = ctx.argument::<JsString>(0)?.value();
            let json: Value = match serde_json::from_str(&json_str) {
                Ok(json) => json,
                Err(e) => panic!("{:?}", e)
            };

            Ok(Selector { json: (&json).into() })
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
            let v = jf.take_value().into_value();
            Ok(neon_serde::to_value(&mut ctx, &v)?)
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