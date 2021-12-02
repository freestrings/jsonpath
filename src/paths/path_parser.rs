use std::str::FromStr;

use paths::tokenizer::{StdTokenRules, TokenRules};
use paths::tokens::*;

use super::parser_node_visitor::ParserNodeVisitor;
use super::parser_token_handler::ParserTokenHandler;
use super::str_reader::StrRange;
use super::tokenizer::{TokenError, TokenReader};
use super::tokens::{FilterToken, ParseToken};

#[derive(Debug)]
pub struct PathParser<'a, 'b> {
    parser: ParserImpl<'a, 'b>,
}

impl<'a, 'b> PathParser<'a, 'b> {
    pub fn compile(input: &'a str) -> Result<Self, TokenError> {
        let mut parser = ParserImpl::new_with_token_rules(input, Box::new(StdTokenRules::new()));
        parser.compile()?;
        Ok(PathParser { parser })
    }

    pub(crate) fn parse<F>(&self, parse_token_handler: &mut F) -> Result<(), String>
        where
            F: ParserTokenHandler<'a>,
    {
        if self.parser.parse_node.is_none() {
            unreachable!()
        }

        let token_reader = &self.parser.token_reader;
        if let Some(parse_node) = self.parser.parse_node.as_ref() {
            self.visit(parse_node, parse_token_handler, &|s| {
                token_reader.read_value(s)
            });
        }

        Ok(())
    }
}

impl<'a, 'b> ParserNodeVisitor<'a> for PathParser<'a, 'b> {}

struct _ParserImpl<'a, 'b> {
    token_reader: TokenReader<'a, 'b>,
    parse_node: Option<_ParseNode<'b>>,
}

impl<'a, 'b> _ParserImpl<'a, 'b> {
    pub fn compile(&mut self) -> Result<&mut Self, TokenError> {
        let node = JsonPathParseNodeHandler {}.handle(&mut self.token_reader, None)?;
        self.parse_node = Some(node);
        Ok(self)
    }
}

const P_TOK_ABSOLUTE: &str = "Absolute";
const P_TOK_LEAVES: &str = "Leaves";
const P_TOK_IN: &str = "In";
const P_TOK_ALL: &str = "All";
const P_TOK_RANGE: &str = "Range";
const P_TOK_RANGE_TO: &str = "RangeTo";
const P_TOK_RANGE_FROM: &str = "RangeFrom";
const P_TOK_UNION: &str = "Union";
const P_TOK_ARRAY: &str = "Array";
const P_TOK_AND: &str = "And";
const P_TOK_OR: &str = "Or";
const P_TOK_RELATIVE: &str = "Relative";
const P_TOK_EOF: &str = "Eof";
const P_TOK_KEY: &str = "Key";
const P_TOK_KEYS: &str = "Keys";
const P_TOK_NUMBER: &str = "Number";
const P_TOK_BOOL: &str = "Bool";
const P_TOK_EQUAL: &str = "FilterEqual";
const P_TOK_NOT_EQUAL: &str = "FilterNotEqual";
const P_TOK_LITTLE: &str = "FilterLittle";
const P_TOK_LITTLE_OR_EQUAL: &str = "FilterLittleOrEqual";
const P_TOK_GREATER: &str = "FilterGreater";
const P_TOK_GREATER_OR_EQUAL: &str = "GreaterOrEqual";

trait ParseNodeHandler<'a> {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError>;
}

struct JsonPathParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for JsonPathParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#json_path");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_ABSOLUTE, .. }) => {
                PathsParseNodeHandler {}.handle(token_reader, Some(_ParseNode::new_token_only(P_TOK_ABSOLUTE)))
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct PathsParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for PathsParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#paths");
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_DOT, .. }) => {
                token_reader.eat_token();
                PathDotParseNodeHandler {}.handle(token_reader, prev)
            }
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                token_reader.eat_token();
                token_reader.eat_whitespace();
                let node = ArrayParseNodeHandler {}.handle(token_reader, prev)?;
                self.handle(token_reader, Some(node))
            }
            _ => Ok(prev.unwrap()),
        }
    }
}

struct PathDotParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for PathDotParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        let node = PathParseNodeHandler {}.handle(token_reader, prev)?;
        PathsParseNodeHandler {}.handle(token_reader, Some(node))
    }
}

struct ArrayParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for ArrayParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#array");
        let ret = ArrayStartParseNodeHandler {}.handle(token_reader, prev)?;
        token_reader.eat_whitespace();
        Ok(CloseParseNodeHandler {
            close_token: _Token::new(TOK_CLOSE_ARRAY, StrRange::new(0, 0))
        }.handle(token_reader, Some(ret))?)
    }
}

struct CloseParseNodeHandler<'a> {
    close_token: _Token<'a>
}

impl<'a> ParseNodeHandler<'a> for CloseParseNodeHandler<'a> {
    fn handle(&mut self, token_reader: &mut TokenReader, ret: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#close_token");
        match token_reader.next_token() {
            Ok(ref t) if t.is_type_matched(&self.close_token) => Ok(ret.unwrap()),
            _ => Err(token_reader.to_error()),
        }
    }
}

struct PathParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for PathParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#path");
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_DOT, .. }) => Ok(PathLeavesParseNodeHandler {}.handle(token_reader, prev)?),
            Ok(_Token { key: TOK_ASTERISK, .. }) => Ok(PathInAllParseNodeHandler {}.handle(token_reader, prev)?),
            Ok(_Token { key: TOK_KEY, .. }) => Ok(PathInAllParseNodeHandler {}.handle(token_reader, prev)?),
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                token_reader.eat_token();
                Ok(ArrayParseNodeHandler {}.handle(token_reader, prev)?)
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct PathInAllParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for PathInAllParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#path_in_all");
        token_reader.eat_token();
        let mut node = _ParseNode::new_token_only(P_TOK_IN);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(_ParseNode::new_token_only(P_TOK_ALL)));
        Ok(node)
    }
}

struct PathInKeyParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for PathInKeyParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#path_in_key");
        let mut node = _ParseNode::new_token_only(P_TOK_IN);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(KeyParseNodeHandler {}.handle(token_reader, None)?));
        Ok(node)
    }
}

struct PathLeavesParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for PathLeavesParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#path_leaves");
        token_reader.eat_token();
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_ASTERISK, .. }) => Ok(PathLeavesAllParseNodeHandler {}.handle(token_reader, prev)?),
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                let mut leaves_node = _ParseNode::new_token_only(P_TOK_LEAVES);
                leaves_node.left = Some(Box::new(prev.unwrap()));
                Ok(PathsParseNodeHandler {}.handle(token_reader, Some(leaves_node))?)
            }
            _ => Ok(PathLeavesKeyParseNodeHandler {}.handle(token_reader, prev)?),
        }
    }
}

struct PathLeavesAllParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for PathLeavesAllParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#path_leaves_all");
        token_reader.eat_token();
        let mut node = _ParseNode::new_token_only(P_TOK_LEAVES);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(_ParseNode::new_token_only(P_TOK_ALL)));
        Ok(node)
    }
}

struct PathLeavesKeyParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for PathLeavesKeyParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#path_leaves_key");
        let mut node = _ParseNode::new_token_only(P_TOK_LEAVES);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(KeyParseNodeHandler {}.handle(token_reader, None)?));
        Ok(node)
    }
}

struct KeyParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for KeyParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#key");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => {
                Ok(_ParseNode::new_with_token_params(P_TOK_KEY, vec![range]))
            },
            _ => Err(token_reader.to_error()),
        }
    }
}

struct ArrayStartParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for ArrayStartParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#array_start");
        let mut node = _ParseNode::new_token_only(P_TOK_ARRAY);
        node.left = Some(Box::new(prev.unwrap()));
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_QUESTION, .. }) => {
                token_reader.eat_token();
                node.right = Some(Box::new(FilterParseNodeHandler {}.handle(token_reader, None)?));
                Ok(node)
            }
            Ok(_Token { key: TOK_ASTERISK, .. }) => {
                token_reader.eat_token();
                node.right = Some(Box::new(_ParseNode::new_token_only(P_TOK_ALL)));
                Ok(node)
            }
            _ => {
                node.right = Some(Box::new(ArrayValueParseNodeHandler {}.handle(token_reader, None)?));
                Ok(node)
            },
        }
    }
}

struct FilterParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for FilterParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#filter");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_OPEN_PARENTHESIS, .. }) => {
                let ret = ExprsParseNodeHandler {}.handle(token_reader, None)?;
                token_reader.eat_whitespace();
                Ok(CloseParseNodeHandler {
                    close_token: _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(0, 0))
                }.handle(token_reader, Some(ret))?)
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct ExprsParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for ExprsParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        token_reader.eat_whitespace();
        debug!("#exprs");
        let node = match token_reader.peek_token() {
            Ok(_Token { key: TOK_OPEN_PARENTHESIS, .. }) => {
                token_reader.eat_token();
                trace!("\t-exprs - open_parenthesis");
                let ret = self.handle(token_reader, None)?;
                token_reader.eat_whitespace();
                CloseParseNodeHandler {
                    close_token: _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(0, 0))
                }.handle(token_reader, Some(ret))?
            }
            _ => {
                trace!("\t-exprs - else");
                ExprParseNodeHandler {}.handle(token_reader, None)?
            }
        };
        token_reader.eat_whitespace();
        Ok(ConditionExprParseNodeHandler {}.handle(token_reader, Some(node))?)
    }
}

struct ConditionExprParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for ConditionExprParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#condition_expr");

        match token_reader.peek_token() {
            Ok(_Token { key: TOK_AND, .. }) => {
                token_reader.eat_token();
                let mut node = _ParseNode::new_token_only(P_TOK_AND);
                node.left = Some(Box::new(prev.unwrap()));
                node.right = Some(Box::new(ExprsParseNodeHandler {}.handle(token_reader, None)?));
                Ok(node)
            }
            Ok(_Token { key: TOK_OR, .. }) => {
                token_reader.eat_token();
                let mut node = _ParseNode::new_token_only(P_TOK_OR);
                node.left = Some(Box::new(prev.unwrap()));
                node.right = Some(Box::new(ExprsParseNodeHandler {}.handle(token_reader, None)?));
                Ok(node)
            }
            _ => Ok(prev.unwrap()),
        }
    }
}

struct ExprParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for ExprParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#expr");

        let has_prop_candidate = matches!(token_reader.peek_token(), Ok(_Token { key: TOK_AT, .. }));

        let node = TermParseNodeHandler {}.handle(token_reader, None);
        token_reader.eat_whitespace();

        if matches!(token_reader.peek_token(),
            Ok(_Token { key: TOK_EQUAL, .. })
            | Ok(_Token { key: TOK_NOT_EQUAL, .. })
            | Ok(_Token { key: TOK_LITTLE, .. })
            | Ok(_Token { key: TOK_LITTLE_OR_EQUAL, .. })
            | Ok(_Token { key: TOK_GREATER, .. })
            | Ok(_Token { key: TOK_GREATER_OR_EQUAL, .. }))
        {
            OpParseNodeHandler {}.handle(token_reader, Some(node?))
        } else if has_prop_candidate {
            node
        } else {
            Err(token_reader.to_error())
        }
    }
}

struct OpParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for OpParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, prev: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#op");
        let mut node = match token_reader.next_token() {
            Ok(_Token { key: TOK_EQUAL, .. }) => _ParseNode::new_token_only(P_TOK_EQUAL),
            Ok(_Token { key: TOK_NOT_EQUAL, .. }) => _ParseNode::new_token_only(P_TOK_NOT_EQUAL),
            Ok(_Token { key: TOK_LITTLE, .. }) => _ParseNode::new_token_only(P_TOK_LITTLE),
            Ok(_Token { key: TOK_LITTLE_OR_EQUAL, .. }) => _ParseNode::new_token_only(P_TOK_LITTLE_OR_EQUAL),
            Ok(_Token { key: TOK_GREATER, .. }) => _ParseNode::new_token_only(P_TOK_GREATER),
            Ok(_Token { key: TOK_GREATER_OR_EQUAL, .. }) => _ParseNode::new_token_only(P_TOK_GREATER_OR_EQUAL),
            _ => {
                return Err(token_reader.to_error());
            }
        };

        token_reader.eat_whitespace();

        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(TermParseNodeHandler {}.handle(token_reader, None)?));
        Ok(node)
    }
}

struct TermParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for TermParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#term");

        if token_reader.peek_token().is_err() {
            return Err(token_reader.to_error());
        }

        let has_term_key = if let Ok(_Token { key: TOK_KEY, range }) = token_reader.peek_token() {
            Some(range.clone())
        } else {
            None
        };

        if let Some(s) = has_term_key {
            let key = token_reader.read_value(&s);
            return match key.as_bytes()[0] {
                b'-' | b'0'..=b'9' => TermNumParseNodeHandler {}.handle(token_reader, None),
                _ => BoolParseNodeHandler {}.handle(token_reader, None),
            };
        }

        match token_reader.peek_token() {
            Ok(_Token { key: TOK_AT, .. }) => {
                token_reader.eat_token();

                let node = _ParseNode::new_token_only(P_TOK_RELATIVE);
                match token_reader.peek_token() {
                    Ok(_Token { key: TOK_WHITESPACE, .. }) => {
                        token_reader.eat_whitespace();
                        Ok(node)
                    }
                    _ => PathsParseNodeHandler {}.handle(token_reader, Some(node)),
                }
            }
            Ok(_Token { key: TOK_ABSOLUTE, .. }) => {
                JsonPathParseNodeHandler {}.handle(token_reader, None)
            }
            Ok(_Token { key: TOK_DOUBLE_QUOTED, .. }) | Ok(_Token { key: TOK_SINGLE_QUOTED, .. }) => {
                ArrayQuoteValueParseNodeHandler {}.handle(token_reader, None)
            }
            _ => {
                Err(token_reader.to_error())
            }
        }
    }
}

struct ArrayQuoteValueParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for ArrayQuoteValueParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#array_quote_value");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_SINGLE_QUOTED, range }) | Ok(_Token { key: TOK_DOUBLE_QUOTED, range }) => {
                if let Ok(_Token { key: TOK_COMMA, .. }) = token_reader.peek_token() {
                    ArrayKeysParseNodeHandler { range: Some(range) }.handle(token_reader, None)
                } else {
                    Ok(_ParseNode::new_with_token_params(P_TOK_KEY, vec![range]))
                }
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct ArrayKeysParseNodeHandler {
    range: Option<StrRange>
}

impl<'a> ParseNodeHandler<'a> for ArrayKeysParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        let mut keys = if let Some(range) = self.range.take() {
            vec![range]
        } else {
            panic!("First key is mandatory!");
        };

        while let Ok(_Token { key: TOK_COMMA, .. }) = token_reader.peek_token() {
            token_reader.eat_token();
            token_reader.eat_whitespace();

            match token_reader.next_token() {
                Ok(_Token { key: TOK_SINGLE_QUOTED, range }) | Ok(_Token { key: TOK_DOUBLE_QUOTED, range }) => {
                    keys.push(range);
                }
                _ => return Err(token_reader.to_error()),
            }

            token_reader.eat_whitespace();
        }

        Ok(_ParseNode::new_with_token_params(P_TOK_KEYS, keys))
    }
}

///
/// TODO Range를 넘기면,, 파서트리 수준에서 값체크를 못하기 때문에 에러를 낼수없다.
///
struct BoolParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for BoolParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#boolean");
        if let Ok(_Token { key: TOK_KEY, range }) = token_reader.next_token() {
            return Ok(_ParseNode::new_with_token_params(P_TOK_BOOL, vec![range]));
        }

        Err(token_reader.to_error())
    }
}

struct TermNumParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for TermNumParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#term_num");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range: exp_range }) => {
                match token_reader.peek_token() {
                    Ok(_Token { key: TOK_DOT, .. }) => {
                        debug!("#term_num_float");
                        token_reader.eat_token();
                        match token_reader.next_token() {
                            Ok(_Token { key: TOK_KEY, range: frac_range }) => {
                                Ok(_ParseNode::new_with_token_params(P_TOK_NUMBER, vec![exp_range.merge(&frac_range)]))
                            }
                            _ => Err(token_reader.to_error()),
                        }
                    },
                    _ => Ok(_ParseNode::new_with_token_params(P_TOK_NUMBER, vec![exp_range]))
                }
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct ArrayValueParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for ArrayValueParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#array_value");
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_KEY, .. }) => {
                Ok(ArrayValueKeyParseNodeHandler {}.handle(token_reader, None)?)
            },
            Ok(_Token { key: TOK_SPLIT, .. }) => {
                token_reader.eat_token();
                RangeToParseNodeHandler {}.handle(token_reader, None)
            }
            Ok(_Token { key: TOK_DOUBLE_QUOTED, .. }) | Ok(_Token { key: TOK_SINGLE_QUOTED, .. }) => {
                ArrayQuoteValueParseNodeHandler {}.handle(token_reader, None)
            }
            Err(TokenError::Eof) => Ok(_ParseNode::new_token_only(P_TOK_EOF)),
            _ => {
                token_reader.eat_token();
                Err(token_reader.to_error())
            }
        }
    }
}

struct ArrayValueKeyParseNodeHandler;

impl<'a> ParseNodeHandler<'a> for ArrayValueKeyParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#array_value_key");

        if let Ok(_Token { key: TOK_KEY, range }) = token_reader.next_token() {
            token_reader.eat_whitespace();

            match token_reader.peek_token() {
                Ok(_Token { key: TOK_COMMA, .. }) => UnionParseNodeHandler { range: Some(range) }.handle(token_reader, None),
                Ok(_Token { key: TOK_SPLIT, .. }) => RangeFromParseNodeHandler { range: Some(range) }.handle(token_reader, None),
                _ => Ok(_ParseNode::new_with_token_params(P_TOK_NUMBER, vec![range])),
            }
        } else {
            Err(token_reader.to_error())
        }
    }
}

struct UnionParseNodeHandler {
    range: Option<StrRange>
}

impl<'a> ParseNodeHandler<'a> for UnionParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#union");
        let mut values = if let Some(range) = self.range.take() {
            vec![range]
        } else {
            panic!("First value is mandatory!");
        };

        while matches!(token_reader.peek_token(), Ok(_Token { key: TOK_COMMA, .. })) {
            token_reader.eat_token();
            token_reader.eat_whitespace();

            match token_reader.next_token() {
                Ok(_Token { key: TOK_KEY, range }) => {
                    values.push(range);
                }
                _ => {
                    return Err(token_reader.to_error());
                }
            }
        }

        Ok(_ParseNode::new_with_token_params(P_TOK_UNION, values))
    }
}

trait RangeParseValueReader {
    fn get_str_range(&mut self, token_reader: &mut TokenReader) -> Option<StrRange> {
        token_reader.eat_whitespace();

        match token_reader.peek_token() {
            Ok(_Token { key: TOK_SPLIT, .. }) => {
                token_reader.eat_token();
                token_reader.eat_whitespace();
            }
            _ => {
                return None;
            }
        }

        match token_reader.peek_token() {
            Ok(_Token { key: TOK_KEY, .. }) => {}
            _ => {
                return None;
            }
        }

        match token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => {
                Some(range)
            }
            _ => {
                unreachable!();
            }
        }
    }
}

struct RangeFromParseNodeHandler {
    range: Option<StrRange>
}

impl RangeParseValueReader for RangeFromParseNodeHandler {}

impl<'a> ParseNodeHandler<'a> for RangeFromParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#range_from");
        token_reader.eat_token();
        token_reader.eat_whitespace();

        let from_range = if let Some(from) = self.range.take() {
            from
        } else {
            panic!("From is mandatory!");
        };

        match token_reader.peek_token() {
            Ok(_Token { key: TOK_KEY, .. }) => {
                RangeParseNodeHandler { range: Some(from_range) }.handle(token_reader, None)
            },
            Ok(_Token { key: TOK_SPLIT, .. }) => match self.get_str_range(token_reader) {
                Some(step) => Ok(_ParseNode::new_with_token_params(P_TOK_RANGE_FROM, vec![from_range, step])),
                _ => Ok(_ParseNode::new_with_token_params(P_TOK_RANGE_FROM, vec![from_range])),
            },
            _ => Ok(_ParseNode::new_with_token_params(P_TOK_RANGE_FROM, vec![from_range])),
        }
    }
}

struct RangeToParseNodeHandler;

impl RangeParseValueReader for RangeToParseNodeHandler {}

impl<'a> ParseNodeHandler<'a> for RangeToParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#range_to");

        if let Some(step_range) = self.get_str_range(token_reader) {
            return Ok(_ParseNode::new_with_token_params(P_TOK_RANGE_TO, vec![step_range]));
        }

        if let Ok(_Token { key: TOK_CLOSE_ARRAY, .. }) = token_reader.peek_token() {
            return Ok(_ParseNode::new_token_only(P_TOK_RANGE_TO));
        }

        match token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range: to_range }) => {
                if let Some(step_range) = self.get_str_range(token_reader) {
                    Ok(_ParseNode::new_with_token_params(P_TOK_RANGE_TO, vec![to_range, step_range]))
                } else {
                    Err(token_reader.to_error())
                }
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct RangeParseNodeHandler {
    range: Option<StrRange>
}

impl RangeParseValueReader for RangeParseNodeHandler {}

impl<'a> ParseNodeHandler<'a> for RangeParseNodeHandler {
    fn handle(&mut self, token_reader: &mut TokenReader, _: Option<_ParseNode<'a>>) -> Result<_ParseNode<'a>, TokenError> {
        debug!("#range");

        let from_range = if let Some(range) = self.range.take() {
            range
        } else {
            panic!("From is mandatory!");
        };

        match token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range: to_range }) => {
                if let Some(step_range) = self.get_str_range(token_reader) {
                    Ok(_ParseNode::new_with_token_params(P_TOK_RANGE, vec![from_range, to_range, step_range]))
                } else {
                    Err(token_reader.to_error())
                }
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

#[derive(Debug)]
struct ParserImpl<'a, 'b> {
    token_reader: TokenReader<'a, 'b>,
    parse_node: Option<ParserNode>,
}

impl<'a, 'b> ParserImpl<'a, 'b> {
    pub fn new_with_token_rules(input: &'a str, token_rules: Box<dyn TokenRules>) -> Self {
        ParserImpl {
            token_reader: TokenReader::new_with_token_rules(input, token_rules),
            parse_node: None,
        }
    }

    fn string_to_num<F, S: FromStr>(string: &str, msg_handler: F) -> Result<S, TokenError>
        where
            F: Fn() -> TokenError,
    {
        match string.parse() {
            Ok(n) => Ok(n),
            _ => Err(msg_handler()),
        }
    }

    pub fn compile(&mut self) -> Result<&mut Self, TokenError> {
        self.parse_node = Some(self.json_path()?);
        Ok(self)
    }

    fn json_path(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#json_path");
        match self.token_reader.next_token() {
            Ok(_Token { key: TOK_ABSOLUTE, .. }) => {
                let node = self.create_node(ParseToken::Absolute);
                self.paths(node)
            }
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn paths(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#paths");
        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_DOT, .. }) => {
                self.eat_token();
                self.paths_dot(prev)
            }
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                self.eat_token();
                self.eat_whitespace();
                let node = self.array(prev)?;
                self.paths(node)
            }
            _ => Ok(prev),
        }
    }

    fn paths_dot(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#paths_dot");
        let node = self.path(prev)?;
        self.paths(node)
    }

    fn path(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#path");
        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_DOT, .. }) => self.path_leaves(prev),
            Ok(_Token { key: TOK_ASTERISK, .. }) => self.path_in_all(prev),
            Ok(_Token { key: TOK_KEY, .. }) => self.path_in_key(prev),
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                self.eat_token();
                self.array(prev)
            }
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn path_leaves(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#path_leaves");
        self.eat_token();
        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_ASTERISK, .. }) => self.path_leaves_all(prev),
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                let mut leaves_node = self.create_node(ParseToken::Leaves);
                leaves_node.left = Some(Box::new(prev));
                Ok(self.paths(leaves_node)?)
            }
            _ => self.path_leaves_key(prev),
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn path_leaves_key(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#path_leaves_key");
        Ok(ParserNode {
            token: ParseToken::Leaves,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.key()?)),
        })
    }

    #[allow(clippy::unnecessary_wraps)]
    fn path_leaves_all(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#path_leaves_all");
        self.eat_token();
        Ok(ParserNode {
            token: ParseToken::Leaves,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.create_node(ParseToken::All))),
        })
    }

    #[allow(clippy::unnecessary_wraps)]
    fn path_in_all(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#path_in_all");
        self.eat_token();
        Ok(ParserNode {
            token: ParseToken::In,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.create_node(ParseToken::All))),
        })
    }

    #[allow(clippy::unnecessary_wraps)]
    fn path_in_key(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#path_in_key");
        Ok(ParserNode {
            token: ParseToken::In,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.key()?)),
        })
    }

    fn key(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#key");
        match self.token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => Ok(self.create_node(ParseToken::Key(range))),
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn boolean(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#boolean");

        fn validation_bool_value(v: &str) -> bool {
            let b = v.as_bytes();
            !b.is_empty() && (b[0] == b't' || b[0] == b'T' || b[0] == b'f' || b[0] == b'F')
        }

        if let Ok(_Token { key: TOK_KEY, range }) = self.token_reader.next_token() {
            let v = self.token_reader.read_value(&range);
            if validation_bool_value(v) {
                return Ok(self.create_node(ParseToken::Bool(v.eq_ignore_ascii_case("true"))));
            }
        }

        Err(self.token_reader.to_error())
    }

    fn array_keys(&mut self, first_key: StrRange) -> Result<ParserNode, TokenError> {
        let mut keys = vec![first_key];

        while let Ok(_Token { key: TOK_COMMA, .. }) = self.token_reader.peek_token() {
            self.eat_token();
            self.eat_whitespace();

            match self.token_reader.next_token() {
                Ok(_Token { key: TOK_SINGLE_QUOTED, range }) | Ok(_Token { key: TOK_DOUBLE_QUOTED, range }) => {
                    keys.push(range);
                }
                _ => return Err(self.token_reader.to_error()),
            }

            self.eat_whitespace();
        }

        Ok(self.create_node(ParseToken::Keys(keys)))
    }

    fn array_quote_value(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#array_quote_value");
        let next = self.token_reader.next_token();
        match next {
            Ok(_Token { key: TOK_SINGLE_QUOTED, range }) | Ok(_Token { key: TOK_DOUBLE_QUOTED, range }) => {
                if let Ok(_Token { key: TOK_COMMA, .. }) = self.token_reader.peek_token() {
                    self.array_keys(range)
                } else {
                    Ok(self.create_node(ParseToken::Key(range)))
                }
            }
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn array_start(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#array_start");
        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_QUESTION, .. }) => {
                self.eat_token();
                Ok(ParserNode {
                    token: ParseToken::Array,
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.filter()?)),
                })
            }
            Ok(_Token { key: TOK_ASTERISK, .. }) => {
                self.eat_token();
                Ok(ParserNode {
                    token: ParseToken::Array,
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.create_node(ParseToken::All))),
                })
            }
            _ => Ok(ParserNode {
                token: ParseToken::Array,
                left: Some(Box::new(prev)),
                right: Some(Box::new(self.array_value()?)),
            }),
        }
    }

    fn array(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#array");
        let ret = self.array_start(prev)?;
        self.eat_whitespace();
        self.close_token(ret, _Token::new(TOK_CLOSE_ARRAY, StrRange::new(0, 0)))
    }

    fn array_value_key(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#array_value_key");

        if let Ok(_Token { key: TOK_KEY, range }) = self.token_reader.next_token() {
            let val = self.token_reader.read_value(&range);
            let digit = Self::string_to_num(val, || self.token_reader.to_error())?;
            self.eat_whitespace();

            match self.token_reader.peek_token() {
                Ok(_Token { key: TOK_COMMA, .. }) => self.union(digit),
                Ok(_Token { key: TOK_SPLIT, .. }) => self.range_from(digit),
                _ => Ok(self.create_node(ParseToken::Number(digit as f64))),
            }
        } else {
            Err(self.token_reader.to_error())
        }
    }

    fn array_value(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#array_value");
        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_KEY, .. }) => self.array_value_key(),
            Ok(_Token { key: TOK_SPLIT, .. }) => {
                self.eat_token();
                self.range_to()
            }
            Ok(_Token { key: TOK_DOUBLE_QUOTED, .. }) | Ok(_Token { key: TOK_SINGLE_QUOTED, .. }) => {
                self.array_quote_value()
            }
            Err(TokenError::Eof) => Ok(self.create_node(ParseToken::Eof)),
            _ => {
                self.eat_token();
                Err(self.token_reader.to_error())
            }
        }
    }

    fn union(&mut self, num: isize) -> Result<ParserNode, TokenError> {
        debug!("#union");
        let mut values = vec![num];
        while matches!(self.token_reader.peek_token(), Ok(_Token { key: TOK_COMMA, .. })) {
            self.eat_token();
            self.eat_whitespace();

            match self.token_reader.next_token() {
                Ok(_Token { key: TOK_KEY, range }) => {
                    let val = self.token_reader.read_value(&range);
                    let digit = Self::string_to_num(val, || self.token_reader.to_error())?;
                    values.push(digit);
                }
                _ => {
                    return Err(self.token_reader.to_error());
                }
            }
        }
        Ok(self.create_node(ParseToken::Union(values)))
    }

    fn range_value<S: FromStr>(&mut self) -> Result<Option<S>, TokenError> {
        self.eat_whitespace();

        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_SPLIT, .. }) => {
                self.eat_token();
                self.eat_whitespace();
            }
            _ => {
                return Ok(None);
            }
        }

        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_KEY, .. }) => {}
            _ => {
                return Ok(None);
            }
        }

        match self.token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => {
                let str_step = self.token_reader.read_value(&range);
                match Self::string_to_num(str_step, || self.token_reader.to_error()) {
                    Ok(step) => Ok(Some(step)),
                    Err(e) => Err(e),
                }
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn range_from(&mut self, from: isize) -> Result<ParserNode, TokenError> {
        debug!("#range_from");
        self.eat_token();
        self.eat_whitespace();

        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_KEY, .. }) => self.range(from),
            Ok(_Token { key: TOK_SPLIT, .. }) => match self.range_value()? {
                Some(step) => Ok(self.create_node(ParseToken::Range(Some(from), None, Some(step)))),
                _ => Ok(self.create_node(ParseToken::Range(Some(from), None, None))),
            },
            _ => Ok(self.create_node(ParseToken::Range(Some(from), None, None))),
        }
    }

    fn range_to(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#range_to");

        if let Some(step) = self.range_value()? {
            return Ok(self.create_node(ParseToken::Range(None, None, Some(step))));
        }

        if let Ok(_Token { key: TOK_CLOSE_ARRAY, .. }) = self.token_reader.peek_token() {
            return Ok(self.create_node(ParseToken::Range(None, None, None)));
        }

        match self.token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => {
                let to_str = self.token_reader.read_value(&range);
                let to = Self::string_to_num(to_str, || self.token_reader.to_error())?;
                let step = self.range_value()?;
                Ok(self.create_node(ParseToken::Range(None, Some(to), step)))
            }
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn range(&mut self, from: isize) -> Result<ParserNode, TokenError> {
        debug!("#range");
        match self.token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => {
                let str_to = self.token_reader.read_value(&range);
                let to = Self::string_to_num(str_to, || self.token_reader.to_error())?;
                let step = self.range_value()?;
                Ok(self.create_node(ParseToken::Range(Some(from), Some(to), step)))
            }
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn filter(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#filter");
        match self.token_reader.next_token() {
            Ok(_Token { key: TOK_OPEN_PARENTHESIS, .. }) => {
                let ret = self.exprs()?;
                self.eat_whitespace();
                self.close_token(ret, _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(0, 0)))
            }
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn exprs(&mut self) -> Result<ParserNode, TokenError> {
        self.eat_whitespace();
        debug!("#exprs");
        let node = match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_OPEN_PARENTHESIS, .. }) => {
                self.eat_token();
                trace!("\t-exprs - open_parenthesis");
                let ret = self.exprs()?;
                self.eat_whitespace();
                self.close_token(ret, _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(0, 0)))?
            }
            _ => {
                trace!("\t-exprs - else");
                self.expr()?
            }
        };
        self.eat_whitespace();
        self.condition_expr(node)
    }

    fn condition_expr(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#condition_expr");
        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_AND, .. }) => {
                self.eat_token();
                Ok(ParserNode {
                    token: ParseToken::Filter(FilterToken::And),
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.exprs()?)),
                })
            }
            Ok(_Token { key: TOK_OR, .. }) => {
                self.eat_token();
                Ok(ParserNode {
                    token: ParseToken::Filter(FilterToken::Or),
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.exprs()?)),
                })
            }
            _ => Ok(prev),
        }
    }

    fn expr(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#expr");

        let has_prop_candidate = matches!(self.token_reader.peek_token(), Ok(_Token { key: TOK_AT, .. }));

        let node = self.term()?;
        self.eat_whitespace();

        if matches!(self.token_reader.peek_token(),
            Ok(_Token { key: TOK_EQUAL, .. })
            | Ok(_Token { key: TOK_NOT_EQUAL, .. })
            | Ok(_Token { key: TOK_LITTLE, .. })
            | Ok(_Token { key: TOK_LITTLE_OR_EQUAL, .. })
            | Ok(_Token { key: TOK_GREATER, .. })
            | Ok(_Token { key: TOK_GREATER_OR_EQUAL, .. }))
        {
            self.op(node)
        } else if has_prop_candidate {
            Ok(node)
        } else {
            Err(self.token_reader.to_error())
        }
    }

    fn term_num(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#term_num");
        match self.token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => {
                let val = self.token_reader.read_value(&range);
                match self.token_reader.peek_token() {
                    Ok(_Token { key: TOK_DOT, .. }) => self.term_num_float(val),
                    _ => {
                        let number = Self::string_to_num(val, || self.token_reader.to_error())?;
                        Ok(self.create_node(ParseToken::Number(number)))
                    }
                }
            }
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn term_num_float(&mut self, num: &'a str) -> Result<ParserNode, TokenError> {
        debug!("#term_num_float");
        self.eat_token();
        match self.token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => {
                let frac = self.token_reader.read_value(&range);
                let number = Self::string_to_num(&[num, ".", frac].concat(), || self.token_reader.to_error())?;
                Ok(self.create_node(ParseToken::Number(number)))
            }
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn term(&mut self) -> Result<ParserNode, TokenError> {
        debug!("#term");

        if self.token_reader.peek_token().is_err() {
            return Err(self.token_reader.to_error());
        }

        let has_term_key = if let Ok(_Token { key: TOK_KEY, range }) = self.token_reader.peek_token() {
            Some(range.clone())
        } else {
            None
        };

        if let Some(s) = has_term_key {
            let key = self.token_reader.read_value(&s);
            return match key.as_bytes()[0] {
                b'-' | b'0'..=b'9' => self.term_num(),
                _ => self.boolean(),
            };
        }

        match self.token_reader.peek_token() {
            Ok(_Token { key: TOK_AT, .. }) => {
                self.eat_token();

                let node = self.create_node(ParseToken::Relative);
                match self.token_reader.peek_token() {
                    Ok(_Token { key: TOK_WHITESPACE, .. }) => {
                        self.eat_whitespace();
                        Ok(node)
                    }
                    _ => self.paths(node),
                }
            }
            Ok(_Token { key: TOK_ABSOLUTE, .. }) => {
                self.json_path()
            }
            Ok(_Token { key: TOK_DOUBLE_QUOTED, .. }) | Ok(_Token { key: TOK_SINGLE_QUOTED, .. }) => {
                self.array_quote_value()
            }
            _ => {
                Err(self.token_reader.to_error())
            }
        }
    }

    fn op(&mut self, prev: ParserNode) -> Result<ParserNode, TokenError> {
        debug!("#op");
        let token = match self.token_reader.next_token() {
            Ok(_Token { key: TOK_EQUAL, .. }) => ParseToken::Filter(FilterToken::Equal),
            Ok(_Token { key: TOK_NOT_EQUAL, .. }) => ParseToken::Filter(FilterToken::NotEqual),
            Ok(_Token { key: TOK_LITTLE, .. }) => ParseToken::Filter(FilterToken::Little),
            Ok(_Token { key: TOK_LITTLE_OR_EQUAL, .. }) => ParseToken::Filter(FilterToken::LittleOrEqual),
            Ok(_Token { key: TOK_GREATER, .. }) => ParseToken::Filter(FilterToken::Greater),
            Ok(_Token { key: TOK_GREATER_OR_EQUAL, .. }) => ParseToken::Filter(FilterToken::GreaterOrEqual),
            _ => {
                return Err(self.token_reader.to_error());
            }
        };

        self.eat_whitespace();

        Ok(ParserNode {
            token,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.term()?)),
        })
    }

    fn eat_whitespace(&mut self) {
        while let Ok(_Token { key: TOK_WHITESPACE, .. }) = self.token_reader.peek_token() {
            let _ = self.token_reader.next_token();
        }
    }

    fn eat_token(&mut self) {
        let _ = self.token_reader.next_token();
    }

    fn close_token(&mut self, ret: ParserNode, token: _Token) -> Result<ParserNode, TokenError> {
        debug!("#close_token");
        match self.token_reader.next_token() {
            Ok(ref t) if t.is_type_matched(&token) => Ok(ret),
            _ => Err(self.token_reader.to_error()),
        }
    }

    fn create_node(&mut self, token: ParseToken) -> ParserNode {
        ParserNode {
            left: None,
            right: None,
            token,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct _ParseNode<'a> {
    pub left: Option<Box<_ParseNode<'a>>>,
    pub right: Option<Box<_ParseNode<'a>>>,
    pub token: _ParseToken<'a>,
}

impl<'a> _ParseNode<'a> {
    pub fn new_token_only(token: &'a str) -> Self {
        _ParseNode {
            left: None,
            right: None,
            token: _ParseToken::new(token),
        }
    }

    pub fn new_with_token_params(token: &'a str, params: Vec<StrRange>) -> Self {
        _ParseNode {
            left: None,
            right: None,
            token: _ParseToken { key: token, data_range: Some(params) },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParserNode {
    pub left: Option<Box<ParserNode>>,
    pub right: Option<Box<ParserNode>>,
    pub token: ParseToken,
}

#[cfg(test)]
mod path_parser_tests {
    use paths::ParserTokenHandler;
    use paths::path_parser::PathParser;
    use paths::str_reader::StrRange;
    use paths::tokens::{FilterToken, ParseToken};

    struct NodeVisitorTestImpl<'a> {
        input: &'a str,
        stack: Vec<ParseToken>,
    }

    impl<'a> NodeVisitorTestImpl<'a> {
        fn new(input: &'a str) -> Self {
            NodeVisitorTestImpl {
                input,
                stack: Vec::new(),
            }
        }

        fn start(&mut self) -> Result<Vec<ParseToken>, String> {
            let parser = PathParser::compile(self.input).map_err(|_| "Token Error")?;
            let _ = parser.parse(self);
            Ok(self.stack.split_off(0))
        }
    }

    impl<'a> ParserTokenHandler<'a> for NodeVisitorTestImpl<'a> {
        fn handle<F>(&mut self, token: &ParseToken, _: &F)
            where
                F: Fn(&StrRange) -> &'a str
        {
            trace!("handle {:?}", token);
            self.stack.push(token.clone());
        }
    }

    fn setup() {
        let _ = env_logger::try_init();
    }

    fn run(input: &str) -> Result<Vec<ParseToken>, String> {
        let mut interpreter = NodeVisitorTestImpl::new(input);
        interpreter.start()
    }

    #[test]
    fn parse_error() {
        setup();

        fn invalid(path: &str) {
            assert!(run(path).is_err());
        }

        invalid("$[]");
        invalid("$[a]");
        invalid("$[?($.a)]");
        invalid("$[?(@.a > @.b]");
        invalid("$[?(@.a < @.b&&(@.c < @.d)]");
        invalid("@.");
        invalid("$..[?(a <= @.a)]"); // invalid term value
        invalid("$['a', b]");
        invalid("$[0, >=]");
        invalid("$[a:]");
        invalid("$[:a]");
        invalid("$[::a]");
        invalid("$[:>]");
        invalid("$[1:>]");
        invalid("$[1,,]");
        invalid("$[?]");
        invalid("$[?(1 = 1)]");
        invalid("$[?(1 = >)]");
    }

    #[test]
    fn parse_path() {
        setup();

        assert_eq!(
            run("$.aa"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "aa".len()))
            ])
        );

        assert_eq!(
            run("$.00.a"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "00".len())),
                ParseToken::In,
                ParseToken::Key(StrRange::new(5, "a".len()))
            ])
        );

        assert_eq!(
            run("$.00.韓창.seok"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "00".len())),
                ParseToken::In,
                ParseToken::Key(StrRange::new(5, "韓창".chars().map(|c| c.len_utf8()).sum())),
                ParseToken::In,
                ParseToken::Key(StrRange::new(12, "seok".len()))
            ])
        );

        assert_eq!(
            run("$.*"),
            Ok(vec![ParseToken::Absolute, ParseToken::In, ParseToken::All])
        );

        assert_eq!(
            run("$..*"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Leaves,
                ParseToken::All
            ])
        );

        assert_eq!(
            run("$..[0]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Leaves,
                ParseToken::Array,
                ParseToken::Number(0.0),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.$a"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "$a".len()))
            ])
        );

        assert_eq!(
            run("$.['$a']"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Key(StrRange::new(3, "'$a'".len())),
                ParseToken::ArrayEof,
            ])
        );

        if run("$.").is_ok() {
            panic!();
        }

        if run("$..").is_ok() {
            panic!();
        }

        if run("$. a").is_ok() {
            panic!();
        }
    }

    #[test]
    fn parse_array_syntax() {
        setup();

        assert_eq!(
            run("$.book[?(@.isbn)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "book".len())),
                ParseToken::Array,
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key(StrRange::new(11, "isbn".len())),
                ParseToken::ArrayEof
            ])
        );

        //
        // Array도 컨텍스트 In으로 간주 할거라서 중첩되면 하나만
        //
        assert_eq!(
            run("$.[*]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[*]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[*].가"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof,
                ParseToken::In,
                ParseToken::Key(StrRange::new(7, '가'.len_utf8()))
            ])
        );

        assert_eq!(
            run("$.a[0][1]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::Number(0_f64),
                ParseToken::ArrayEof,
                ParseToken::Array,
                ParseToken::Number(1_f64),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[1,2]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::Union(vec![1, 2]),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[10:]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::Range(Some(10), None, None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[:11]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::Range(None, Some(11), None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[-12:13]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::Range(Some(-12), Some(13), None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[0:3:2]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(Some(0), Some(3), Some(2)),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[:3:2]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, Some(3), Some(2)),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[:]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, None, None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[::]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, None, None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[::2]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, None, Some(2)),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$["a", 'b']"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Keys(vec![StrRange::new(2, "\"a\"".len()), StrRange::new(7, "'b'".len())]),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[?(1>2)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::Number(1_f64),
                ParseToken::Number(2_f64),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[?($.b>3)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(8, "b".len())),
                ParseToken::Number(3_f64),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[?($.c>@.d && 1==2)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(6, "c".len())),
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key(StrRange::new(10, "c".len())),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::Number(1_f64),
                ParseToken::Number(2_f64),
                ParseToken::Filter(FilterToken::Equal),
                ParseToken::Filter(FilterToken::And),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[?($.c>@.d&&(1==2||3>=4))]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(6, "c".len())),
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key(StrRange::new(10, "d".len())),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::Number(1_f64),
                ParseToken::Number(2_f64),
                ParseToken::Filter(FilterToken::Equal),
                ParseToken::Number(3_f64),
                ParseToken::Number(4_f64),
                ParseToken::Filter(FilterToken::GreaterOrEqual),
                ParseToken::Filter(FilterToken::Or),
                ParseToken::Filter(FilterToken::And),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[?(@.a<@.b)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key(StrRange::new(6, "a".len())),
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key(StrRange::new(10, "b".len())),
                ParseToken::Filter(FilterToken::Little),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[*][*][*]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof,
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof,
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$['a']['bb']"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Key(StrRange::new(2, "'a'".len())),
                ParseToken::ArrayEof,
                ParseToken::Array,
                ParseToken::Key(StrRange::new(7, "'bb'".len())),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[?(@.e==true)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key(StrRange::new(2, "a".len())),
                ParseToken::Array,
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key(StrRange::new(8, "e".len())),
                ParseToken::Bool(true),
                ParseToken::Filter(FilterToken::Equal),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[?(@ > 1)]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Relative,
                ParseToken::Number(1_f64),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[:]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, None, None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$['single\'quote']"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Key(StrRange::new(2, r#"'single\'quote'"#.len())),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$["single\"quote"]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Key(StrRange::new(2, r#""single\"quote""#.len())),
                ParseToken::ArrayEof
            ])
        );
    }

    #[test]
    fn parse_array_float() {
        setup();

        assert_eq!(
            run("$[?(1.1<2.1)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Number(1.1),
                ParseToken::Number(2.1),
                ParseToken::Filter(FilterToken::Little),
                ParseToken::ArrayEof
            ])
        );

        if run("$[1.1]").is_ok() {
            panic!();
        }

        if run("$[?(1.1<.2)]").is_ok() {
            panic!();
        }

        if run("$[?(1.1<2.)]").is_ok() {
            panic!();
        }

        if run("$[?(1.1<2.a)]").is_ok() {
            panic!();
        }
    }
}
