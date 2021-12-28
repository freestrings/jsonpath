use paths::tokenizer::TokenReader;
use super::str_reader::StrRange;

pub(crate) mod constants {
    pub const CH_DOLLA: char = '$';
    pub const CH_DOT: char = '.';
    pub const CH_ASTERISK: char = '*';
    pub const CH_LARRAY: char = '[';
    pub const CH_RARRAY: char = ']';
    pub const CH_LPAREN: char = '(';
    pub const CH_RPAREN: char = ')';
    pub const CH_AT: char = '@';
    pub const CH_QUESTION: char = '?';
    pub const CH_COMMA: char = ',';
    pub const CH_SEMICOLON: char = ':';
    pub const CH_EQUAL: char = '=';
    pub const CH_AMPERSAND: char = '&';
    pub const CH_PIPE: char = '|';
    pub const CH_LITTLE: char = '<';
    pub const CH_GREATER: char = '>';
    pub const CH_EXCLAMATION: char = '!';
    pub const CH_SINGLE_QUOTE: char = '\'';
    pub const CH_DOUBLE_QUOTE: char = '"';

    // tokens
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

    // parser tokens
    pub const P_TOK_ABSOLUTE: &str = "Absolute";
    pub const P_TOK_RELATIVE: &str = "Relative";
    pub const P_TOK_LEAVES: &str = "Leaves";
    pub const P_TOK_IN: &str = "In";
    pub const P_TOK_ALL: &str = "All";
    pub const P_TOK_RANGE: &str = "Range";
    pub const P_TOK_RANGE_TO: &str = "RangeTo";
    pub const P_TOK_RANGE_FROM: &str = "RangeFrom";
    pub const P_TOK_UNION: &str = "Union";
    pub const P_TOK_ARRAY: &str = "Array";
    pub const P_TOK_ARRAY_END: &str = "ArrayEnd";
    pub const P_TOK_END: &str = "End";
    pub const P_TOK_KEY: &str = "Key";
    pub const P_TOK_KEYS: &str = "Keys";
    pub const P_TOK_NUMBER: &str = "Number";
    pub const P_TOK_BOOL: &str = "Bool";
    pub const P_TOK_FILTER_AND: &str = "And";
    pub const P_TOK_FILTER_OR: &str = "Or";
    pub const P_TOK_FILTER_EQUAL: &str = "FilterEqual";
    pub const P_TOK_FILTER_NOT_EQUAL: &str = "FilterNotEqual";
    pub const P_TOK_FILTER_LITTLE: &str = "FilterLittle";
    pub const P_TOK_FILTER_LITTLE_OR_EQUAL: &str = "FilterLittleOrEqual";
    pub const P_TOK_FILTER_GREATER: &str = "FilterGreater";
    pub const P_TOK_FILTER_GREATER_OR_EQUAL: &str = "GreaterOrEqual";
}

#[derive(Debug, PartialEq)]
pub(crate) struct _Token<'a> {
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
pub(crate) enum _TokenType {
    String(StrRange),
    Int(StrRange),
    Float(StrRange),
    Bool(StrRange),
}

impl _TokenType {
    pub fn validate_token_type(&self, token_reader: &mut TokenReader) -> Result<(), StrRange> {
        match self {
            _TokenType::String(_) => Ok(()),
            _TokenType::Float(r) => {
                let v = token_reader.read_value(r);
                v.parse::<f64>().map_err(|_| {
                    trace!("expected float but: {}", v);
                    r.clone()
                })?;
                Ok(())
            }
            _TokenType::Int(r) => {
                let v = token_reader.read_value(r);
                v.parse::<isize>().map_err(|_| {
                    trace!("expected int but: {}", v);
                    r.clone()
                })?;
                Ok(())
            }
            _TokenType::Bool(r) => {
                let v = token_reader.read_value(r);
                let bytes = v.as_bytes();

                if match &bytes[0] {
                    b't' | b'T' if &bytes.len() == &4_usize => {
                        (&bytes[1] == &b'r' || &bytes[1] == &b'R')
                            && (&bytes[2] == &b'u' || &bytes[2] == &b'U')
                            && (&bytes[3] == &b'e' || &bytes[3] == &b'E')
                    }
                    b'f' | b'F' if &bytes.len() == &5_usize => {
                        (&bytes[1] == &b'a' || &bytes[1] == &b'A')
                            && (&bytes[2] == &b'l' || &bytes[2] == &b'L')
                            && (&bytes[3] == &b's' || &bytes[3] == &b'S')
                            && (&bytes[4] == &b'e' || &bytes[4] == &b'E')
                    }
                    _ => false
                } {
                    Ok(())
                } else {
                    trace!("expected bool but: {}", v);
                    Err(r.clone())
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum _TokenValue<'a> {
    String(&'a str),
    Int(isize),
    Float(f64),
    Bool(bool),
}

impl<'a> _TokenValue<'a> {
    pub fn is_string(&self) -> bool {
        if let Self::String(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_string(&self) -> Option<&'a str> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn is_int(&self) -> bool {
        if let Self::Int(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_int(&self) -> Option<isize> {
        if let Self::Int(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn is_float(&self) -> bool {
        if let Self::Float(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Self::Float(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn is_bool(&self) -> bool {
        if let Self::Bool(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct _ParserToken<'a> {
    pub key: &'a str,
    pub token_type: Option<Vec<_TokenType>>,
}

impl<'a> _ParserToken<'a> {
    pub fn new(key: &'a str) -> Self {
        _ParserToken {
            key,
            token_type: None,
        }
    }

    pub fn new_with_type(key: &'a str, token_type: _TokenType) -> Self {
        _ParserToken {
            key,
            token_type: Some(vec![token_type]),
        }
    }

    pub fn new_with_types(key: &'a str, data_type: Vec<_TokenType>) -> Self {
        _ParserToken {
            key,
            token_type: Some(data_type),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ParseToken {
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

