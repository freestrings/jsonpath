extern crate env_logger;
extern crate jsonpath_lib as jsonpath;

use std::result;
use jsonpath::parser::parser::{Parser, ParseToken, NodeVisitor, FilterToken};

struct NodeVisitorTestImpl<'a> {
    input: &'a str,
    stack: Vec<ParseToken>,
}

impl<'a> NodeVisitorTestImpl<'a> {
    fn new(input: &'a str) -> Self {
        NodeVisitorTestImpl { input, stack: Vec::new() }
    }

    fn visit(&mut self) -> result::Result<Vec<ParseToken>, String> {
        let mut parser = Parser::new(self.input);
        parser.parse(self)?;
        Ok(self.stack.split_off(0))
    }
}

impl<'a> NodeVisitor for NodeVisitorTestImpl<'a> {
    fn visit_token(&mut self, token: ParseToken) {
        self.stack.push(token);
    }
}

fn setup() {
    let _ = env_logger::try_init();
}

fn run(input: &str) -> result::Result<Vec<ParseToken>, String> {
    let mut interpreter = NodeVisitorTestImpl::new(input);
    interpreter.visit()
}

#[test]
fn parse_path() {
    setup();

    assert_eq!(run("$.aa"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::In,
        ParseToken::Key("aa".to_owned())
    ]));

    assert_eq!(run("$.00.a"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::In,
        ParseToken::Key("00".to_owned()),
        ParseToken::In,
        ParseToken::Key("a".to_owned())
    ]));

    assert_eq!(run("$.00.韓창.seok"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::In,
        ParseToken::Key("00".to_owned()),
        ParseToken::In,
        ParseToken::Key("韓창".to_owned()),
        ParseToken::In,
        ParseToken::Key("seok".to_owned())
    ]));

    assert_eq!(run("$.*"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::In,
        ParseToken::All
    ]));

    assert_eq!(run("$..*"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Leaves,
        ParseToken::All
    ]));

    assert_eq!(run("$..[0]"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Leaves,
        ParseToken::Array,
        ParseToken::Number(0.0),
        ParseToken::ArrayEof
    ]));

    match run("$.") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$..") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$. a") {
        Ok(_) => panic!(),
        _ => {}
    }
}

#[test]
fn parse_array_sytax() {
    setup();

    assert_eq!(run("$.book[?(@.isbn)]"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::In,
        ParseToken::Key("book".to_string()),
        ParseToken::Array,
        ParseToken::Relative,
        ParseToken::In,
        ParseToken::Key("isbn".to_string()),
        ParseToken::ArrayEof
    ]));

    //
    // Array도 컨텍스트 In으로 간주 할거라서 중첩되면 하나만
    //
    assert_eq!(run("$.[*]"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Array,
        ParseToken::All,
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$.a[*]"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::All,
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$.a[*].가"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::All,
        ParseToken::ArrayEof,
        ParseToken::In, ParseToken::Key("가".to_owned())
    ]));

    assert_eq!(run("$.a[0][1]"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::Number(0_f64),
        ParseToken::ArrayEof,
        ParseToken::Array,
        ParseToken::Number(1_f64),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$.a[1,2]"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::Union(vec![1, 2]),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$.a[10:]"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::Range(Some(10), None),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$.a[:11]"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::Range(None, Some(11)),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$.a[-12:13]"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::Range(Some(-12), Some(13)),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$.a[?(1>2)]"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::Number(1_f64), ParseToken::Number(2_f64), ParseToken::Filter(FilterToken::Greater),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$.a[?($.b>3)]"), Ok(vec![
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Array,
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("b".to_owned()), ParseToken::Number(3_f64), ParseToken::Filter(FilterToken::Greater),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$[?($.c>@.d && 1==2)]"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Array,
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("c".to_owned()),
        ParseToken::Relative, ParseToken::In, ParseToken::Key("d".to_owned()),
        ParseToken::Filter(FilterToken::Greater),
        ParseToken::Number(1_f64), ParseToken::Number(2_f64), ParseToken::Filter(FilterToken::Equal),
        ParseToken::Filter(FilterToken::And),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$[?($.c>@.d&&(1==2||3>=4))]"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Array,
        ParseToken::Absolute, ParseToken::In, ParseToken::Key("c".to_owned()),
        ParseToken::Relative, ParseToken::In, ParseToken::Key("d".to_owned()),
        ParseToken::Filter(FilterToken::Greater),
        ParseToken::Number(1_f64), ParseToken::Number(2_f64), ParseToken::Filter(FilterToken::Equal),
        ParseToken::Number(3_f64), ParseToken::Number(4_f64), ParseToken::Filter(FilterToken::GreaterOrEqual),
        ParseToken::Filter(FilterToken::Or),
        ParseToken::Filter(FilterToken::And),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$[?(@.a<@.b)]"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Array,
        ParseToken::Relative, ParseToken::In, ParseToken::Key("a".to_owned()),
        ParseToken::Relative, ParseToken::In, ParseToken::Key("b".to_owned()),
        ParseToken::Filter(FilterToken::Little),
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$[*][*][*]"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Array,
        ParseToken::All,
        ParseToken::ArrayEof,
        ParseToken::Array,
        ParseToken::All,
        ParseToken::ArrayEof,
        ParseToken::Array,
        ParseToken::All,
        ParseToken::ArrayEof
    ]));

    assert_eq!(run("$['a']['bb']"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Array,
        ParseToken::Key("a".to_string()),
        ParseToken::ArrayEof,
        ParseToken::Array,
        ParseToken::Key("bb".to_string()),
        ParseToken::ArrayEof
    ]));

    match run("$[") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$[]") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$[a]") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$[?($.a)]") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$[?(@.a > @.b]") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$[?(@.a < @.b&&(@.c < @.d)]") {
        Ok(_) => panic!(),
        _ => {}
    }
}

#[test]
fn parse_array_float() {
    setup();

    assert_eq!(run("$[?(1.1<2.1)]"), Ok(vec![
        ParseToken::Absolute,
        ParseToken::Array,
        ParseToken::Number(1.1), ParseToken::Number(2.1), ParseToken::Filter(FilterToken::Little),
        ParseToken::ArrayEof
    ]));

    match run("$[1.1]") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$[?(1.1<.2)]") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$[?(1.1<2.)]") {
        Ok(_) => panic!(),
        _ => {}
    }

    match run("$[?(1.1<2.a)]") {
        Ok(_) => panic!(),
        _ => {}
    }
}