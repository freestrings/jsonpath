use std::collections::HashMap;
use paths::str_reader::StrReader;
use paths::{StrRange, TokenError};

use paths::tokenizer::{StdTokenRules, TokenRule, TokenRules};
use paths::tokens::*;

///
/// order of priority:
///     1. `ext rule`
///     2. `std rule`
///
#[derive(Debug)]
pub(super) struct ExtTokenRules {
    rules: HashMap<char, Box<dyn TokenRule>>,
    std_rules: StdTokenRules,
}

impl ExtTokenRules {
    pub fn new() -> Self {
        let mut rules: HashMap<char, Box<dyn TokenRule>> = HashMap::new();

        let rule = ParentRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        ExtTokenRules { rules, std_rules: StdTokenRules::new() }
    }
}

impl TokenRules for ExtTokenRules {
    fn get_token(&self, ch: &char) -> Option<&Box<dyn TokenRule>> {
        if self.rules.contains_key(ch) {
            self.rules.get(ch)
        } else if self.std_rules.rules.contains_key(ch) {
            self.std_rules.get_token(ch)
        } else {
            None
        }
    }
}

const CH_PARENT: char = '^';

struct ParentRule;

impl TokenRule for ParentRule {
    fn match_char(&self) -> char {
        CH_PARENT
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, range: StrRange, ch: char) -> Result<_Token<'a>, TokenError> {
        todo!()
    }
}