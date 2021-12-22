// use std::str::FromStr;

use paths::parser_node_visitor::_ParserNodeVisitor;
use paths::parser_token_handler::_ParserTokenHandler;
use paths::tokenizer::{StdTokenRules, TokenRules};
use paths::tokens::*;

// use super::parser_node_visitor::ParserNodeVisitor;
// use super::parser_token_handler::ParserTokenHandler;
use super::str_reader::StrRange;
use super::tokenizer::{TokenError, TokenReader};
// use super::tokens::{FilterToken, ParseToken};

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
            F: _ParserTokenHandler<'a, 'b>,
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

impl<'a, 'b> _ParserNodeVisitor<'a, 'b> for PathParser<'a, 'b> {}

struct _ParserImpl<'a, 'b> {
    token_reader: TokenReader<'a, 'b>,
    parse_node: Option<_ParserNode<'b>>,
}

impl<'a, 'b> _ParserImpl<'a, 'b> {
    pub fn compile(&mut self) -> Result<&mut Self, TokenError> {
        let node = JsonPathParserNodeBuilder {}.build(&mut self.token_reader, None)?;
        self.parse_node = Some(node);
        Ok(self)
    }
}

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

trait ParserNodeBuilder<'a> {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError>;

    // fn _parse(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError>;
    //
    // fn _validate(&mut self, _: &mut TokenReader, _: &_ParserToken<'a>) -> Result<(), TokenError> {
    //     Ok(())
    // }
}

struct JsonPathParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for JsonPathParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#json_path");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_ABSOLUTE, .. }) => {
                PathsParserNodeBuilder {}.build(token_reader, Some(_ParserNode::new(P_TOK_ABSOLUTE)))
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct PathsParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathsParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#paths");
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_DOT, .. }) => {
                token_reader.eat_token();
                PathDotParserNodeBuilder {}.build(token_reader, prev)
            }
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                token_reader.eat_token();
                token_reader.eat_whitespace();
                let node = ArrayParserNodeBuilder {}.build(token_reader, prev)?;
                self.build(token_reader, Some(node))
            }
            _ => Ok(prev.unwrap()),
        }
    }
}

struct PathDotParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathDotParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#paths_dot");
        let node = PathParserNodeBuilder {}.build(token_reader, prev)?;
        PathsParserNodeBuilder {}.build(token_reader, Some(node))
    }
}

struct ArrayParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#array");
        let ret = ArrayStartParserNodeBuilder {}.build(token_reader, prev)?;
        token_reader.eat_whitespace();
        Ok(CloseParserNodeBuilder {
            close_token: _Token::new(TOK_CLOSE_ARRAY, StrRange::new(0, 0))
        }.build(token_reader, Some(ret))?)
    }
}

struct CloseParserNodeBuilder<'a> {
    close_token: _Token<'a>,
}

impl<'a> ParserNodeBuilder<'a> for CloseParserNodeBuilder<'a> {
    fn build(&mut self, token_reader: &mut TokenReader, ret: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#close_token");
        match token_reader.next_token() {
            Ok(ref t) if t.is_type_matched(&self.close_token) => Ok(ret.unwrap()),
            _ => Err(token_reader.to_error()),
        }
    }
}

struct PathParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#path");
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_DOT, .. }) => Ok(PathLeavesParserNodeBuilder {}.build(token_reader, prev)?),
            Ok(_Token { key: TOK_ASTERISK, .. }) => {
                Ok(PathInAllParserNodeBuilder {}.build(token_reader, prev)?)
            }
            Ok(_Token { key: TOK_KEY, .. }) => Ok(PathInKeyParserNodeBuilder {}.build(token_reader, prev)?),
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                token_reader.eat_token();
                Ok(ArrayParserNodeBuilder {}.build(token_reader, prev)?)
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct PathInAllParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathInAllParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#path_in_all");
        token_reader.eat_token();
        let mut node = _ParserNode::new(P_TOK_IN);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(_ParserNode::new(P_TOK_ALL)));
        Ok(node)
    }
}

struct PathInKeyParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathInKeyParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#path_in_key");
        let mut node = _ParserNode::new(P_TOK_IN);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(KeyParserNodeBuilder {}.build(token_reader, None)?));
        Ok(node)
    }
}

struct PathLeavesParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathLeavesParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#path_leaves");
        token_reader.eat_token();
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_ASTERISK, .. }) => Ok(PathLeavesAllParserNodeBuilder {}.build(token_reader, prev)?),
            Ok(_Token { key: TOK_OPEN_ARRAY, .. }) => {
                let mut leaves_node = _ParserNode::new(P_TOK_LEAVES);
                leaves_node.left = Some(Box::new(prev.unwrap()));
                Ok(PathsParserNodeBuilder {}.build(token_reader, Some(leaves_node))?)
            }
            _ => Ok(PathLeavesKeyParserNodeBuilder {}.build(token_reader, prev)?),
        }
    }
}

struct PathLeavesAllParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathLeavesAllParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#path_leaves_all");
        token_reader.eat_token();
        let mut node = _ParserNode::new(P_TOK_LEAVES);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(_ParserNode::new(P_TOK_ALL)));
        Ok(node)
    }
}

struct PathLeavesKeyParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathLeavesKeyParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#path_leaves_key");
        let mut node = _ParserNode::new(P_TOK_LEAVES);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(KeyParserNodeBuilder {}.build(token_reader, None)?));
        Ok(node)
    }
}

struct KeyParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for KeyParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#key");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range }) => {
                Ok(_ParserNode::new_with_token_value(P_TOK_KEY, vec![range], _TokenValueType::String))
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct ArrayStartParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayStartParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#array_start");
        let mut node = _ParserNode::new(P_TOK_ARRAY);
        node.left = Some(Box::new(prev.unwrap()));
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_QUESTION, .. }) => {
                token_reader.eat_token();
                node.right = Some(Box::new(FilterParserNodeBuilder {}.build(token_reader, None)?));
                Ok(node)
            }
            Ok(_Token { key: TOK_ASTERISK, .. }) => {
                token_reader.eat_token();
                node.right = Some(Box::new(_ParserNode::new(P_TOK_ALL)));
                Ok(node)
            }
            _ => {
                node.right = Some(Box::new(ArrayValueParserNodeBuilder {}.build(token_reader, None)?));
                Ok(node)
            }
        }
    }
}

struct FilterParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for FilterParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#filter");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_OPEN_PARENTHESIS, .. }) => {
                let ret = ExprsParserNodeBuilder {}.build(token_reader, None)?;
                token_reader.eat_whitespace();
                Ok(CloseParserNodeBuilder {
                    close_token: _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(0, 0))
                }.build(token_reader, Some(ret))?)
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct ExprsParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ExprsParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        token_reader.eat_whitespace();
        debug!("#exprs");
        let node = match token_reader.peek_token() {
            Ok(_Token { key: TOK_OPEN_PARENTHESIS, .. }) => {
                token_reader.eat_token();
                trace!("\t-exprs - open_parenthesis");
                let ret = self.build(token_reader, None)?;
                token_reader.eat_whitespace();
                CloseParserNodeBuilder {
                    close_token: _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(0, 0))
                }.build(token_reader, Some(ret))?
            }
            _ => {
                trace!("\t-exprs - else");
                ExprParserNodeBuilder {}.build(token_reader, None)?
            }
        };
        token_reader.eat_whitespace();
        Ok(ConditionExprParserNodeBuilder {}.build(token_reader, Some(node))?)
    }
}

struct ConditionExprParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ConditionExprParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#condition_expr");

        match token_reader.peek_token() {
            Ok(_Token { key: TOK_AND, .. }) => {
                token_reader.eat_token();
                let mut node = _ParserNode::new(P_TOK_FILTER_AND);
                node.left = Some(Box::new(prev.unwrap()));
                node.right = Some(Box::new(ExprsParserNodeBuilder {}.build(token_reader, None)?));
                Ok(node)
            }
            Ok(_Token { key: TOK_OR, .. }) => {
                token_reader.eat_token();
                let mut node = _ParserNode::new(P_TOK_FILTER_OR);
                node.left = Some(Box::new(prev.unwrap()));
                node.right = Some(Box::new(ExprsParserNodeBuilder {}.build(token_reader, None)?));
                Ok(node)
            }
            _ => Ok(prev.unwrap()),
        }
    }
}

struct ExprParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ExprParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#expr");

        let has_prop_candidate = matches!(token_reader.peek_token(), Ok(_Token { key: TOK_AT, .. }));

        let node = TermParserNodeBuilder {}.build(token_reader, None);
        token_reader.eat_whitespace();

        if matches!(token_reader.peek_token(),
            Ok(_Token { key: TOK_EQUAL, .. })
            | Ok(_Token { key: TOK_NOT_EQUAL, .. })
            | Ok(_Token { key: TOK_LITTLE, .. })
            | Ok(_Token { key: TOK_LITTLE_OR_EQUAL, .. })
            | Ok(_Token { key: TOK_GREATER, .. })
            | Ok(_Token { key: TOK_GREATER_OR_EQUAL, .. }))
        {
            OpParserNodeBuilder {}.build(token_reader, Some(node?))
        } else if has_prop_candidate {
            node
        } else {
            Err(token_reader.to_error())
        }
    }
}

struct OpParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for OpParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#op");
        let mut node = match token_reader.next_token() {
            Ok(_Token { key: TOK_EQUAL, .. }) => _ParserNode::new(P_TOK_FILTER_EQUAL),
            Ok(_Token { key: TOK_NOT_EQUAL, .. }) => _ParserNode::new(P_TOK_FILTER_NOT_EQUAL),
            Ok(_Token { key: TOK_LITTLE, .. }) => _ParserNode::new(P_TOK_FILTER_LITTLE),
            Ok(_Token { key: TOK_LITTLE_OR_EQUAL, .. }) => _ParserNode::new(P_TOK_FILTER_LITTLE_OR_EQUAL),
            Ok(_Token { key: TOK_GREATER, .. }) => _ParserNode::new(P_TOK_FILTER_GREATER),
            Ok(_Token { key: TOK_GREATER_OR_EQUAL, .. }) => _ParserNode::new(P_TOK_FILTER_GREATER_OR_EQUAL),
            _ => {
                return Err(token_reader.to_error());
            }
        };

        token_reader.eat_whitespace();

        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(TermParserNodeBuilder {}.build(token_reader, None)?));
        Ok(node)
    }
}

struct TermParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for TermParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#term");

        if token_reader.peek_token().is_err() {
            return Err(token_reader.to_error());
        }

        let has_term_key = if let Ok(_Token { key: TOK_KEY, range }) = token_reader.peek_token() {
            Some(range.clone())
        } else {
            None
        };

        if let Some(range) = has_term_key {
            let key = token_reader.read_value(&range);
            return match key.as_bytes()[0] {
                b'-' | b'0'..=b'9' => TermNumParserNodeBuilder {}.build(token_reader, None),
                _ => BoolParserNodeBuilder {}.build(token_reader, None),
            };
        }

        match token_reader.peek_token() {
            Ok(_Token { key: TOK_AT, .. }) => {
                token_reader.eat_token();

                let node = _ParserNode::new(P_TOK_RELATIVE);
                match token_reader.peek_token() {
                    Ok(_Token { key: TOK_WHITESPACE, .. }) => {
                        token_reader.eat_whitespace();
                        Ok(node)
                    }
                    _ => PathsParserNodeBuilder {}.build(token_reader, Some(node)),
                }
            }
            Ok(_Token { key: TOK_ABSOLUTE, .. }) => {
                JsonPathParserNodeBuilder {}.build(token_reader, None)
            }
            Ok(_Token { key: TOK_DOUBLE_QUOTED, .. }) | Ok(_Token { key: TOK_SINGLE_QUOTED, .. }) => {
                ArrayQuoteValueParserNodeBuilder {}.build(token_reader, None)
            }
            _ => {
                Err(token_reader.to_error())
            }
        }
    }
}

struct ArrayQuoteValueParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayQuoteValueParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#array_quote_value");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_SINGLE_QUOTED, range }) | Ok(_Token { key: TOK_DOUBLE_QUOTED, range }) => {
                if let Ok(_Token { key: TOK_COMMA, .. }) = token_reader.peek_token() {
                    ArrayKeysParserNodeBuilder { range: Some(range) }.build(token_reader, None)
                } else {
                    Ok(_ParserNode::new_with_token_value(P_TOK_KEY, vec![range], _TokenValueType::String))
                }
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct ArrayKeysParserNodeBuilder {
    range: Option<StrRange>,
}

impl<'a> ParserNodeBuilder<'a> for ArrayKeysParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
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

        Ok(_ParserNode::new_with_token_value(P_TOK_KEYS, keys, _TokenValueType::String))
    }
}

///
/// TODO Range를 넘기면,, 파서트리 수준에서 값체크를 못하기 때문에 에러를 낼수없다.
///
struct BoolParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for BoolParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#boolean");
        if let Ok(_Token { key: TOK_KEY, range }) = token_reader.next_token() {
            let value = token_reader.read_value(&range);
            let bytes = value.as_bytes();

            let checked = match &bytes[0] {
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
            };

            if !checked {
                panic!("Invalid bool value {}", value)
            }

            return Ok(_ParserNode::new_with_token_value(P_TOK_BOOL, vec![range], _TokenValueType::Bool));
        }

        Err(token_reader.to_error())
    }
}

struct TermNumParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for TermNumParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#term_num");
        match token_reader.next_token() {
            Ok(_Token { key: TOK_KEY, range: exp_range }) => {
                match token_reader.peek_token() {
                    Ok(_Token { key: TOK_DOT, .. }) => {
                        debug!("#term_num_float");
                        token_reader.eat_token();
                        match token_reader.next_token() {
                            Ok(_Token { key: TOK_KEY, range: frac_range }) => {
                                Ok(_ParserNode::new_with_token_value(P_TOK_NUMBER,
                                                                     vec![exp_range.merge(&frac_range)],
                                                                     _TokenValueType::Float))
                            }
                            _ => Err(token_reader.to_error()),
                        }
                    }
                    _ => Ok(
                        _ParserNode::new_with_token_value(P_TOK_NUMBER,
                                                          vec![exp_range],
                                                          _TokenValueType::Int)
                    )
                }
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

struct ArrayValueParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayValueParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#array_value");
        match token_reader.peek_token() {
            Ok(_Token { key: TOK_KEY, .. }) => {
                Ok(ArrayValueKeyParserNodeBuilder {}.build(token_reader, None)?)
            }
            Ok(_Token { key: TOK_SPLIT, .. }) => {
                // token_reader.eat_token();
                _RangeParserNodeBuilder {
                    range_parser_type: _RangeParserNodeBuilder::TO,
                    range: None,
                }.build(token_reader, None)
            }
            Ok(_Token { key: TOK_DOUBLE_QUOTED, .. }) | Ok(_Token { key: TOK_SINGLE_QUOTED, .. }) => {
                ArrayQuoteValueParserNodeBuilder {}.build(token_reader, None)
            }
            Err(TokenError::Eof) => Ok(_ParserNode::new(P_TOK_END)),
            _ => {
                token_reader.eat_token();
                Err(token_reader.to_error())
            }
        }
    }
}

struct ArrayValueKeyParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayValueKeyParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#array_value_key");

        if let Ok(_Token { key: TOK_KEY, range }) = token_reader.next_token() {
            token_reader.eat_whitespace();

            match token_reader.peek_token() {
                Ok(_Token { key: TOK_COMMA, .. }) => UnionParserNodeBuilder { range: Some(range) }.build(token_reader, None),
                Ok(_Token { key: TOK_SPLIT, .. }) => _RangeParserNodeBuilder {
                    range_parser_type: _RangeParserNodeBuilder::FROM,
                    range: Some(range),
                }.build(token_reader, None),
                _ => Ok(_ParserNode::new_with_token_value(P_TOK_NUMBER, vec![range], _TokenValueType::String)),
            }
        } else {
            Err(token_reader.to_error())
        }
    }
}

struct _RangeParserNodeBuilder {
    range_parser_type: u8,
    range: Option<StrRange>,
}

impl _RangeParserNodeBuilder {
    const FROM: u8 = 1;
    const TO: u8 = 2;
    const STEP: u8 = 3;

    //
    // ':{key_range}'
    //
    fn get_key_range(&mut self, token_reader: &mut TokenReader) -> Option<StrRange> {
        token_reader.eat_whitespace();

        if let Ok(_Token { key: TOK_SPLIT, .. }) = token_reader.peek_token() {
            token_reader.eat_token();
            token_reader.eat_whitespace();

            if let Ok(_Token { key: TOK_KEY, .. }) = token_reader.peek_token() {
                if let Ok(_Token { key: TOK_KEY, range }) = token_reader.next_token() {
                    return Some(range);
                }
            }
        }

        None
    }
}

impl<'a> ParserNodeBuilder<'a> for _RangeParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
        debug!("#range {}", match self.range_parser_type {
            _RangeParserNodeBuilder::FROM => "from",
            _RangeParserNodeBuilder::TO => "to",
            _RangeParserNodeBuilder::STEP => "step",
            _ => "range_unknown"
        });

        let mut params = vec![self.range.take(), None, None];
        let mut split_count = 1;

        while matches!(token_reader.peek_token(), Ok(_Token { key: TOK_SPLIT, .. })) {
            if let Some(range) = self.get_key_range(token_reader) {
                params[split_count].replace(range);
            };

            split_count = split_count + 1;

            if split_count > 3 {
                return Err(token_reader.to_error())
            }
        }

        debug!(" - params: {:?}", params);

        //
        // from
        //  1. $.a[10:]
        //
        // to
        //  0. $[:]
        //  1. $.a[:11]
        //  2. $.a[-12:13]
        //
        // step
        //  0. $[::]
        //  1. $[::2]
        //  2. $[:3:2]
        //  3. $[0:3:2]
        //

        // TODO validate value
        match params.as_slice() {
            [None, None, None] => {
                Ok(_ParserNode::new(P_TOK_RANGE))
            }
            [Some(from), None, None] => {
                Ok(_ParserNode::new_with_token_value(P_TOK_RANGE_FROM, vec![from.clone()], _TokenValueType::Int))
            }
            [None, Some(to), None] => {
                Ok(_ParserNode::new_with_token_value(P_TOK_RANGE_TO, vec![to.clone()], _TokenValueType::Int))
            }
            [Some(from), Some(to), None] => {
                Ok(_ParserNode::new_with_token_value(P_TOK_RANGE_TO, vec![from.clone(), to.clone()], _TokenValueType::Int))
            }
            [None, None, Some(step)] => {
                Ok(_ParserNode::new_with_token_value(P_TOK_RANGE, vec![step.clone()], _TokenValueType::Int))
            }
            [None, Some(to), Some(step)] => {
                Ok(_ParserNode::new_with_token_value(P_TOK_RANGE, vec![to.clone(), step.clone()], _TokenValueType::Int))
            }
            [Some(from), Some(to), Some(step)] => {
                Ok(_ParserNode::new_with_token_value(P_TOK_RANGE, vec![from.clone(), to.clone(), step.clone()], _TokenValueType::Int))
            }
            _ => panic!("Unexpected range types")
        }
    }
}

struct UnionParserNodeBuilder {
    range: Option<StrRange>,
}

impl<'a> ParserNodeBuilder<'a> for UnionParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, _: Option<_ParserNode<'a>>) -> Result<_ParserNode<'a>, TokenError> {
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
                    // TODO 값 검증
                    values.push(range);
                }
                _ => {
                    return Err(token_reader.to_error());
                }
            }
        }

        Ok(_ParserNode::new_with_token_value(P_TOK_UNION, values, _TokenValueType::String))
    }
}

#[derive(Debug)]
struct ParserImpl<'a, 'b> {
    token_reader: TokenReader<'a, 'b>,
    parse_node: Option<_ParserNode<'b>>,
}

impl<'a, 'b> ParserImpl<'a, 'b> {
    pub fn new_with_token_rules(input: &'a str, token_rules: Box<dyn TokenRules>) -> Self {
        ParserImpl {
            token_reader: TokenReader::new_with_token_rules(input, token_rules),
            parse_node: None,
        }
    }

    pub fn compile(&mut self) -> Result<&mut Self, TokenError> {
        self.parse_node = Some(JsonPathParserNodeBuilder {}.build(&mut self.token_reader, None)?);
        Ok(self)
    }
}

trait _ParserTokenValueReader {
    fn read_value<T>(&self, token_reader: &mut TokenReader) -> T;
}

#[derive(Debug, Clone)]
pub(crate) struct _ParserNode<'a> {
    pub left: Option<Box<_ParserNode<'a>>>,
    pub right: Option<Box<_ParserNode<'a>>>,
    pub token: _ParserToken<'a>,
}

impl<'a> _ParserNode<'a> {
    pub fn new(token: &'a str) -> Self {
        _ParserNode {
            left: None,
            right: None,
            token: _ParserToken::new(token),
        }
    }

    pub fn new_with_token_value(token: &'a str, value_range: Vec<StrRange>, value_type: _TokenValueType) -> Self {
        _ParserNode {
            left: None,
            right: None,
            token: _ParserToken { key: token, value_range: Some(value_range), value_type: Some(value_type) },
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
    use paths::_ParserTokenHandler;
    use paths::path_parser::*;
    use paths::str_reader::StrRange;
    // use paths::tokens::*;

    struct NodeVisitorTestImpl<'a, 'b> {
        input: &'a str,
        stack: Vec<_ParserToken<'b>>,
    }

    impl<'a, 'b> NodeVisitorTestImpl<'a, 'b> {
        fn new(input: &'a str) -> Self {
            NodeVisitorTestImpl {
                input,
                stack: Vec::new(),
            }
        }

        fn start(&mut self) -> Result<Vec<_ParserToken<'b>>, String> {
            let parser = PathParser::compile(self.input).map_err(|_| "Token Error")?;
            let _ = parser.parse(self);
            Ok(self.stack.split_off(0))
        }
    }

    impl<'a, 'b> _ParserTokenHandler<'a, 'b> for NodeVisitorTestImpl<'a, 'b> {
        fn handle<F>(&mut self, token: &_ParserToken<'b>, _: &F)
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

    fn run(input: &str) -> Result<Vec<_ParserToken>, String> {
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
            // Ok(vec![
            //     ParseToken::Absolute,
            //     ParseToken::In,
            //     ParseToken::Key(StrRange::new(2, "aa".len()))
            // ])
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "aa".len())]), value_type: None }
            ])
        );

        assert_eq!(
            run("$.00.a"),
            // Ok(vec![
            //     ParseToken::Absolute,
            //     ParseToken::In,
            //     ParseToken::Key(StrRange::new(2, "00".len())),
            //     ParseToken::In,
            //     ParseToken::Key(StrRange::new(5, "a".len()))
            // ])
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "00".len())]), value_type: None },
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(5, "a".len())]), value_type: None },
            ])
        );

        assert_eq!(
            run("$.00.韓창.seok"),
            // Ok(vec![
            //     ParseToken::Absolute,
            //     ParseToken::In,
            //     ParseToken::Key(StrRange::new(2, "00".len())),
            //     ParseToken::In,
            //     ParseToken::Key(StrRange::new(5, "韓창".chars().map(|c| c.len_utf8()).sum())),
            //     ParseToken::In,
            //     ParseToken::Key(StrRange::new(12, "seok".len()))
            // ])
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "00".len())]), value_type: None },
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(5, "韓창".chars().map(|c| c.len_utf8()).sum())]), value_type: None },
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(12, "seok".len())]), value_type: None },
            ])
        );

        assert_eq!(
            run("$.*"),
            // Ok(vec![ParseToken::Absolute, ParseToken::In, ParseToken::All])
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken::new(P_TOK_ALL),
            ])
        );

        assert_eq!(
            run("$..*"),
            // Ok(vec![
            //     ParseToken::Absolute,
            //     ParseToken::Leaves,
            //     ParseToken::All
            // ])
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_LEAVES),
                _ParserToken::new(P_TOK_ALL),
            ])
        );

        assert_eq!(
            run("$..[0]"),
            // Ok(vec![
            //     ParseToken::Absolute,
            //     ParseToken::Leaves,
            //     ParseToken::Array,
            //     ParseToken::Number(0.0),
            //     ParseToken::ArrayEof
            // ])
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_LEAVES),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(4, "0".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.$a"),
            // Ok(vec![
            //     ParseToken::Absolute,
            //     ParseToken::In,
            //     ParseToken::Key(StrRange::new(2, "$a".len()))
            // ])
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "$a".len())]), value_type: None },
            ])
        );

        assert_eq!(
            run("$.['$a']"),
            // Ok(vec![
            //     ParseToken::Absolute,
            //     ParseToken::Array,
            //     ParseToken::Key(StrRange::new(3, "'$a'".len())),
            //     ParseToken::ArrayEof,
            // ])
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(3, "'$a'".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
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
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "book".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_RELATIVE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(11, "isbn".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        //
        // Array도 컨텍스트 In으로 간주 할거라서 중첩되면 하나만
        //
        assert_eq!(
            run("$.[*]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ALL),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.a[*]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ALL),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.a[*].가"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ALL),
                _ParserToken::new(P_TOK_ARRAY_END),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(7, '가'.len_utf8())]), value_type: None },
            ])
        );

        assert_eq!(
            run("$.a[0][1]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(4, "0".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(7, "1".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.a[1,2]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_UNION, value_range: Some(vec![StrRange::new(4, "1".len()), StrRange::new(6, "2".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        // from
        //  1. $.a[10:]
        //
        // to
        //  0. $[:]
        //  1. $.a[:11]
        //  2. $.a[-12:13]
        //
        // step
        //  0. $[::]
        //  1. $[::2]
        //  2. $[:3:2]
        //  3. $[0:3:2]

        assert_eq!(
            run("$.a[10:]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_RANGE_FROM, value_range: Some(vec![StrRange::new(4, "10".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.a[:11]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_RANGE_TO, value_range: Some(vec![StrRange::new(5, "11".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.a[-12:13]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken {
                    key: P_TOK_RANGE_TO,
                    value_range: Some(vec![
                        StrRange::new(4, "-12".len()),
                        StrRange::new(8, "13".len())
                    ]),
                    value_type: None
                },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$[0:3:2]"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken {
                    key: P_TOK_RANGE,
                    value_range: Some(vec![
                        StrRange::new(2, "0".len()),
                        StrRange::new(4, "3".len()),
                        StrRange::new(6, "2".len())
                    ]),
                    value_type: None
                },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$[:3:2]"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken {
                    key: P_TOK_RANGE,
                    value_range: Some(vec![
                        StrRange::new(3, "3".len()),
                        StrRange::new(5, "2".len())
                    ]),
                    value_type: None
                },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$[:]"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_RANGE),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$[::]"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_RANGE),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$[::2]"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken {
                    key: P_TOK_RANGE,
                    value_range: Some(vec![
                        StrRange::new(4, "2".len())
                    ]),
                    value_type: None
                },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$["a", 'b']"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken {
                    key: P_TOK_KEYS,
                    value_range: Some(vec![
                        StrRange::new(2, "\"a\"".len()),
                        StrRange::new(7, "'b'".len())
                    ]),
                    value_type: None
                },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.a[?(1>2)]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(6, "1".len())]), value_type: None },
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(8, "2".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_GREATER),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.a[?($.b>3)]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(8, "b".len())]), value_type: None },
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(10, "3".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_GREATER),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$[?($.c>@.d && 1==2)]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(6, "c".len())]), value_type: None },
                _ParserToken::new(P_TOK_RELATIVE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(10, "c".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_GREATER),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(15, "1".len())]), value_type: None },
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(18, "2".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_EQUAL),
                _ParserToken::new(P_TOK_FILTER_AND),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$[?($.c>@.d&&(1==2||3>=4))]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(6, "c".len())]), value_type: None },
                _ParserToken::new(P_TOK_RELATIVE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(10, "c".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_GREATER),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(14, "1".len())]), value_type: None },
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(17, "2".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_EQUAL),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(20, "3".len())]), value_type: None },
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(23, "4".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_GREATER_OR_EQUAL),
                _ParserToken::new(P_TOK_FILTER_OR),
                _ParserToken::new(P_TOK_FILTER_AND),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$[?(@.a<@.b)]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_RELATIVE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(6, "c".len())]), value_type: None },
                _ParserToken::new(P_TOK_RELATIVE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(10, "b".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_LITTLE),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$[*][*][*]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ALL),
                _ParserToken::new(P_TOK_ARRAY_END),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ALL),
                _ParserToken::new(P_TOK_ARRAY_END),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_ALL),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$['a']['bb']"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "'a'".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(7, "'bb'".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$.a[?(@.e==true)]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, "a".len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_RELATIVE),
                _ParserToken::new(P_TOK_IN),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(8, "e".len())]), value_type: None },
                _ParserToken { key: P_TOK_BOOL, value_range: Some(vec![StrRange::new(11, "true".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_EQUAL),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$[?(@ > 1)]"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_RELATIVE),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(8, "1".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_GREATER),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run("$[:]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken::new(P_TOK_RANGE),
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$['single\'quote']"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, r#"'single\'quote'"#.len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );

        assert_eq!(
            run(r#"$["single\"quote"]"#),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_KEY, value_range: Some(vec![StrRange::new(2, r#""single\"quote""#.len())]), value_type: None },
                _ParserToken::new(P_TOK_ARRAY_END),
            ])
        );
    }

    #[test]
    fn parse_array_float() {
        setup();

        assert_eq!(
            run("$[?(1.1<2.1)]"),
            Ok(vec![
                _ParserToken::new(P_TOK_ABSOLUTE),
                _ParserToken::new(P_TOK_ARRAY),
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(4, "1.1".len())]), value_type: None },
                _ParserToken { key: P_TOK_NUMBER, value_range: Some(vec![StrRange::new(8, "2.1".len())]), value_type: None },
                _ParserToken::new(P_TOK_FILTER_LITTLE),
                _ParserToken::new(P_TOK_ARRAY_END),
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
