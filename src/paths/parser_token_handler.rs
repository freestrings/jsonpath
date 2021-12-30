use paths::tokens::{_ParserToken, _TokenType, _TokenValue};

pub(crate) trait _ParserTokenHandler<'a, 'b> {
    fn handle<F>(&mut self, token: &_ParserToken<'b>, parse_value_reader: &F)
        where
            F: Fn(&_TokenType) -> _TokenValue<'a>;
}