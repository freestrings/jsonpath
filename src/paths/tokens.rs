use super::str_reader::StrRange;

pub const TOK_ABSOLUTE: &str = "$";
pub const TOK_DOT: &str = ".";
pub const TOK_AT: &str = "@";
pub const TOK_OPEN_ARRAY: &str = "[";
pub const TOK_CLOSE_ARRAY: &str = "]";
pub const TOK_ASTERISK: &str = "*";
pub const TOK_QUESTION: &str = "?";
pub const TOK_COMMA: &str = ",";
pub const TOK_SPLIT: &str = ":";
pub const TOK_OPEN_PARENTHESIS: &str = "(";
pub const TOK_CLOSE_PARENTHESIS: &str = ")";
pub const TOK_KEY: &str = "___KEY___";
pub const TOK_DOUBLE_QUOTED: &str = "\"";
pub const TOK_SINGLE_QUOTED: &str = "'";
pub const TOK_EQUAL: &str = "==";
pub const TOK_GREATER_OR_EQUAL: &str = ">=";
pub const TOK_GREATER: &str = ">";
pub const TOK_LITTLE: &str = "<";
pub const TOK_LITTLE_OR_EQUAL: &str = "<=";
pub const TOK_NOT_EQUAL: &str = "!=";
pub const TOK_AND: &str = "&&";
pub const TOK_OR: &str = "||";
pub const TOK_WHITESPACE: &str = "___WHITESPACE___";

#[derive(Debug, PartialEq)]
pub(super) struct _Token<'a> {
    pub key: &'a str,
    pub range: StrRange,
}

impl<'a> _Token<'a> {
    pub fn new(key: &'a str, range: StrRange) -> Self {
        _Token { key, range }
    }

    pub fn replace_range(&mut self, range: StrRange) -> Self {
        _Token { key: self.key, range }
    }

    pub fn is_type_matched(&self, other: &_Token) -> bool {
        self.key == other.key
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum _TokenValueType {
    String,
    // TODO NegativeInt,
    Int,
    Float,
    Bool,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum _TokenValue<'a> {
    String(&'a str),
    Int(isize),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct _ParserToken<'a> {
    pub key: &'a str,
    pub value_range: Option<Vec<StrRange>>,
    pub value_type: Option<_TokenValueType>,
}

impl<'a> _ParserToken<'a> {
    pub fn new(key: &'a str) -> Self {
        _ParserToken {
            key,
            value_range: None,
            value_type: None,
        }
    }

    // pub fn parse_value<'b>(&self, token_reader: &TokenReader) -> Vec<_TokenValue<'b>> {
    //     if let Some(ranges) = &self.value_range {
    //         ranges.iter().map(|range| {
    //             let value = token_reader.read_value(range);
    //             match self.value_type {
    //                 Some(_TokenValueType::String) => _TokenValue::String(value),
    //                 _ => panic!("")
    //             }
    //         }).collect()
    //     } else {
    //         panic!("Value range is null")
    //     }
    // }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseToken {
    // '$'
    Absolute,
    // '@'
    Relative,
    // '.'
    In,
    // '..'
    Leaves,
    // '*'
    All,

    Key(StrRange),
    Keys(Vec<StrRange>),
    // []
    Array,
    // 메타토큰
    ArrayEof,
    // ?( filter )
    Filter(FilterToken),
    // 1 : 2
    Range(Option<isize>, Option<isize>, Option<usize>),
    // 1, 2, 3
    Union(Vec<isize>),

    Number(f64),

    Bool(bool),

    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterToken {
    Equal,
    NotEqual,
    Little,
    LittleOrEqual,
    Greater,
    GreaterOrEqual,
    And,
    Or,
}
