pub(crate) use self::parser_token_handler::ParserTokenHandler;
pub use self::path_parser::PathParser;
use self::str_reader::StrRange;
pub(crate) use self::tokenizer::std_token_str;
pub(crate) use self::tokenizer::TokenError;
use self::tokenizer::TokenReader;

mod str_reader;

mod tokenizer;
mod tokenizer_ext;
mod parser_token_handler;

mod path_parser;


#[derive(Debug, PartialEq)]
pub(crate) struct Token<'a> {
    pub key: &'a str,
    pub range: StrRange,
}

impl<'a> Token<'a> {
    pub fn new(key: &'a str, range: StrRange) -> Self {
        Token { key, range }
    }

    pub fn replace_range(&mut self, range: StrRange) -> Self {
        Token { key: self.key, range }
    }

    pub fn is_type_matched(&self, other: &Token) -> bool {
        self.key == other.key
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TokenType {
    String(StrRange),
    Int(StrRange),
    Float(StrRange),
    Bool(StrRange),
}

impl TokenType {
    pub fn validate_token_type(&self, token_reader: &mut TokenReader) -> Result<(), StrRange> {
        match self {
            TokenType::String(_) => Ok(()),
            TokenType::Float(r) => {
                let v = token_reader.read_value(r);
                v.parse::<f64>().map_err(|_| {
                    trace!("expected float but: {}", v);
                    r.clone()
                })?;
                Ok(())
            }
            TokenType::Int(r) => {
                let v = token_reader.read_value(r);
                v.parse::<isize>().map_err(|_| {
                    trace!("expected int but: {}", v);
                    r.clone()
                })?;
                Ok(())
            }
            TokenType::Bool(r) => {
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
pub(crate) enum TokenValue<'a> {
    String(&'a str),
    Int(isize),
    Float(f64),
    Bool(bool),
}

impl<'a> TokenValue<'a> {
    pub fn is_str(&self) -> bool {
        if let Self::String(_) = self {
            true
        } else {
            false
        }
    }

    pub fn get_str(&self) -> Option<&'a str> {
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
        if let Self::String(v) = self {
            if let Ok(v) = v.parse::<isize>() {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_int(&self) -> Option<isize> {
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
        if let Self::String(v) = self {
            if let Ok(v) = v.parse::<f64>() {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_float(&self) -> Option<f64> {
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
        if let Self::String(v) = self {
            let v = v.to_lowercase(); // FIXME
            if let Ok(v) = v.parse::<bool>() {
                Some(v)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ParserToken<'a> {
    pub key: &'a str,
    pub token_type: Option<Vec<TokenType>>,
}

impl<'a> ParserToken<'a> {
    pub fn new(key: &'a str) -> Self {
        ParserToken {
            key,
            token_type: None,
        }
    }

    pub fn new_with_type(key: &'a str, token_type: TokenType) -> Self {
        ParserToken {
            key,
            token_type: Some(vec![token_type]),
        }
    }

    pub fn new_with_types(key: &'a str, data_type: Vec<TokenType>) -> Self {
        ParserToken {
            key,
            token_type: Some(data_type),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ParserNode<'a> {
    pub left: Option<Box<ParserNode<'a>>>,
    pub right: Option<Box<ParserNode<'a>>>,
    pub token: ParserToken<'a>,
}

impl<'a> ParserNode<'a> {
    pub fn new(token: &'a str) -> Self {
        ParserNode {
            left: None,
            right: None,
            token: ParserToken::new(token),
        }
    }

    pub fn new_with_token_value(token: &'a str, token_type: TokenType) -> Self {
        ParserNode {
            left: None,
            right: None,
            token: ParserToken::new_with_type(token, token_type),
        }
    }

    pub fn new_with_token_values(token: &'a str, token_type: Vec<TokenType>) -> Self {
        ParserNode {
            left: None,
            right: None,
            token: ParserToken::new_with_types(token, token_type),
        }
    }
}