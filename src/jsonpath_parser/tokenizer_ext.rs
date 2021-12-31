use super::str_reader::StrRange;
use super::str_reader::StrReader;
use super::Token;
use super::TokenError;
use super::tokenizer::{StdTokenRules, TokenRule, TokenRules};

use self::ext_token_str::*;

pub(crate) mod ext_token_str {
    pub const CH_PARENT: char = '^';
}

///
/// rule priority:
///     1. `ext rule`
///     2. `std rule`
///
#[derive(Debug, Default)]
pub(super) struct ExtTokenRules {
    std_rules: StdTokenRules,
}

impl TokenRules for ExtTokenRules {
    fn read_token<'a>(
        &self,
        ch: &char,
        input: &mut StrReader<'_>,
        range: StrRange
    ) -> Option<Result<Token<'a>, TokenError>> {
        match ch {
            &CH_PARENT => Some(ParentRule {}.compute_token(input, range, ch)),
            _ => {
                self.std_rules.read_token(ch, input, range)
            }
        }
    }
}

// CH_PARENT
struct ParentRule;

impl TokenRule for ParentRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        ch: &char
    ) -> Result<Token<'a>, TokenError> {
        todo!()
    }
}