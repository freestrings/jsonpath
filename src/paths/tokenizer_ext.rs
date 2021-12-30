use super::TokenError;
use super::str_reader::StrRange;
use super::str_reader::StrReader;

use super::tokenizer::{StdTokenRules, TokenRule, TokenRules};
use super::tokens::*;

///
/// order of priority:
///     1. `ext rule`
///     2. `std rule`
///
#[derive(Debug, Default)]
pub(super) struct ExtTokenRules {
    std_rules: StdTokenRules,
}

impl ExtTokenRules {
    // pub fn new() -> Self {
    //     let mut rules: HashMap<char, Box<dyn TokenRule>> = HashMap::new();
    //
    //     let rule = ParentRule {};
    //     rules.insert(rule.match_char(), Box::new(rule));
    //
    //     ExtTokenRules { rules, std_rules: StdTokenRules::new() }
    // }
}

impl TokenRules for ExtTokenRules {
    fn read_token<'a>(
        &self,
        ch: &char,
        input: &mut StrReader<'_>,
        range: StrRange
    ) -> Option<Result<_Token<'a>, TokenError>> {
        match ch {
            _ => {
                self.std_rules.read_token(ch, input, range)
            }
        }
    }
}

const CH_PARENT: char = '^';

// CH_PARENT
struct ParentRule;

impl TokenRule for ParentRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        ch: &char
    ) -> Result<_Token<'a>, TokenError> {
        todo!()
    }
}