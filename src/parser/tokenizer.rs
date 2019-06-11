use std::io::Write;
use std::result::Result;

use super::path_reader::{PathReader, ReaderError};

pub const ABSOLUTE: &'static str = "$";
pub const DOT: &'static str = ".";
pub const AT: &'static str = "@";
pub const OPEN_ARRAY: &'static str = "[";
pub const CLOSE_ARRAY: &'static str = "]";
pub const ASTERISK: &'static str = "*";
pub const QUESTION: &'static str = "?";
pub const COMMA: &'static str = ",";
pub const SPLIT: &'static str = ":";
pub const OPEN_PARENTHESIS: &'static str = "(";
pub const CLOSE_PARENTHESIS: &'static str = ")";
pub const KEY: &'static str = "Key";
pub const DOUBLE_QUOTA: &'static str = "\"";
pub const SINGLE_QUOTA: &'static str = "'";
pub const EQUAL: &'static str = "==";
pub const GREATER_OR_EQUAL: &'static str = ">=";
pub const GREATER: &'static str = ">";
pub const LITTLE: &'static str = "<";
pub const LITTLE_OR_EQUAL: &'static str = "<=";
pub const NOT_EQUAL: &'static str = "!=";
pub const AND: &'static str = "&&";
pub const OR: &'static str = "||";
pub const WHITESPACE: &'static str = " ";

const CH_DOLLA: char = '$';
const CH_DOT: char = '.';
const CH_ASTERISK: char = '*';
const CH_LARRAY: char = '[';
const CH_RARRAY: char = ']';
const CH_LPAREN: char = '(';
const CH_RPAREN: char = ')';
const CH_AT: char = '@';
const CH_QUESTION: char = '?';
const CH_COMMA: char = ',';
const CH_SEMICOLON: char = ':';
const CH_EQUAL: char = '=';
const CH_AMPERSAND: char = '&';
const CH_PIPE: char = '|';
const CH_LITTLE: char = '<';
const CH_GREATER: char = '>';
const CH_EXCLAMATION: char = '!';
const CH_SINGLE_QUOTA: char = '\'';
const CH_DOUBLE_QUOTA: char = '"';

#[derive(Debug, Clone, PartialEq)]
pub enum TokenError {
    Eof,
    Position(usize),
}

fn to_token_error(read_err: ReaderError) -> TokenError {
    match read_err {
        ReaderError::Eof => TokenError::Eof
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Absolute(usize),
    Dot(usize),
    At(usize),
    OpenArray(usize),
    CloseArray(usize),
    Asterisk(usize),
    Question(usize),
    Comma(usize),
    Split(usize),
    OpenParenthesis(usize),
    CloseParenthesis(usize),
    Key(usize, String),
    DoubleQuoted(usize, String),
    SingleQuoted(usize, String),
    Equal(usize),
    GreaterOrEqual(usize),
    Greater(usize),
    Little(usize),
    LittleOrEqual(usize),
    NotEqual(usize),
    And(usize),
    Or(usize),
    Whitespace(usize, usize),
}

impl Token {
    pub fn partial_eq(&self, other: Token) -> bool {
        self.to_simple() == other.to_simple()
    }

    pub fn simple_eq(&self, str_token: &str) -> bool {
        self.to_simple() == str_token
    }

    fn to_simple(&self) -> &'static str {
        match self {
            Token::Absolute(_) => ABSOLUTE,
            Token::Dot(_) => DOT,
            Token::At(_) => AT,
            Token::OpenArray(_) => OPEN_ARRAY,
            Token::CloseArray(_) => CLOSE_ARRAY,
            Token::Asterisk(_) => ASTERISK,
            Token::Question(_) => QUESTION,
            Token::Comma(_) => COMMA,
            Token::Split(_) => SPLIT,
            Token::OpenParenthesis(_) => OPEN_PARENTHESIS,
            Token::CloseParenthesis(_) => CLOSE_PARENTHESIS,
            Token::Key(_, _) => KEY,
            Token::DoubleQuoted(_, _) => DOUBLE_QUOTA,
            Token::SingleQuoted(_, _) => SINGLE_QUOTA,
            Token::Equal(_) => EQUAL,
            Token::GreaterOrEqual(_) => GREATER_OR_EQUAL,
            Token::Greater(_) => GREATER,
            Token::Little(_) => LITTLE,
            Token::LittleOrEqual(_) => LITTLE_OR_EQUAL,
            Token::NotEqual(_) => NOT_EQUAL,
            Token::And(_) => AND,
            Token::Or(_) => OR,
            Token::Whitespace(_, _) => WHITESPACE
        }
    }
}

fn simple_matched_token(ch: char, pos: usize) -> Option<Token> {
    match ch {
        CH_DOLLA => Some(Token::Absolute(pos)),
        CH_DOT => Some(Token::Dot(pos)),
        CH_ASTERISK => Some(Token::Asterisk(pos)),
        CH_LARRAY => Some(Token::OpenArray(pos)),
        CH_RARRAY => Some(Token::CloseArray(pos)),
        CH_LPAREN => Some(Token::OpenParenthesis(pos)),
        CH_RPAREN => Some(Token::CloseParenthesis(pos)),
        CH_AT => Some(Token::At(pos)),
        CH_QUESTION => Some(Token::Question(pos)),
        CH_COMMA => Some(Token::Comma(pos)),
        CH_SEMICOLON => Some(Token::Split(pos)),
        _ => None
    }
}

pub struct Tokenizer<'a> {
    input: PathReader<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            input: PathReader::new(input),
        }
    }

    fn single_quota(&mut self, pos: usize, ch: char) -> Result<Token, TokenError> {
        let (_, val) = self.input.take_while(|c| *c != ch).map_err(to_token_error)?;
        self.input.next_char().map_err(to_token_error)?;
        Ok(Token::SingleQuoted(pos, val))
    }

    fn double_quota(&mut self, pos: usize, ch: char) -> Result<Token, TokenError> {
        let (_, val) = self.input.take_while(|c| *c != ch).map_err(to_token_error)?;
        self.input.next_char().map_err(to_token_error)?;
        Ok(Token::DoubleQuoted(pos, val))
    }

    fn equal(&mut self, pos: usize, _: char) -> Result<Token, TokenError> {
        let (_, ch) = self.input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                self.input.next_char().map_err(to_token_error)?;
                Ok(Token::Equal(pos))
            }
            _ => Err(TokenError::Position(pos))
        }
    }

    fn not_equal(&mut self, pos: usize, _: char) -> Result<Token, TokenError> {
        let (_, ch) = self.input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                self.input.next_char().map_err(to_token_error)?;
                Ok(Token::NotEqual(pos))
            }
            _ => Err(TokenError::Position(pos))
        }
    }

    fn little(&mut self, pos: usize, _: char) -> Result<Token, TokenError> {
        let (_, ch) = self.input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                self.input.next_char().map_err(to_token_error)?;
                Ok(Token::LittleOrEqual(pos))
            }
            _ => Ok(Token::Little(pos)),
        }
    }

    fn greater(&mut self, pos: usize, _: char) -> Result<Token, TokenError> {
        let (_, ch) = self.input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                self.input.next_char().map_err(to_token_error)?;
                Ok(Token::GreaterOrEqual(pos))
            }
            _ => Ok(Token::Greater(pos)),
        }
    }

    fn and(&mut self, pos: usize, _: char) -> Result<Token, TokenError> {
        let (_, ch) = self.input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_AMPERSAND => {
                let _ = self.input.next_char().map_err(to_token_error);
                Ok(Token::And(pos))
            }
            _ => Err(TokenError::Position(pos))
        }
    }

    fn or(&mut self, pos: usize, _: char) -> Result<Token, TokenError> {
        let (_, ch) = self.input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_PIPE => {
                self.input.next_char().map_err(to_token_error)?;
                Ok(Token::Or(pos))
            }
            _ => Err(TokenError::Position(pos))
        }
    }

    fn whitespace(&mut self, pos: usize, _: char) -> Result<Token, TokenError> {
        let (_, vec) = self.input.take_while(|c| c.is_whitespace()).map_err(to_token_error)?;
        Ok(Token::Whitespace(pos, vec.len()))
    }

    fn other(&mut self, pos: usize, ch: char) -> Result<Token, TokenError> {
        let fun = |c: &char| {
            match simple_matched_token(*c, pos) {
                Some(_) => false,
                _ if c == &CH_LITTLE
                    || c == &CH_GREATER
                    || c == &CH_EQUAL
                    || c == &CH_AMPERSAND
                    || c == &CH_PIPE
                    || c == &CH_EXCLAMATION => false,
                _ => !c.is_whitespace()
            }
        };
        let (_, mut vec) = self.input.take_while(fun).map_err(to_token_error)?;
        vec.insert(0, ch);
        Ok(Token::Key(pos, vec))
    }

    pub fn next_token(&mut self) -> Result<Token, TokenError> {
        let (pos, ch) = self.input.next_char().map_err(to_token_error)?;
        match simple_matched_token(ch, pos) {
            Some(t) => Ok(t),
            None => {
                match ch {
                    CH_SINGLE_QUOTA => self.single_quota(pos, ch),
                    CH_DOUBLE_QUOTA => self.double_quota(pos, ch),
                    CH_EQUAL => self.equal(pos, ch),
                    CH_GREATER => self.greater(pos, ch),
                    CH_LITTLE => self.little(pos, ch),
                    CH_AMPERSAND => self.and(pos, ch),
                    CH_PIPE => self.or(pos, ch),
                    CH_EXCLAMATION => self.not_equal(pos, ch),
                    _ if ch.is_whitespace() => self.whitespace(pos, ch),
                    _ => self.other(pos, ch),
                }
            }
        }
    }

    fn current_pos(&self) -> usize {
        self.input.current_pos()
    }
}

pub struct TokenReader<'a> {
    origin_input: &'a str,
    err: TokenError,
    err_pos: usize,
    tokens: Vec<(usize, Token)>,
    curr_pos: Option<usize>,
}

impl<'a> TokenReader<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut tokenizer = Tokenizer::new(input);
        let mut tokens = vec![];
        loop {
            match tokenizer.next_token() {
                Ok(t) => {
                    tokens.insert(0, (tokenizer.current_pos(), t));
                }
                Err(e) => {
                    return TokenReader {
                        origin_input: input.clone(),
                        err: e,
                        err_pos: tokenizer.current_pos(),
                        tokens,
                        curr_pos: None,
                    };
                }
            }
        }
    }

    pub fn peek_is(&self, simple_token: &str) -> bool {
        match self.peek_token() {
            Ok(t) => t.simple_eq(simple_token),
            _ => false
        }
    }

    pub fn peek_token(&self) -> Result<&Token, TokenError> {
        match self.tokens.last() {
            Some((_, t)) => {
                trace!("%{:?}", t);
                Ok(t)
            }
            _ => {
                trace!("%{:?}", self.err);
                Err(self.err.clone())
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token, TokenError> {
        match self.tokens.pop() {
            Some((pos, t)) => {
                self.curr_pos = Some(pos);
                trace!("@{:?}", t);
                Ok(t)
            }
            _ => {
                trace!("@{:?}", self.err);
                Err(self.err.clone())
            }
        }
    }

    pub fn err_msg_with_pos(&self, pos: usize) -> String {
        let mut w = Vec::new();
        writeln!(&mut w, "{}", self.origin_input).unwrap();
        writeln!(&mut w, "{}", "^".repeat(pos)).unwrap();
        match std::str::from_utf8(&w[..]) {
            Ok(s) => s.to_owned(),
            Err(_) => panic!("Invalid UTF-8")
        }
    }

    pub fn err_msg(&self) -> String {
        match self.curr_pos {
            Some(pos) => {
                self.err_msg_with_pos(pos)
            }
            _ => {
                self.err_msg_with_pos(self.err_pos)
            }
        }
    }
}