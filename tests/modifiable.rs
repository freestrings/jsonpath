//extern crate indexmap;
//extern crate jsonpath_lib;
//#[macro_use]
//extern crate serde_json;
//
//use std::io::Read;
//
//use serde_json::Value;
//
//use jsonpath_lib::filter::value_filter::JsonValueFilter;
//use jsonpath_lib::parser::parser::Parser;
//use jsonpath_lib::ref_value::model::RefValue;
//
//fn setup() {
//    let _ = env_logger::try_init();
//}
//
//fn do_filter(path: &str, file: &str) -> JsonValueFilter {
//    let string = read_json(file);
//    let mut jf = JsonValueFilter::new(string.as_str()).unwrap();
//    let mut parser = Parser::new(path);
//    parser.parse(&mut jf).unwrap();
//    jf
//}
//
//fn read_json(path: &str) -> String {
//    let mut f = std::fs::File::open(path).unwrap();
//    let mut contents = String::new();
//    f.read_to_string(&mut contents).unwrap();
//    contents
//}
