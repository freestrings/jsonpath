use std::collections::HashSet;
use jsonpath_parser::tokenizer_ext::ExtTokenRules;
use super::{
    ParserNode,
    ParserToken,
    Token,
    TokenType,
    TokenValue,
};
use super::parser_token_handler::ParserTokenHandler;
use super::str_reader::StrRange;
use super::tokenizer::{
    std_token_str::*,
    StdTokenRules,
    TokenError,
    TokenReader,
    TokenRules,
};

pub trait ParserNodeBuilder<'a> {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError>;
}

pub struct AbsolutePathParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for AbsolutePathParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#absolute_path");
        match token_reader.next_token() {
            Ok(Token { key: TOK_ABSOLUTE, .. }) => {
                PathsParserNodeBuilder {}.build(token_reader, Some(ParserNode::new(P_TOK_ABSOLUTE)))
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

pub struct PathsParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathsParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#paths");
        match token_reader.peek_token() {
            Ok(Token { key: TOK_DOT, .. }) => {
                token_reader.eat_token();
                PathDotParserNodeBuilder {}.build(token_reader, prev)
            }
            Ok(Token { key: TOK_OPEN_ARRAY, .. }) => {
                token_reader.eat_token();
                token_reader.eat_whitespace();
                let node = ArrayParserNodeBuilder {}.build(token_reader, prev)?;
                self.build(token_reader, Some(node))
            }
            _ => Ok(prev.unwrap()),
        }
    }
}

pub struct PathDotParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathDotParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#paths_dot");
        let node = PathParserNodeBuilder {}.build(token_reader, prev)?;
        PathsParserNodeBuilder {}.build(token_reader, Some(node))
    }
}

pub struct ArrayParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#array");
        let ret = ArrayStartParserNodeBuilder {}.build(token_reader, prev)?;
        token_reader.eat_whitespace();
        Ok(CloseParserNodeBuilder {
            close_token: Token::new(TOK_CLOSE_ARRAY, StrRange::new(0, 0))
        }.build(token_reader, Some(ret))?)
    }
}

pub struct CloseParserNodeBuilder<'a> {
    close_token: Token<'a>,
}

impl<'a> ParserNodeBuilder<'a> for CloseParserNodeBuilder<'a> {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        ret: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#close_token");
        match token_reader.next_token() {
            Ok(ref t) if t.is_type_matched(&self.close_token) => Ok(ret.unwrap()),
            _ => Err(token_reader.to_error()),
        }
    }
}

pub struct PathParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#path");
        match token_reader.peek_token() {
            Ok(Token { key: TOK_DOT, .. }) => Ok(PathLeavesParserNodeBuilder {}.build(token_reader, prev)?),
            Ok(Token { key: TOK_ASTERISK, .. }) => {
                Ok(PathInAllParserNodeBuilder {}.build(token_reader, prev)?)
            }
            Ok(Token { key: TOK_KEY, .. }) => Ok(PathInKeyParserNodeBuilder {}.build(token_reader, prev)?),
            Ok(Token { key: TOK_OPEN_ARRAY, .. }) => {
                token_reader.eat_token();
                Ok(ArrayParserNodeBuilder {}.build(token_reader, prev)?)
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

pub struct PathInAllParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathInAllParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#path_in_all");
        token_reader.eat_token();
        let mut node = ParserNode::new(P_TOK_IN);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(ParserNode::new(P_TOK_ALL)));
        Ok(node)
    }
}

pub struct PathInKeyParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathInKeyParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#path_in_key");
        let mut node = ParserNode::new(P_TOK_IN);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(KeyParserNodeBuilder {}.build(token_reader, None)?));
        Ok(node)
    }
}

pub struct PathLeavesParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathLeavesParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#path_leaves");
        token_reader.eat_token();
        match token_reader.peek_token() {
            Ok(Token { key: TOK_ASTERISK, .. }) => Ok(PathLeavesAllParserNodeBuilder {}.build(token_reader, prev)?),
            Ok(Token { key: TOK_OPEN_ARRAY, .. }) => {
                let mut leaves_node = ParserNode::new(P_TOK_LEAVES);
                leaves_node.left = Some(Box::new(prev.unwrap()));
                Ok(PathsParserNodeBuilder {}.build(token_reader, Some(leaves_node))?)
            }
            _ => Ok(PathLeavesKeyParserNodeBuilder {}.build(token_reader, prev)?),
        }
    }
}

pub struct PathLeavesAllParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathLeavesAllParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#path_leaves_all");
        token_reader.eat_token();
        let mut node = ParserNode::new(P_TOK_LEAVES);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(ParserNode::new(P_TOK_ALL)));
        Ok(node)
    }
}

pub struct PathLeavesKeyParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathLeavesKeyParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#path_leaves_key");
        let mut node = ParserNode::new(P_TOK_LEAVES);
        node.left = Some(Box::new(prev.unwrap()));
        node.right = Some(Box::new(KeyParserNodeBuilder {}.build(token_reader, None)?));
        Ok(node)
    }
}

pub struct KeyParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for KeyParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#key");
        match token_reader.next_token() {
            Ok(Token { key: TOK_KEY, range }) => {
                Ok(ParserNode::new_with_token_value(P_TOK_KEY, TokenType::String(range)))
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

pub struct ArrayStartParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayStartParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#array_start");
        let mut node = ParserNode::new(P_TOK_ARRAY);
        node.left = Some(Box::new(prev.unwrap()));
        match token_reader.peek_token() {
            Ok(Token { key: TOK_QUESTION, .. }) => {
                token_reader.eat_token();
                node.right = Some(Box::new(FilterParserNodeBuilder {}.build(token_reader, None)?));
                Ok(node)
            }
            Ok(Token { key: TOK_ASTERISK, .. }) => {
                token_reader.eat_token();
                node.right = Some(Box::new(ParserNode::new(P_TOK_ALL)));
                Ok(node)
            }
            _ => {
                node.right = Some(Box::new(ArrayValueParserNodeBuilder {}.build(token_reader, None)?));
                Ok(node)
            }
        }
    }
}

pub struct FilterParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for FilterParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#filter");
        match token_reader.next_token() {
            Ok(Token { key: TOK_OPEN_PARENTHESIS, .. }) => {
                let ret = ExprsParserNodeBuilder {}.build(token_reader, None)?;
                token_reader.eat_whitespace();
                Ok(CloseParserNodeBuilder {
                    close_token: Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(0, 0))
                }.build(token_reader, Some(ret))?)
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

pub struct ExprsParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ExprsParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        token_reader.eat_whitespace();
        debug!("#exprs");
        let node = match token_reader.peek_token() {
            Ok(Token { key: TOK_OPEN_PARENTHESIS, .. }) => {
                token_reader.eat_token();
                trace!("\t-exprs - open_parenthesis");
                let ret = self.build(token_reader, None)?;
                token_reader.eat_whitespace();
                CloseParserNodeBuilder {
                    close_token: Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(0, 0))
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

pub struct ConditionExprParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ConditionExprParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#condition_expr");

        match token_reader.peek_token() {
            Ok(Token { key: TOK_AND, .. }) => {
                token_reader.eat_token();
                let mut node = ParserNode::new(P_TOK_FILTER_AND);
                node.left = Some(Box::new(prev.unwrap()));
                node.right = Some(Box::new(ExprsParserNodeBuilder {}.build(token_reader, None)?));
                Ok(node)
            }
            Ok(Token { key: TOK_OR, .. }) => {
                token_reader.eat_token();
                let mut node = ParserNode::new(P_TOK_FILTER_OR);
                node.left = Some(Box::new(prev.unwrap()));
                node.right = Some(Box::new(ExprsParserNodeBuilder {}.build(token_reader, None)?));
                Ok(node)
            }
            _ => Ok(prev.unwrap()),
        }
    }
}

pub struct ExprParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ExprParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#expr");

        let has_prop_candidate = matches!(token_reader.peek_token(), Ok(Token { key: TOK_AT, .. }));

        let node = TermParserNodeBuilder {}.build(token_reader, None);
        token_reader.eat_whitespace();

        if matches!(token_reader.peek_token(),
            Ok(Token { key: TOK_EQUAL, .. })
            | Ok(Token { key: TOK_NOT_EQUAL, .. })
            | Ok(Token { key: TOK_LITTLE, .. })
            | Ok(Token { key: TOK_LITTLE_OR_EQUAL, .. })
            | Ok(Token { key: TOK_GREATER, .. })
            | Ok(Token { key: TOK_GREATER_OR_EQUAL, .. }))
        {
            OpParserNodeBuilder {}.build(token_reader, Some(node?))
        } else if has_prop_candidate {
            node
        } else {
            Err(token_reader.to_error())
        }
    }
}

pub struct OpParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for OpParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        prev: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#op");
        let mut node = match token_reader.next_token() {
            Ok(Token { key: TOK_EQUAL, .. }) => ParserNode::new(P_TOK_FILTER_EQUAL),
            Ok(Token { key: TOK_NOT_EQUAL, .. }) => ParserNode::new(P_TOK_FILTER_NOT_EQUAL),
            Ok(Token { key: TOK_LITTLE, .. }) => ParserNode::new(P_TOK_FILTER_LITTLE),
            Ok(Token { key: TOK_LITTLE_OR_EQUAL, .. }) => ParserNode::new(P_TOK_FILTER_LITTLE_OR_EQUAL),
            Ok(Token { key: TOK_GREATER, .. }) => ParserNode::new(P_TOK_FILTER_GREATER),
            Ok(Token { key: TOK_GREATER_OR_EQUAL, .. }) => ParserNode::new(P_TOK_FILTER_GREATER_OR_EQUAL),
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

pub struct TermParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for TermParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#term");

        if token_reader.peek_token().is_err() {
            return Err(token_reader.to_error());
        }

        let has_term_key = if let Ok(Token { key: TOK_KEY, range }) = token_reader.peek_token() {
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
            Ok(Token { key: TOK_AT, .. }) => {
                token_reader.eat_token();

                let node = ParserNode::new(P_TOK_RELATIVE);
                match token_reader.peek_token() {
                    Ok(Token { key: TOK_WHITESPACE, .. }) => {
                        token_reader.eat_whitespace();
                        Ok(node)
                    }
                    _ => PathsParserNodeBuilder {}.build(token_reader, Some(node)),
                }
            }
            Ok(Token { key: TOK_ABSOLUTE, .. }) => {
                AbsolutePathParserNodeBuilder {}.build(token_reader, None)
            }
            Ok(Token { key: TOK_DOUBLE_QUOTED, .. }) | Ok(Token { key: TOK_SINGLE_QUOTED, .. }) => {
                ArrayQuoteValueParserNodeBuilder {}.build(token_reader, None)
            }
            _ => {
                Err(token_reader.to_error())
            }
        }
    }
}

pub struct ArrayQuoteValueParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayQuoteValueParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#array_quote_value");
        match token_reader.next_token() {
            Ok(Token { key: TOK_SINGLE_QUOTED, range })
            | Ok(Token { key: TOK_DOUBLE_QUOTED, range }) => {
                if let Ok(Token { key: TOK_COMMA, .. }) = token_reader.peek_token() {
                    ArrayKeysParserNodeBuilder { range: Some(range) }.build(token_reader, None)
                } else {
                    Ok(ParserNode::new_with_token_value(P_TOK_KEY, TokenType::String(range)))
                }
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

pub struct ArrayKeysParserNodeBuilder {
    range: Option<StrRange>,
}

impl<'a> ParserNodeBuilder<'a> for ArrayKeysParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        let mut keys = if let Some(range) = self.range.take() {
            vec![TokenType::String(range)]
        } else {
            panic!("First key is mandatory!");
        };

        while let Ok(Token { key: TOK_COMMA, .. }) = token_reader.peek_token() {
            token_reader.eat_token();
            token_reader.eat_whitespace();

            match token_reader.next_token() {
                Ok(Token { key: TOK_SINGLE_QUOTED, range }) | Ok(Token { key: TOK_DOUBLE_QUOTED, range }) => {
                    keys.push(TokenType::String(range));
                }
                _ => return Err(token_reader.to_error()),
            }

            token_reader.eat_whitespace();
        }

        Ok(ParserNode::new_with_token_values(P_TOK_KEYS, keys))
    }
}

pub struct BoolParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for BoolParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#boolean");
        if let Ok(Token { key: TOK_KEY, range }) = token_reader.next_token() {
            let t = TokenType::Bool(range);
            t.validate_token_type(token_reader).map_err(|r| TokenError::Position(r.pos))?;
            return Ok(ParserNode::new_with_token_value(P_TOK_BOOL, t));
        }

        Err(token_reader.to_error())
    }
}

pub struct TermNumParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for TermNumParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#term_num");

        match token_reader.next_token() {
            Ok(Token { key: TOK_KEY, range: exp_range }) => {
                match token_reader.peek_token() {
                    Ok(Token { key: TOK_DOT, .. }) => {
                        debug!("#term_num_float");

                        token_reader.eat_token();
                        match token_reader.next_token() {
                            Ok(Token { key: TOK_KEY, range: frac_range }) => {
                                let range = exp_range.merge(&frac_range);
                                let t = TokenType::Float(range);
                                t.validate_token_type(token_reader).map_err(|r| TokenError::Position(r.pos))?;
                                Ok(ParserNode::new_with_token_value(P_TOK_NUMBER, t))
                            }
                            _ => Err(token_reader.to_error()),
                        }
                    }
                    _ => {
                        let t = TokenType::Int(exp_range);
                        t.validate_token_type(token_reader).map_err(|r| TokenError::Position(r.pos))?;
                        Ok(ParserNode::new_with_token_value(P_TOK_NUMBER, t))
                    }
                }
            }
            _ => Err(token_reader.to_error()),
        }
    }
}

pub struct ArrayValueParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayValueParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#array_value");
        match token_reader.peek_token() {
            Ok(Token { key: TOK_KEY, .. }) => {
                Ok(ArrayValueKeyParserNodeBuilder {}.build(token_reader, None)?)
            }
            Ok(Token { key: TOK_SPLIT, .. }) => {
                RangeParserNodeBuilder {
                    range_parser_type: RangeParserNodeBuilder::TO,
                    range: None
                }.build(token_reader, None)
            }
            Ok(Token { key: TOK_DOUBLE_QUOTED, .. }) | Ok(Token { key: TOK_SINGLE_QUOTED, .. }) => {
                ArrayQuoteValueParserNodeBuilder {}.build(token_reader, None)
            }
            Err(TokenError::Eof) => Ok(ParserNode::new(P_TOK_END)),
            _ => {
                token_reader.eat_token();
                Err(token_reader.to_error())
            }
        }
    }
}

pub struct ArrayValueKeyParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for ArrayValueKeyParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#array_value_key");

        if let Ok(Token { key: TOK_KEY, range }) = token_reader.next_token() {
            token_reader.eat_whitespace();

            match token_reader.peek_token() {
                Ok(Token { key: TOK_COMMA, .. }) => UnionParserNodeBuilder { range: Some(range) }.build(token_reader, None),
                Ok(Token { key: TOK_SPLIT, .. }) => RangeParserNodeBuilder {
                    range_parser_type: RangeParserNodeBuilder::FROM,
                    range: Some(range),
                }.build(token_reader, None),
                Ok(Token { key: TOK_DOUBLE_QUOTED, .. })
                | Ok(Token { key: TOK_SINGLE_QUOTED, .. }) => {
                    Ok(ParserNode::new_with_token_value(P_TOK_NUMBER, TokenType::String(range)))
                }
                _ => {
                    let t = TokenType::Int(range);
                    t.validate_token_type(token_reader).map_err(|r| TokenError::Position(r.pos))?;
                    Ok(ParserNode::new_with_token_value(P_TOK_NUMBER, t))
                },
            }
        } else {
            Err(token_reader.to_error())
        }
    }
}

pub struct RangeParserNodeBuilder {
    range_parser_type: u8,
    range: Option<StrRange>,
}

impl RangeParserNodeBuilder {
    const FROM: u8 = 1;
    const TO: u8 = 2;
    const STEP: u8 = 3;

    //
    // ':{key_range}'
    //
    fn get_key_range(&mut self, token_reader: &mut TokenReader) -> Option<StrRange> {
        token_reader.eat_whitespace();

        if let Ok(Token { key: TOK_SPLIT, .. }) = token_reader.peek_token() {
            token_reader.eat_token();
            token_reader.eat_whitespace();

            if let Ok(Token { key: TOK_KEY, .. }) = token_reader.peek_token() {
                if let Ok(Token { key: TOK_KEY, range }) = token_reader.next_token() {
                    return Some(range);
                }
            }
        }

        None
    }
}

impl<'a> ParserNodeBuilder<'a> for RangeParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#range {}", match self.range_parser_type {
            RangeParserNodeBuilder::FROM => "from",
            RangeParserNodeBuilder::TO => "to",
            RangeParserNodeBuilder::STEP => "step",
            _ => "range_unknown"
        });

        let mut params = vec![self.range.take(), None, None];
        let mut split_count = 1;

        while matches!(token_reader.peek_token(), Ok(Token { key: TOK_SPLIT, .. })) {
            if let Some(range) = self.get_key_range(token_reader) {
                params[split_count].replace(range);
            };

            split_count = split_count + 1;

            if split_count > 3 {
                return Err(token_reader.to_error())
            }
        }

        debug!(" - params: {:?}", params);

        fn validate(
            range: &StrRange,
            token_reader: &mut TokenReader
        ) -> Result<TokenType, TokenError> {
            let t = TokenType::Int(range.clone());
            t.validate_token_type(token_reader).map_err(|r| TokenError::Position(r.pos))?;
            Ok(t)
        }

        fn validate_all(
            ranges: Vec<&StrRange>,
            token_reader: &mut TokenReader
        ) -> Result<Vec<TokenType>, TokenError> {
            let types: Vec<TokenType> = ranges.iter().map(|r| TokenType::Int(StrRange::new(r.pos, r.offset))).collect();
            for t in types.iter() {
                t.validate_token_type(token_reader).map_err(|r| TokenError::Position(r.pos))?;
            }
            Ok(types)
        }
        //
        // from
        //  1. $.a[10:]
        //  2. $[1::2]
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
        match params.as_slice() {
            [None, None, None] => {
                Ok(ParserNode::new(P_TOK_RANGE))
            }
            [Some(from), None, None] => {
                Ok(ParserNode::new_with_token_value(P_TOK_RANGE_FROM, validate(from, token_reader)?))
            }
            [None, Some(to), None] => {
                Ok(ParserNode::new_with_token_value(P_TOK_RANGE_TO, validate(to, token_reader)?))
            }
            [Some(from), Some(to), None] => {
                Ok(ParserNode::new_with_token_values(P_TOK_RANGE_TO, validate_all(vec![from, to], token_reader)?))
            }
            [None, None, Some(step)] => {
                Ok(ParserNode::new_with_token_value(P_TOK_RANGE, validate(step, token_reader)?))
            }
            [None, Some(to), Some(step)] => {
                Ok(ParserNode::new_with_token_values(P_TOK_RANGE, validate_all(vec![to, step], token_reader)?))
            }
            [Some(from), None, Some(step)] => {
                Ok(ParserNode::new_with_token_values(P_TOK_RANGE_FROM, validate_all(vec![from, step], token_reader)?))
            }
            [Some(from), Some(to), Some(step)] => {
                Ok(ParserNode::new_with_token_values(P_TOK_RANGE, validate_all(vec![from, to, step], token_reader)?))
            }
            _ => panic!("Unexpected range types: {:?}", params)
        }
    }
}

pub struct UnionParserNodeBuilder {
    range: Option<StrRange>,
}

impl<'a> ParserNodeBuilder<'a> for UnionParserNodeBuilder {
    fn build(
        &mut self,
        token_reader: &mut TokenReader,
        _: Option<ParserNode<'a>>
    ) -> Result<ParserNode<'a>, TokenError> {
        debug!("#union");

        let mut values = if let Some(range) = self.range.take() {
            vec![TokenType::String(range)]
        } else {
            panic!("First value is mandatory!");
        };

        while matches!(token_reader.peek_token(), Ok(Token { key: TOK_COMMA, .. })) {
            token_reader.eat_token();
            token_reader.eat_whitespace();

            match token_reader.next_token() {
                Ok(Token { key: TOK_KEY, range }) => {
                    values.push(TokenType::String(range));
                }
                _ => {
                    return Err(token_reader.to_error());
                }
            }
        }

        Ok(ParserNode::new_with_token_values(P_TOK_UNION, values))
    }
}

pub(super) trait ParserNodeVisitor<'a, 'b> {
    fn visit<F, F1>(
        &self,
        parse_node: &ParserNode<'b>,
        token_handler: &mut F,
        parse_value_reader: &F1
    ) where F: ParserTokenHandler<'a, 'b>,
            F1: Fn(&TokenType) -> TokenValue<'a> {
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

                token_handler.handle(&ParserToken::new(P_TOK_ARRAY_END), parse_value_reader);
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

// TODO
// import를 하면 enable 되게 하고 싶은디..
//
pub struct JsonPathParserOption {
    extended_tokens: HashSet<char>,
}

#[derive(Debug)]
pub struct JsonPathParser<'a, 'b> {
    parser: ParserImpl<'a, 'b>,
}

impl<'a, 'b> JsonPathParser<'a, 'b> {
    pub fn compile(input: &'a str) -> Result<Self, TokenError> {
        let mut parser = ParserImpl::new(input, Box::new(StdTokenRules::default()));
        parser.compile()?;
        Ok(JsonPathParser { parser })
    }

    pub fn compile_with_option(input: &'a str, options: JsonPathParserOption) -> Result<Self, TokenError> {
        if options.extended_tokens.len() == 0 {
            return Self::compile(input)
        }

        let mut parser = ParserImpl::new(input, Box::new(ExtTokenRules::default()));
        parser.compile()?;
        Ok(JsonPathParser { parser })
    }

    pub(crate) fn parse<F>(&self, parse_token_handler: &mut F) -> Result<(), String>
        where
            F: ParserTokenHandler<'a, 'b>,
    {
        if self.parser.parse_node.is_none() {
            unreachable!()
        }

        let token_reader = &self.parser.token_reader;
        if let Some(parse_node) = self.parser.parse_node.as_ref() {
            self.visit(parse_node, parse_token_handler, &|t| {
                match t {
                    TokenType::String(range) => {
                        TokenValue::String(token_reader.read_value(range))
                    }
                    TokenType::Int(range) => {
                        let v = token_reader.read_value(range);
                        TokenValue::Int(v.parse().expect(format!("Expected int but: {}", v).as_str()))
                    }
                    TokenType::Float(range) => {
                        let v = token_reader.read_value(range);
                        TokenValue::Float(v.parse().expect(format!("Expected float but: {}", v).as_str()))
                    }
                    TokenType::Bool(range) => {
                        let v = token_reader.read_value(range);
                        TokenValue::Bool(v.parse().expect(format!("Expected bool but: {}", v).as_str()))
                    }
                }
            });
        }

        Ok(())
    }
}

impl<'a, 'b> ParserNodeVisitor<'a, 'b> for JsonPathParser<'a, 'b> {}

#[derive(Debug)]
struct ParserImpl<'a, 'b> {
    token_reader: TokenReader<'a, 'b>,
    parse_node: Option<ParserNode<'b>>,
}

impl<'a, 'b> ParserImpl<'a, 'b> {
    pub fn new(input: &'a str, token_rules: Box<dyn TokenRules>) -> Self {
        ParserImpl {
            token_reader: TokenReader::new(input, token_rules),
            parse_node: None,
        }
    }

    pub fn compile(&mut self) -> Result<&mut Self, TokenError> {
        self.parse_node = Some(AbsolutePathParserNodeBuilder {}.build(&mut self.token_reader, None)?);
        Ok(self)
    }
}

#[cfg(test)]
mod path_parser_tests {
    use jsonpath_parser::{ParserToken, TokenType, TokenValue};
    use jsonpath_parser::parser_token_handler::ParserTokenHandler;
    use jsonpath_parser::std_token_str::*;
    use jsonpath_parser::str_reader::StrRange;
    use JsonPathParser;

    struct NodeVisitorTestImpl<'a, 'b> {
        input: &'a str,
        stack: Vec<ParserToken<'b>>,
    }

    impl<'a, 'b> NodeVisitorTestImpl<'a, 'b> {
        fn new(input: &'a str) -> Self {
            NodeVisitorTestImpl {
                input,
                stack: Vec::new(),
            }
        }

        fn start(&mut self) -> Result<Vec<ParserToken<'b>>, String> {
            let parser = JsonPathParser::compile(self.input).map_err(|_| "Token Error")?;
            let _ = parser.parse(self);
            Ok(self.stack.split_off(0))
        }
    }

    impl<'a, 'b> ParserTokenHandler<'a, 'b> for NodeVisitorTestImpl<'a, 'b> {
        fn handle<F>(&mut self, token: &ParserToken<'b>, _: &F)
            where
                F: Fn(&'a TokenType) -> TokenValue
        {
            trace!("handle {:?}", token);
            self.stack.push(token.clone());
        }
    }

    fn setup() {
        let _ = env_logger::try_init();
    }

    fn run(input: &str) -> Result<Vec<ParserToken>, String> {
        let mut interpreter = NodeVisitorTestImpl::new(input);
        interpreter.start()
    }

    #[test]
    fn parse_error() {
        setup();

        fn invalid(path: &str) {
            assert!(run(path).is_err(), "{}", path);
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
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "aa".len()))]) }
            ]),
            "$.aa"
        );

        assert_eq!(
            run("$.00.a"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "00".len()))]) },
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(5, "a".len()))]) },
            ]),
            "$.00.a"
        );

        assert_eq!(
            run("$.00.韓창.seok"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "00".len()))]) },
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(5, "韓창".chars().map(|c| c.len_utf8()).sum()))]) },
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(12, "seok".len()))]) },
            ]),
            "$.00.韓창.seok"
        );

        assert_eq!(
            run("$.*"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken::new(P_TOK_ALL),
            ]),
            "$.*"
        );

        assert_eq!(
            run("$..*"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_LEAVES),
                ParserToken::new(P_TOK_ALL),
            ]),
            "$..*"
        );

        assert_eq!(
            run("$..[0]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_LEAVES),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(4, "0".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$..[0]"
        );

        assert_eq!(
            run("$.$a"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "$a".len()))]) },
            ]),
            "$.$a"
        );

        assert_eq!(
            run("$.['$a']"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(3, "'$a'".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.['$a']"
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
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "book".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_RELATIVE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(11, "isbn".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.book[?(@.isbn)]"
        );

        //
        // Array도 컨텍스트 In으로 간주 할거라서 중첩되면 하나만
        //
        assert_eq!(
            run("$.[*]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ALL),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.[*]"
        );

        assert_eq!(
            run("$.a[*]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ALL),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[*]"
        );

        assert_eq!(
            run("$.a[*].가"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ALL),
                ParserToken::new(P_TOK_ARRAY_END),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(7, '가'.len_utf8()))]) },
            ]),
            "$.a[*].가"
        );

        assert_eq!(
            run("$.a[0][1]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(4, "0".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(7, "1".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[0][1]"
        );

        assert_eq!(
            run("$.a[1,2]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_UNION, token_type: Some(vec![TokenType::String(StrRange::new(4, "1".len())), TokenType::String(StrRange::new(6, "2".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[1,2]"
        );

        // from
        //  1. $.a[10:]
        //  2. $[1::2]
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
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_RANGE_FROM, token_type: Some(vec![TokenType::Int(StrRange::new(4, "10".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[10:]"
        );
        //
        assert_eq!(
            run("$.a[:11]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_RANGE_TO, token_type: Some(vec![TokenType::Int(StrRange::new(5, "11".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[:11]"
        );

        assert_eq!(
            run("$.a[-12:13]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken {
                    key: P_TOK_RANGE_TO,
                    token_type: Some(vec![
                        TokenType::Int(StrRange::new(4, "-12".len())),
                        TokenType::Int(StrRange::new(8, "13".len()))
                    ])
                },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[-12:13]"
        );

        assert_eq!(
            run(r#"$[0:3:2]"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken {
                    key: P_TOK_RANGE,
                    token_type: Some(vec![
                        TokenType::Int(StrRange::new(2, "0".len())),
                        TokenType::Int(StrRange::new(4, "3".len())),
                        TokenType::Int(StrRange::new(6, "2".len()))
                    ])
                },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            r#"$[0:3:2]"#
        );

        assert_eq!(
            run(r#"$[:3:2]"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken {
                    key: P_TOK_RANGE,
                    token_type: Some(vec![
                        TokenType::Int(StrRange::new(3, "3".len())),
                        TokenType::Int(StrRange::new(5, "2".len()))
                    ])
                },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            r#"$[:3:2]"#
        );

        assert_eq!(
            run(r#"$[:]"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_RANGE),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            r#"$[:]"#
        );

        assert_eq!(
            run(r#"$[::]"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_RANGE),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            r#"$[::]"#
        );

        assert_eq!(
            run(r#"$[::2]"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken {
                    key: P_TOK_RANGE,
                    token_type: Some(vec![TokenType::Int(StrRange::new(4, "2".len()))]),
                },
                ParserToken::new(P_TOK_ARRAY_END)
            ]),
            r#"$[::2]"#
        );

        assert_eq!(
            run(r#"$["a", 'b']"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken {
                    key: P_TOK_KEYS,
                    token_type: Some(vec![
                        TokenType::String(StrRange::new(2, "\"a\"".len())),
                        TokenType::String(StrRange::new(7, "'b'".len()))
                    ])
                },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            r#"$["a", 'b']"#
        );

        assert_eq!(
            run("$.a[?(1>2)]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(6, "1".len()))]) },
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(8, "2".len()))]) },
                ParserToken::new(P_TOK_FILTER_GREATER),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[?(1>2)]"
        );

        assert_eq!(
            run("$.a[?($.b>3)]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(8, "b".len()))]) },
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(10, "3".len()))]) },
                ParserToken::new(P_TOK_FILTER_GREATER),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[?($.b>3)]"
        );

        assert_eq!(
            run("$[?($.c>@.d && 1==2)]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(6, "c".len()))]) },
                ParserToken::new(P_TOK_RELATIVE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(10, "c".len()))]) },
                ParserToken::new(P_TOK_FILTER_GREATER),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(15, "1".len()))]) },
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(18, "2".len()))]) },
                ParserToken::new(P_TOK_FILTER_EQUAL),
                ParserToken::new(P_TOK_FILTER_AND),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$[?($.c>@.d && 1==2)]"
        );

        assert_eq!(
            run("$[?($.c>@.d&&(1==2||3>=4))]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(6, "c".len()))]) },
                ParserToken::new(P_TOK_RELATIVE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(10, "c".len()))]) },
                ParserToken::new(P_TOK_FILTER_GREATER),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(14, "1".len()))]) },
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(17, "2".len()))]) },
                ParserToken::new(P_TOK_FILTER_EQUAL),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(20, "3".len()))]) },
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(23, "4".len()))]) },
                ParserToken::new(P_TOK_FILTER_GREATER_OR_EQUAL),
                ParserToken::new(P_TOK_FILTER_OR),
                ParserToken::new(P_TOK_FILTER_AND),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$[?($.c>@.d&&(1==2||3>=4))]"
        );

        assert_eq!(
            run("$[?(@.a<@.b)]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_RELATIVE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(6, "c".len()))]) },
                ParserToken::new(P_TOK_RELATIVE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(10, "b".len()))]) },
                ParserToken::new(P_TOK_FILTER_LITTLE),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$[?(@.a<@.b)]"
        );

        assert_eq!(
            run("$[*][*][*]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ALL),
                ParserToken::new(P_TOK_ARRAY_END),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ALL),
                ParserToken::new(P_TOK_ARRAY_END),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_ALL),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$[*][*][*]"
        );

        assert_eq!(
            run("$['a']['bb']"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "'a'".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(7, "'bb'".len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$['a']['bb']"
        );

        assert_eq!(
            run("$.a[?(@.e==true)]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, "a".len()))]) },
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_RELATIVE),
                ParserToken::new(P_TOK_IN),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(8, "e".len()))]) },
                ParserToken { key: P_TOK_BOOL, token_type: Some(vec![TokenType::Bool(StrRange::new(11, "true".len()))]) },
                ParserToken::new(P_TOK_FILTER_EQUAL),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$.a[?(@.e==true)]"
        );

        assert_eq!(
            run(r#"$[?(@ > 1)]"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_RELATIVE),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Int(StrRange::new(8, "1".len()))]) },
                ParserToken::new(P_TOK_FILTER_GREATER),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            r#"$[?(@ > 1)]"#
        );

        assert_eq!(
            run("$[:]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken::new(P_TOK_RANGE),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$[:]"
        );

        assert_eq!(
            run(r#"$['single\'quote']"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, r#"'single\'quote'"#.len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            r#"$['single\'quote']"#
        );

        assert_eq!(
            run(r#"$["single\"quote"]"#),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_KEY, token_type: Some(vec![TokenType::String(StrRange::new(2, r#""single\"quote""#.len()))]) },
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            r#"$["single\"quote"]"#
        );
    }

    #[test]
    fn parse_array_float() {
        setup();

        assert_eq!(
            run("$[?(1.1<2.1)]"),
            Ok(vec![
                ParserToken::new(P_TOK_ABSOLUTE),
                ParserToken::new(P_TOK_ARRAY),
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Float(StrRange::new(4, "1.1".len()))]) },
                ParserToken { key: P_TOK_NUMBER, token_type: Some(vec![TokenType::Float(StrRange::new(8, "2.1".len()))]) },
                ParserToken::new(P_TOK_FILTER_LITTLE),
                ParserToken::new(P_TOK_ARRAY_END),
            ]),
            "$[?(1.1<2.1)]"
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
