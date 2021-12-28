use paths::tokens::{_ParserToken, _TokenType, _TokenValue};

use super::str_reader::StrRange;
use super::tokens::ParseToken;

pub(crate) trait ParserTokenHandler<'a> {
    fn handle<F>(&mut self, token: &ParseToken, parse_value_reader: &F)
        where
            F: Fn(&StrRange) -> &'a str;
}

pub(crate) trait _ParserTokenHandler<'a, 'b> {
    fn handle<F>(&mut self, token: &_ParserToken<'b>, parse_value_reader: &F)
        where
            F: Fn(&_TokenType) -> _TokenValue<'a>;
}