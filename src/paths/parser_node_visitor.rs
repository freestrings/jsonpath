use paths::{ParserTokenHandler, StrRange};
use paths::parser_token_handler::_ParserTokenHandler;
use paths::path_parser::*;
use paths::tokens::*;

pub trait ParserNodeVisitor<'a> {
    fn visit<F, F1>(&self, parse_node: &ParserNode, token_handler: &mut F, parse_value_reader: &F1)
        where
            F: ParserTokenHandler<'a>,
            F1: Fn(&StrRange) -> &'a str
    {
        trace!("visit {:?}", parse_node);
        match &parse_node.token {
            ParseToken::Absolute
            | ParseToken::Relative
            | ParseToken::All
            | ParseToken::Key(_)
            | ParseToken::Keys(_)
            | ParseToken::Range(_, _, _)
            | ParseToken::Union(_)
            | ParseToken::Number(_)
            | ParseToken::Bool(_) => {
                token_handler.handle(&parse_node.token, parse_value_reader);
            }
            ParseToken::In | ParseToken::Leaves => {
                if let Some(n) = &parse_node.left {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                token_handler.handle(&parse_node.token, parse_value_reader);

                if let Some(n) = &parse_node.right {
                    self.visit(&*n, token_handler, parse_value_reader);
                }
            }
            ParseToken::Array => {
                if let Some(n) = &parse_node.left {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                token_handler.handle(&parse_node.token, parse_value_reader);

                if let Some(n) = &parse_node.right {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                token_handler.handle(&ParseToken::ArrayEof, parse_value_reader);
            }
            ParseToken::Filter(_) => {
                if let Some(n) = &parse_node.left {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                if let Some(n) = &parse_node.right {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                token_handler.handle(&parse_node.token, parse_value_reader);
            }
            _ => {}
        }
    }
}

pub(crate) trait _ParserNodeVisitor<'a, 'b> {
    fn visit<F, F1>(&self, parse_node: &_ParserNode<'b>, token_handler: &mut F, parse_value_reader: &F1)
        where
            F: _ParserTokenHandler<'a, 'b>,
            F1: Fn(&StrRange) -> &'a str
    {
        trace!("visit {:?}", parse_node);
        match &parse_node.token.key {
            &P_TOK_ABSOLUTE
            | &P_TOK_RELATIVE
            | &P_TOK_ALL
            | &P_TOK_KEY
            | &P_TOK_KEYS
            | &P_TOK_RANGE
            | &P_TOK_RANGE_FROM
            | &P_TOK_RANGE_TO
            | &P_TOK_UNION
            | &P_TOK_NUMBER
            | &P_TOK_BOOL => {
                token_handler.handle(&parse_node.token, parse_value_reader);
            }
            &P_TOK_IN
            | &P_TOK_LEAVES => {
                if let Some(n) = &parse_node.left {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                token_handler.handle(&parse_node.token, parse_value_reader);

                if let Some(n) = &parse_node.right {
                    self.visit(&*n, token_handler, parse_value_reader);
                }
            }
            &P_TOK_ARRAY => {
                if let Some(n) = &parse_node.left {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                token_handler.handle(&parse_node.token, parse_value_reader);

                if let Some(n) = &parse_node.right {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                token_handler.handle(&_ParserToken::new(P_TOK_ARRAY_END), parse_value_reader);
            }
            &P_TOK_FILTER_AND
            | &P_TOK_FILTER_OR
            | &P_TOK_FILTER_EQUAL
            | &P_TOK_FILTER_NOT_EQUAL
            | &P_TOK_FILTER_LITTLE
            | &P_TOK_FILTER_LITTLE_OR_EQUAL
            | &P_TOK_FILTER_GREATER
            | &P_TOK_FILTER_GREATER_OR_EQUAL => {
                if let Some(n) = &parse_node.left {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                if let Some(n) = &parse_node.right {
                    self.visit(&*n, token_handler, parse_value_reader);
                }

                token_handler.handle(&parse_node.token, parse_value_reader);
            }
            _ => {
                debug!("parse token : {:?}", &parse_node.token);
            }
        }
    }
}