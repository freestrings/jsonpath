#[macro_use]
extern crate log;
extern crate env_logger;

extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_json;
#[cfg(not(test))]
extern crate serde_json;

extern crate core;
extern crate indexmap;

pub mod parser;
pub mod filter;

use parser::parser::*;
use filter::value_filter::*;

use std::result;
use serde_json::Value;

type Result = result::Result<Value, String>;

pub fn compile<'a>(path: &'a str) -> impl FnMut(Value) -> Result + 'a {
    let mut parser = Parser::new(path);
    let node = parser.compile();
    move |json| {
        match &node {
            Ok(n) => {
                let mut jf = JsonValueFilter::new_from_value(json);
                jf.visit(n.clone());
                Ok(jf.take_value())
            },
            Err(e) => Err(e.clone())
        }
    }
}

pub fn reader(json: Value) -> impl FnMut(&str) -> Result {
    let mut jf = JsonValueFilter::new_from_value(json);
    move |path: &str| {
        let mut parser = Parser::new(path);
        parser.parse(&mut jf)?;
        Ok(jf.take_value())
    }
}

pub fn read(json: Value, path: &str) -> Result {
    let mut jf = JsonValueFilter::new_from_value(json);
    let mut parser = Parser::new(path);
    parser.parse(&mut jf)?;
    Ok(jf.take_value())
}
