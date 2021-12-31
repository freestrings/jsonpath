use super::{ParserToken, TokenType, TokenValue};

pub(crate) trait ParserTokenHandler<'a, 'b> {
    fn handle<F>(&mut self, token: &ParserToken<'b>, parse_value_reader: &F)
        where
            F: Fn(&TokenType) -> TokenValue<'a>;
}