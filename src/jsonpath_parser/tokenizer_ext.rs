use std::collections::HashSet;
use jsonpath_parser::path_parser::{ArrayParserNodeBuilder, KeyParserNodeBuilder, ParserNodeBuilder};
use jsonpath_parser::{ParserNode, ParserToken, TokenReader};
use jsonpath_parser::std_token_str::*;

use super::str_reader::StrRange;
use super::str_reader::StrReader;
use super::Token;
use super::TokenError;
use super::tokenizer::{StdTokenRules, TokenRule, TokenRules};

use self::ext_token_str::*;

pub(crate) mod ext_token_str {
    pub const CH_PARENT: char = '^';

    pub const TOK_CARET: &str = "^";

    pub const P_TOK_PARENT: &str = "^";
}

///
/// rule priority:
///     1. `ext rule`
///     2. `std rule`
///
#[derive(Debug, Default)]
pub(super) struct ExtTokenRules {
    std_rules: StdTokenRules,
    enabled_tokens: HashSet<char>,
}

impl TokenRules for ExtTokenRules {
    fn read_token<'a>(
        &self,
        ch: &char,
        input: &mut StrReader<'_>,
        range: StrRange
    ) -> Option<Result<Token<'a>, TokenError>> {
        match ch {
            &CH_PARENT if self.enabled_tokens.contains(&CH_PARENT) => {
                Some(ParentTokenRule {}.compute_token(input, range, ch))
            },
            _ => {
                self.std_rules.read_token(ch, input, range)
            }
        }
    }
}

//
// > public 으로 풀어야 할게 많다. ParserNode는 생성(new)외에는 public 이면 안되는데,,
// > public이 아니면 struct 패턴매칭이 안되네.. 젠장,,

// CH_PARENT
struct ParentTokenRule;

impl TokenRule for ParentTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_CARET, range))
    }
}

struct PathCaretParserNodeBuilder;

impl<'a> ParserNodeBuilder<'a> for PathCaretParserNodeBuilder {
    fn build(&mut self, token_reader: &mut TokenReader, prev: Option<ParserNode<'a>>) -> Result<ParserNode<'a>, TokenError> {
        debug!("#path_caret");
        match token_reader.peek_token() {
            Ok(Token { key: TOK_CARET, .. }) => {
                Ok(ParserNode {
                    token: ParserToken::new(P_TOK_PARENT),
                    left: Some(Box::new(prev.unwrap())),
                    right: None,
                })
            }
            Ok(Token { key: TOK_KEY, .. }) => {
                debug!("#path_parent_key");
                Ok(ParserNode {
                    token: ParserToken::new(P_TOK_PARENT),
                    left: Some(Box::new(prev.unwrap())),
                    right: Some(Box::new(KeyParserNodeBuilder {}.build(token_reader, None)?)),
                })
            },
            Ok(Token { key: TOK_OPEN_ARRAY, .. }) => {
                token_reader.eat_token();
                ArrayParserNodeBuilder{}.build(token_reader, prev)
            }
            _ => Err(token_reader.to_error()),
        }
    }
}