use std::result;

use jsonpath::path_reader::{
    Error,
    PathReader,
};

use super::utils;

pub const ABSOLUTE: &'static str = "Absolute";
pub const RELATIVE: &'static str = "Relative";
pub const AT: &'static str = "At";
pub const OPEN_ARRAY: &'static str = "OpenArray";
pub const CLOSE_ARRAY: &'static str = "CloseArray";
pub const ASTERISK: &'static str = "Asterisk";
pub const QUESTION: &'static str = "Question";
pub const COMMA: &'static str = "Comma";
pub const SPLIT: &'static str = "Split";
pub const OPEN_PARENTHESIS: &'static str = "OpenParenthesis";
pub const CLOSE_PARENTHESIS: &'static str = "CloseParenthesis";
pub const KEY: &'static str = "Key";
pub const STRING: &'static str = "String";
pub const SINGLE_QUOTA_LITERAL: &'static str = "SingleQuotaLiteral";
pub const FLOAT: &'static str = "Float";
pub const EQUAL: &'static str = "Equal";
pub const GREATER_OR_EQUAL: &'static str = "GreaterOrEqual";
pub const GREATER: &'static str = "Greater";
pub const LITTLE: &'static str = "Little";
pub const LITTLE_OR_EQUAL: &'static str = "LittleOrEqual";
pub const NOT_EQUAL: &'static str = "NotEqual";
pub const AND: &'static str = "And";
pub const OR: &'static str = "Or";
pub const WHITESPACE: &'static str = "Whitespace";

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

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Absolute(usize),
    Relative(usize),
    At(usize),
    OpenArray(usize),
    CloseArray(usize),
    Asterisk(usize),
    Question(usize),
    Comma(usize),
    Split(usize),
    OpenParenthesis(usize),
    CloseParenthesis(usize),
    Key(usize, Vec<char>),
    DoubleQuoted(usize, Vec<char>),
    SingleQuoted(usize, Vec<char>),
    Float(usize, Vec<char>),
    Equal(usize),
    GreaterOrEqual(usize),
    Greater(usize),
    Little(usize),
    LittleOrEqual(usize),
    NotEqual(usize),
    And(usize),
    Or(usize),
    Whitespace(usize),
}

impl Token {
    pub fn alias_of(&self, type_str: &'static str) -> bool {
        self.to_alias() == type_str
    }

    fn to_alias(&self) -> &'static str {
        match self {
            Token::Absolute(_) => ABSOLUTE,
            Token::Relative(_) => RELATIVE,
            Token::At(_) => AT,
            Token::OpenArray(_) => OPEN_ARRAY,
            Token::CloseArray(_) => CLOSE_ARRAY,
            Token::Asterisk(_) => ASTERISK,
            Token::Question(_) => QUESTION,
            Token::Comma(_) => COMMA,
            Token::Split(_) => SPLIT,
            Token::OpenParenthesis(_) => OPEN_ARRAY,
            Token::CloseParenthesis(_) => CLOSE_ARRAY,
            Token::Key(_, _) => KEY,
            Token::DoubleQuoted(_, _) => STRING,
            Token::SingleQuoted(_, _) => SINGLE_QUOTA_LITERAL,
            Token::Float(_, _) => FLOAT,
            Token::Equal(_) => EQUAL,
            Token::GreaterOrEqual(_) => GREATER_OR_EQUAL,
            Token::Greater(_) => GREATER,
            Token::Little(_) => LITTLE,
            Token::LittleOrEqual(_) => LITTLE_OR_EQUAL,
            Token::NotEqual(_) => NOT_EQUAL,
            Token::And(_) => AND,
            Token::Or(_) => OR,
            Token::Whitespace(_) => WHITESPACE
        }
    }
}

fn simple_matched_token(ch: char, pos: usize) -> Option<Token> {
    match ch {
        CH_DOLLA => Some(Token::Absolute(pos)),
        CH_DOT => Some(Token::Relative(pos)),
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
    prev_token: Option<Token>,
    error: Option<Error>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            input: PathReader::new(input),
            prev_token: None,
            error: None,
        }
    }

    pub fn get_error(&self) -> Option<&Error> {
        self.error.as_ref()
    }

    fn single_quota(&mut self, pos: usize, ch: char) -> result::Result<Token, Error> {
        let (_, vec) = self.input.take_while(|c| *c != ch)?;
        self.input.next_char()?;
        Ok(Token::SingleQuoted(pos, vec))
    }

    fn double_quota(&mut self, pos: usize, ch: char) -> result::Result<Token, Error> {
        let (_, vec) = self.input.take_while(|c| *c != ch)?;
        self.input.next_char()?;
        Ok(Token::DoubleQuoted(pos, vec))
    }

    fn equal(&mut self, pos: usize, _: char) -> result::Result<Token, Error> {
        let (_, ch) = self.input.peek_char()?;
        match ch {
            CH_EQUAL => {
                self.input.next_char()?;
                Ok(Token::Equal(pos))
            }
            _ => Err(Error::Position(pos))
        }
    }

    fn not_equal(&mut self, pos: usize, _: char) -> result::Result<Token, Error> {
        let (_, ch) = self.input.peek_char()?;
        match ch {
            CH_EQUAL => {
                self.input.next_char()?;
                Ok(Token::NotEqual(pos))
            }
            _ => Err(Error::Position(pos))
        }
    }

    fn little(&mut self, pos: usize, _: char) -> result::Result<Token, Error> {
        let (_, ch) = self.input.peek_char()?;
        match ch {
            CH_EQUAL => {
                self.input.next_char()?;
                Ok(Token::GreaterOrEqual(pos))
            }
            _ if ch.is_whitespace()
                || ch == CH_SINGLE_QUOTA
                || ch == CH_DOUBLE_QUOTA
                || ch.is_numeric()
            => Ok(Token::Greater(pos)),
            _ => Err(Error::Position(pos))
        }
    }

    fn greater(&mut self, pos: usize, _: char) -> result::Result<Token, Error> {
        let (_, ch) = self.input.peek_char()?;
        match ch {
            CH_EQUAL => {
                self.input.next_char()?;
                Ok(Token::LittleOrEqual(pos))
            }
            _ if ch.is_whitespace()
                || ch == CH_SINGLE_QUOTA
                || ch == CH_DOUBLE_QUOTA
                || ch.is_numeric()
            => Ok(Token::Little(pos)),
            _ => Err(Error::Position(pos))
        }
    }

    fn and(&mut self, pos: usize, _: char) -> result::Result<Token, Error> {
        let (_, ch) = self.input.peek_char()?;
        match ch {
            CH_AMPERSAND => {
                self.input.next_char()?;
                Ok(Token::And(pos))
            }
            _ => Err(Error::Position(pos))
        }
    }

    fn or(&mut self, pos: usize, _: char) -> result::Result<Token, Error> {
        let (_, ch) = self.input.peek_char()?;
        match ch {
            CH_PIPE => {
                self.input.next_char()?;
                Ok(Token::Or(pos))
            }
            _ => Err(Error::Position(pos))
        }
    }

    fn whitespace(&mut self, pos: usize, _: char) -> result::Result<Token, Error> {
        match self.prev_token {
            Some(Token::Absolute(_))
            | Some(Token::At(_))
            | Some(Token::Relative(_))
            | Some(Token::Question(_))
            => Err(Error::Position(pos)),
            _ => Ok(Token::Whitespace(pos))
        }
    }

    fn other(&mut self, pos: usize, ch: char) -> result::Result<Token, Error> {
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

        let (_, mut vec) = self.input.take_while(fun)?;
        vec.insert(0, ch);

        let is_float = match self.input.peek_char() {
            Ok((_, ch)) => ch == '.' && utils::vec_to_number(&vec).is_ok(),
            _ => false
        };

        if is_float {
            self.input.next_char()?;
            let (pos, decimal) = self.input.take_while(fun)?;

            if decimal.len() == 0 {
                return Err(Error::Position(pos));
            }

            vec.push('.');
            for f in decimal {
                vec.push(f);
            }
        }

        Ok(Token::Key(pos, vec))
    }

    fn next_token(&mut self) -> result::Result<Token, Error> {
        let (pos, ch) = self.input.next_char()?;

        let result = match simple_matched_token(ch, pos) {
            Some(t) => Ok(t),
            None => {
                match ch {
                    CH_SINGLE_QUOTA => self.single_quota(pos, ch),
                    CH_DOUBLE_QUOTA => self.double_quota(pos, ch),
                    CH_EQUAL => self.equal(pos, ch),
                    CH_GREATER => self.little(pos, ch),
                    CH_LITTLE => self.greater(pos, ch),
                    CH_AMPERSAND => self.and(pos, ch),
                    CH_PIPE => self.or(pos, ch),
                    CH_EXCLAMATION => self.not_equal(pos, ch),
                    _ if ch.is_whitespace() => self.whitespace(pos, ch),
                    _ => self.other(pos, ch),
                }
            }
        };

        match &result {
            &Ok(Token::Whitespace(_)) => {}
            &Ok(ref t) => self.prev_token = Some(t.clone()),
            &Err(ref e) => self.error = Some(e.clone())
        }

        result
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        match self.next_token() {
            Ok(token) => Some(token),
            Err(_) => None
        }
    }
}

#[cfg(test)]
mod tests {
    use jsonpath::path_reader::Error;
    use jsonpath::tokenizer::{
        self,
        Token,
        Tokenizer,
    };

    fn collect_token(input: &str) -> (Vec<Token>, Option<Error>) {
        let mut tokenizer = Tokenizer::new(input);
        let mut vec = vec![];
        loop {
            match tokenizer.next_token() {
                Ok(t) => if !t.alias_of(tokenizer::WHITESPACE) {
                    vec.push(t)
                },
                Err(e) => return (vec, Some(e)),
            }
        }
    }

    fn run(input: &str, expected: (Vec<Token>, Option<Error>)) {
        let (vec, err) = collect_token(input.clone());
        assert_eq!((vec, err), expected, "\"{}\"", input);
    }

    #[test]
    fn token() {
        run("$. []",
            (
                vec![
                    Token::Absolute(0),
                    Token::Relative(1),
                ]
                , Some(Error::Position(2))
            ));

        run("$..",
            (
                vec![
                    Token::Absolute(0),
                    Token::Relative(1),
                    Token::Relative(2),
                ]
                , Some(Error::Eof)
            ));

        run("[-1, 2 ]",
            (
                vec![
                    Token::OpenArray(0),
                    Token::Key(1, vec!['-', '1']),
                    Token::Comma(3),
                    Token::Key(5, vec!['2']),
                    Token::CloseArray(7),
                ]
                , Some(Error::Eof)
            ));

        run("[ 1 2 , 3 \"abc\" : -10 ]",
            (
                vec![
                    Token::OpenArray(0),
                    Token::Key(2, vec!['1']),
                    Token::Key(4, vec!['2']),
                    Token::Comma(6),
                    Token::Key(8, vec!['3']),
                    Token::DoubleQuoted(10, vec!['a', 'b', 'c']),
                    Token::Split(16),
                    Token::Key(18, vec!['-', '1', '0']),
                    Token::CloseArray(22),
                ]
                , Some(Error::Eof)
            ));

        run("?(@.a <41.01)",
            (
                vec![
                    Token::Question(0),
                    Token::OpenParenthesis(1),
                    Token::At(2),
                    Token::Relative(3),
                    Token::Key(4, vec!['a']),
                    Token::Little(6),
                    Token::Key(7, vec!['4', '1', '.', '0', '1']),
                    Token::CloseParenthesis(12),
                ]
                , Some(Error::Eof)
            ));

        run("?(@.a <4a.01)",
            (
                vec![
                    Token::Question(0),
                    Token::OpenParenthesis(1),
                    Token::At(2),
                    Token::Relative(3),
                    Token::Key(4, vec!['a']),
                    Token::Little(6),
                    Token::Key(7, vec!['4', 'a']),
                    Token::Relative(9),
                    Token::Key(10, vec!['0', '1']),
                    Token::CloseParenthesis(12),
                ]
                , Some(Error::Eof)
            ));
    }
}