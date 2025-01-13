use crate::paths::path_parser::ParserNode;
use crate::paths::tokens::{FilterToken, ParseToken};
use crate::paths::{ParserTokenHandler, StrRange};

pub trait ParserNodeVisitor<'a> {
    fn visit<F, F1>(
        &self,
        parse_node: &ParserNode,
        token_handler: &mut F,
        parse_value_reader: &F1,
    ) where
        F: ParserTokenHandler<'a>,
        F1: Fn(&StrRange) -> &'a str,
    {
        trace!("visit {:?}", parse_node);

        // FIXME When written in "match" grammar, it is determined that "tarpaulin" did not cover the test coverage.
        if &parse_node.token == &ParseToken::Absolute
            || &parse_node.token == &ParseToken::Relative
            || &parse_node.token == &ParseToken::All
            || matches!(&parse_node.token, &ParseToken::Key(_))
            || matches!(&parse_node.token, &ParseToken::Keys(_))
            || matches!(&parse_node.token, &ParseToken::Range(_, _, _))
            || matches!(&parse_node.token, &ParseToken::Union(_))
            || matches!(&parse_node.token, &ParseToken::Number(_))
            || matches!(&parse_node.token, &ParseToken::Bool(_))
        {
            token_handler.handle(&parse_node.token, parse_value_reader);
        } else if &parse_node.token == &ParseToken::In
            || &parse_node.token == &ParseToken::Leaves
        {
            if let Some(n) = &parse_node.left {
                self.visit(n, token_handler, parse_value_reader);
            }

            token_handler.handle(&parse_node.token, parse_value_reader);

            if let Some(n) = &parse_node.right {
                self.visit(n, token_handler, parse_value_reader);
            }
        } else if &parse_node.token == &ParseToken::Array {
            if let Some(n) = &parse_node.left {
                self.visit(n, token_handler, parse_value_reader);
            }

            token_handler.handle(&parse_node.token, parse_value_reader);

            if let Some(n) = &parse_node.right {
                self.visit(n, token_handler, parse_value_reader);
            }

            token_handler.handle(&ParseToken::ArrayEof, parse_value_reader);
        } else if &parse_node.token == &ParseToken::Filter(FilterToken::And)
            || &parse_node.token == &ParseToken::Filter(FilterToken::Or)
        {
            if let Some(n) = &parse_node.left {
                self.visit(n, token_handler, parse_value_reader);
            }

            if let Some(n) = &parse_node.right {
                self.visit(n, token_handler, parse_value_reader);
            }

            token_handler.handle(&parse_node.token, parse_value_reader);
        } else if matches!(&parse_node.token, &ParseToken::Filter(_)) {
            if let Some(n) = &parse_node.left {
                self.visit(n, token_handler, parse_value_reader);
            }

            if let Some(n) = &parse_node.right {
                self.visit(n, token_handler, parse_value_reader);
            }

            token_handler.handle(&parse_node.token, parse_value_reader);
        }
    }
}
