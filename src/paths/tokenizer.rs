use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::result::Result;

use paths::tokens::*;

use super::str_reader::{ReaderError, StrRange, StrReader};

pub(super) trait TokenRules {
    fn get_token(&self, ch: &char) -> Option<&Box<dyn TokenRule>>;
}

impl Debug for dyn TokenRules {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("TokenRules"))
    }
}

#[derive(Debug)]
pub(super) struct StdTokenRules {
    pub rules: HashMap<char, Box<dyn TokenRule>>
}

impl StdTokenRules {
    pub fn new() -> Self {
        let mut rules: HashMap<char, Box<dyn TokenRule>> = HashMap::new();

        let rule = DollaTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = SingleQuotaTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = DoubleQuotaTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = EqualTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = ExclamationTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = LittleTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = GreaterTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = AmpersandTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = PipeTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = DotTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = AsteriskTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = LArrayTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = RArrayTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = LParaenTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = RParaenTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = AtTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = QuestionTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = CommaTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        let rule = SemicolonTokenRule {};
        rules.insert(rule.match_char(), Box::new(rule));

        StdTokenRules { rules }
    }
}

impl TokenRules for StdTokenRules {
    fn get_token(&self, ch: &char) -> Option<&Box<dyn TokenRule>> {
        self.rules.get(ch)
    }
}

pub(super) trait TokenRule {
    fn match_char(&self) -> char;
    fn compute_token<'a>(&self, input: &mut StrReader<'_>, range: StrRange, ch: char) -> Result<_Token<'a>, TokenError>;
}

impl Debug for dyn TokenRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("TokenRule '{}'", self.match_char()))
    }
}

const CH_DOLLA: char = '$';
const CH_DOT: char = '.';
const CH_ASTERISK: char = '*';
const CH_LARRAY: char = '[';
const CH_RARRAY: char = ']';
const CH_LPAREN: char = '(';
const CH_RPAREN: char = ')';
const CH_AT: char = '@';
const CH_QUESTION: char = '?';
const CH_COMMA: char = ',';
const CH_SEMICOLON: char = ':';
const CH_EQUAL: char = '=';
const CH_AMPERSAND: char = '&';
const CH_PIPE: char = '|';
const CH_LITTLE: char = '<';
const CH_GREATER: char = '>';
const CH_EXCLAMATION: char = '!';
const CH_SINGLE_QUOTE: char = '\'';
const CH_DOUBLE_QUOTE: char = '"';

struct DollaTokenRule;

impl TokenRule for DollaTokenRule {
    fn match_char(&self) -> char {
        CH_DOLLA
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, _: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        let read = input.take_while(|c| match c {
            _ if !c.is_alphanumeric() => false,
            _ => !c.is_whitespace(),
        }).map_err(to_token_error)?;

        if read.offset == 0 {
            Ok(_Token::new(TOK_ABSOLUTE, read))
        } else {
            Ok(_Token::new(TOK_KEY, read))
        }
    }
}

trait QuotaToken {
    fn quote(&self, input: &mut StrReader<'_>, ch: char) -> Result<StrRange, TokenError> {
        let range = input.take_while(|c| *c != ch).map_err(to_token_error)?;
        let val = input.read(&range);
        if let Some('\\') = val.chars().last() {
            input.next_char().map_err(to_token_error)?;
            let remain_range = input.take_while(|c| *c != ch).map_err(to_token_error)?;
            input.next_char().map_err(to_token_error)?;
            Ok(StrRange::new(range.pos, remain_range.offset))
        } else {
            input.next_char().map_err(to_token_error)?;
            Ok(range)
        }
    }
}

struct SingleQuotaTokenRule;

impl QuotaToken for SingleQuotaTokenRule {}

impl TokenRule for SingleQuotaTokenRule {
    fn match_char(&self) -> char {
        CH_SINGLE_QUOTE
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, _: StrRange, ch: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_SINGLE_QUOTED, self.quote(input, ch)?))
    }
}

struct DoubleQuotaTokenRule;

impl QuotaToken for DoubleQuotaTokenRule {}

impl TokenRule for DoubleQuotaTokenRule {
    fn match_char(&self) -> char {
        CH_DOUBLE_QUOTE
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, _: StrRange, ch: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_DOUBLE_QUOTED, self.quote(input, ch)?))
    }
}

struct EqualTokenRule;

impl TokenRule for EqualTokenRule {
    fn match_char(&self) -> char {
        CH_EQUAL
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                input.next_char().map_err(to_token_error)?;
                Ok(_Token::new(TOK_EQUAL, range))
            }
            _ => Err(TokenError::Position(range.pos)),
        }
    }
}

struct ExclamationTokenRule;

impl TokenRule for ExclamationTokenRule {
    fn match_char(&self) -> char {
        CH_EXCLAMATION
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                input.next_char().map_err(to_token_error)?;
                Ok(_Token::new(TOK_NOT_EQUAL, range))
            }
            _ => Err(TokenError::Position(range.pos)),
        }
    }
}

struct LittleTokenRule;

impl TokenRule for LittleTokenRule {
    fn match_char(&self) -> char {
        CH_LITTLE
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                input.next_char().map_err(to_token_error)?;
                Ok(_Token::new(TOK_LITTLE_OR_EQUAL, range))
            }
            _ => Ok(_Token::new(TOK_LITTLE, range)),
        }
    }
}

struct GreaterTokenRule;

impl TokenRule for GreaterTokenRule {
    fn match_char(&self) -> char {
        CH_GREATER
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                input.next_char().map_err(to_token_error)?;
                Ok(_Token::new(TOK_GREATER_OR_EQUAL, range))
            }
            _ => Ok(_Token::new(TOK_GREATER, range)),
        }
    }
}

struct AmpersandTokenRule;

impl TokenRule for AmpersandTokenRule {
    fn match_char(&self) -> char {
        CH_AMPERSAND
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_AMPERSAND => {
                let _ = input.next_char().map_err(to_token_error);
                Ok(_Token::new(TOK_AND, range))
            }
            _ => Err(TokenError::Position(range.pos)),
        }
    }
}

struct PipeTokenRule;

impl TokenRule for PipeTokenRule {
    fn match_char(&self) -> char {
        CH_PIPE
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_PIPE => {
                input.next_char().map_err(to_token_error)?;
                Ok(_Token::new(TOK_OR, range))
            }
            _ => Err(TokenError::Position(range.pos)),
        }
    }
}

struct DotTokenRule;

impl TokenRule for DotTokenRule {
    fn match_char(&self) -> char {
        CH_DOT
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_DOT, range))
    }
}

struct AsteriskTokenRule;

impl TokenRule for AsteriskTokenRule {
    fn match_char(&self) -> char {
        CH_ASTERISK
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_ASTERISK, range))
    }
}

struct LArrayTokenRule;

impl TokenRule for LArrayTokenRule {
    fn match_char(&self) -> char {
        CH_LARRAY
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_OPEN_ARRAY, range))
    }
}

struct RArrayTokenRule;

impl TokenRule for RArrayTokenRule {
    fn match_char(&self) -> char {
        CH_RARRAY
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_CLOSE_ARRAY, range))
    }
}

struct LParaenTokenRule;

impl TokenRule for LParaenTokenRule {
    fn match_char(&self) -> char {
        CH_LPAREN
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_OPEN_PARENTHESIS, range))
    }
}

struct RParaenTokenRule;

impl TokenRule for RParaenTokenRule {
    fn match_char(&self) -> char {
        CH_RPAREN
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_CLOSE_PARENTHESIS, range))
    }
}

struct AtTokenRule;

impl TokenRule for AtTokenRule {
    fn match_char(&self) -> char {
        CH_AT
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_AT, range))
    }
}

struct QuestionTokenRule;

impl TokenRule for QuestionTokenRule {
    fn match_char(&self) -> char {
        CH_QUESTION
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_QUESTION, range))
    }
}

struct CommaTokenRule;

impl TokenRule for CommaTokenRule {
    fn match_char(&self) -> char {
        CH_COMMA
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_COMMA, range))
    }
}

struct SemicolonTokenRule;

impl TokenRule for SemicolonTokenRule {
    fn match_char(&self) -> char {
        CH_SEMICOLON
    }

    fn compute_token<'a>(&self, _: &mut StrReader<'_>, range: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_SPLIT, range))
    }
}

#[derive(Debug)]
struct StdOtherRule;

impl TokenRule for StdOtherRule {
    fn match_char(&self) -> char {
        '\u{0FFF}' // ...((((;'')
    }

    fn compute_token<'a>(&self, input: &mut StrReader<'_>, _: StrRange, _: char) -> Result<_Token<'a>, TokenError> {
        let fun = |c: &char| {
            match c {
                _ if !c.is_alphanumeric() => false,
                _ => !c.is_whitespace(),
            }
        };
        let range = input.take_while(fun).map_err(to_token_error)?;
        Ok(_Token::new(TOK_KEY, range))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenError {
    Eof,
    Position(usize),
    Unknown
}

fn to_token_error(read_err: ReaderError) -> TokenError {
    match read_err {
        ReaderError::Eof => TokenError::Eof,
    }
}

#[derive(Debug)]
pub(super) struct Tokenizer<'a> {
    input: StrReader<'a>,
    token_rule: Box<dyn TokenRules>,
    std_other_rule: StdOtherRule
}

impl<'a, 'b> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self::new_with_token_rules(input, Box::new(StdTokenRules::new()))
    }

    pub fn new_with_token_rules(input: &'a str, token_rules: Box<dyn TokenRules>) -> Self {
        trace!("input: {}", input);
        Tokenizer {
            input: StrReader::new(input),
            token_rule: token_rules,
            std_other_rule: StdOtherRule {}
        }
    }

    fn whitespace(&mut self) -> Result<_Token<'b>, TokenError> {
        let range = self
            .input
            .take_while(|c| c.is_whitespace())
            .map_err(to_token_error)?;
        Ok(_Token::new(TOK_WHITESPACE, range))
    }

    fn read_token(&mut self, range: StrRange, ch: char) -> Result<_Token<'b>, TokenError> {
        if let Some(rule) = self.token_rule.get_token(&ch) {
            rule.compute_token(&mut self.input, range, ch)
        } else if ch.is_whitespace() {
            self.whitespace()
        } else {
            self.std_other_rule.compute_token(&mut self.input, range, ch)
        }
    }

    pub fn next_token(&mut self) -> Result<_Token<'b>, TokenError> {
        let (range, ch) = self.input.next_char().map_err(to_token_error)?;
        match self.read_token(range, ch) {
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }

    fn current_pos(&self) -> usize {
        self.input.current_pos()
    }

    fn read_range(&self, range: &StrRange) -> &'a str {
        self.input.read(range)
    }
}

#[derive(Debug)]
pub(super) struct TokenReader<'a, 'b> {
    tokenizer: Tokenizer<'a>,
    curr_pos: usize,
    err: Option<TokenError>,
    peeked: Option<Result<_Token<'b>, TokenError>>,
}

impl<'a, 'b> TokenReader<'a, 'b> {
    pub fn new(input: &'a str) -> Self {
        TokenReader {
            tokenizer: Tokenizer::new(input),
            curr_pos: 0,
            err: None,
            peeked: None,
        }
    }

    pub fn new_with_token_rules(input: &'a str, token_rules: Box<dyn TokenRules>) -> Self {
        TokenReader {
            tokenizer: Tokenizer::new_with_token_rules(input, token_rules),
            curr_pos: 0,
            err: None,
            peeked: None,
        }
    }

    pub fn read_value(&self, str_range: &StrRange) -> &'a str {
        self.tokenizer.read_range(str_range)
    }

    pub fn peek_token(&mut self) -> Result<&_Token<'b>, &TokenError> {
        let tokenizer = &mut self.tokenizer;
        let prev_pos = self.curr_pos;
        let peeked = self.peeked.get_or_insert_with(|| {
            let mut token = tokenizer.next_token();
            if let Ok(token) = &mut token {
                let token = token.replace_range(StrRange::new(prev_pos, tokenizer.current_pos() - prev_pos));
                return Ok(token);
            }
            token
        });
        self.curr_pos = tokenizer.current_pos();
        peeked.as_ref()
    }

    pub fn next_token(&mut self) -> Result<_Token<'b>, TokenError> {
        match self.peeked.take() {
            Some(v) => v,
            None => {
                let prev_pos = self.curr_pos;
                let tokenizer = &mut self.tokenizer;
                let mut token = tokenizer.next_token();
                if let Ok(token) = &mut token {
                    let current_pos = tokenizer.current_pos();
                    let token = token.replace_range(StrRange::new(prev_pos, current_pos - prev_pos));
                    self.curr_pos = current_pos;
                    return Ok(token);
                }
                token
            }
        }
    }

    pub fn eat_token(&mut self) {
        let _ = self.next_token();
    }

    pub fn eat_whitespace(&mut self) {
        while let Ok(_Token { key: TOK_WHITESPACE, .. }) = self.peek_token() {
            self.eat_token();
        }
    }

    pub fn to_error(&self) -> TokenError {
        let path = self.tokenizer.input.origin_str();
        let curr_pos = self.curr_pos;
        if path.len() == curr_pos {
            TokenError::Eof
        } else {
            TokenError::Position(curr_pos)
        }
    }
}

#[cfg(test)]
mod tokenizer_tests {
    use paths::str_reader::StrRange;
    use paths::tokenizer::{TokenError, TokenReader};
    use paths::tokens::*;

    fn setup() {
        let _ = env_logger::try_init();
    }

    fn collect_token(input: &str) -> (Vec<_Token>, Option<TokenError>) {
        let mut tokenizer = TokenReader::new(input);
        let mut vec = vec![];
        loop {
            match tokenizer.next_token() {
                Ok(t) => vec.push(t),
                Err(e) => return (vec, Some(e)),
            }
        }
    }

    fn run(input: &str, expected: (Vec<_Token>, Option<TokenError>)) {
        let (vec, err) = collect_token(input);
        assert_eq!((vec, err), expected, "\"{}\"", input);
    }

    #[test]
    fn peek() {
        let mut tokenizer = TokenReader::new("$.a");
        match tokenizer.next_token() {
            Ok(t) => assert_eq!(_Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)), t),
            _ => panic!(),
        }

        match tokenizer.peek_token() {
            Ok(t) => assert_eq!(&_Token::new(TOK_DOT, StrRange::new(1, 1)), t),
            _ => panic!(),
        }

        match tokenizer.peek_token() {
            Ok(t) => assert_eq!(&_Token::new(TOK_DOT, StrRange::new(1, 1)), t),
            _ => panic!(),
        }

        match tokenizer.next_token() {
            Ok(t) => assert_eq!(_Token::new(TOK_DOT, StrRange::new(1, 1)), t),
            _ => panic!(),
        }
    }

    #[test]
    fn token() {
        setup();

        run(
            "$.01.a",
            (
                vec![
                    _Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    _Token::new(TOK_DOT, StrRange::new(1, 1)),
                    _Token::new(TOK_KEY, StrRange::new(2, 2)),
                    _Token::new(TOK_DOT, StrRange::new(4, 1)),
                    _Token::new(TOK_KEY, StrRange::new(5, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$.   []",
            (
                vec![
                    _Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    _Token::new(TOK_DOT, StrRange::new(1, 1)),
                    _Token::new(TOK_WHITESPACE, StrRange::new(2, 3)),
                    _Token::new(TOK_OPEN_ARRAY, StrRange::new(5, 1)),
                    _Token::new(TOK_CLOSE_ARRAY, StrRange::new(6, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..",
            (
                vec![_Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                     _Token::new(TOK_DOT, StrRange::new(1, 1)),
                     _Token::new(TOK_DOT, StrRange::new(2, 1))],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..ab",
            (
                vec![
                    _Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    _Token::new(TOK_DOT, StrRange::new(1, 1)),
                    _Token::new(TOK_DOT, StrRange::new(2, 1)),
                    _Token::new(TOK_KEY, StrRange::new(3, "ab".len())),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..가 [",
            (
                vec![
                    _Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    _Token::new(TOK_DOT, StrRange::new(1, 1)),
                    _Token::new(TOK_DOT, StrRange::new(2, 1)),
                    _Token::new(TOK_KEY, StrRange::new(3, '가'.len_utf8())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(6, 1)),
                    _Token::new(TOK_OPEN_ARRAY, StrRange::new(7, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "[-1, 2 ]",
            (
                vec![
                    _Token::new(TOK_OPEN_ARRAY, StrRange::new(0, 1)),
                    _Token::new(TOK_KEY, StrRange::new(1, "-1".len())),
                    _Token::new(TOK_COMMA, StrRange::new(3, 1)),
                    _Token::new(TOK_WHITESPACE, StrRange::new(4, 1)),
                    _Token::new(TOK_KEY, StrRange::new(5, "2".len())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(6, 1)),
                    _Token::new(TOK_CLOSE_ARRAY, StrRange::new(7, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "[ 1 2 , 3 \"abc\" : -10 ]",
            (
                vec![
                    _Token::new(TOK_OPEN_ARRAY, StrRange::new(0, 1)),
                    _Token::new(TOK_WHITESPACE, StrRange::new(1, 1)),
                    _Token::new(TOK_KEY, StrRange::new(2, "1".len())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(3, 1)),
                    _Token::new(TOK_KEY, StrRange::new(4, "2".len())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(5, 1)),
                    _Token::new(TOK_COMMA, StrRange::new(6, 1)),
                    _Token::new(TOK_WHITESPACE, StrRange::new(7, 1)),
                    _Token::new(TOK_KEY, StrRange::new(8, "3".len())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(9, 1)),
                    _Token::new(TOK_DOUBLE_QUOTED, StrRange::new(10, "\"abc\"".len())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(15, 1)),
                    _Token::new(TOK_SPLIT, StrRange::new(16, 1)),
                    _Token::new(TOK_WHITESPACE, StrRange::new(17, 1)),
                    _Token::new(TOK_KEY, StrRange::new(18, "-10".len())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(21, 1)),
                    _Token::new(TOK_CLOSE_ARRAY, StrRange::new(22, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?(@.a가 <41.01)",
            (
                vec![
                    _Token::new(TOK_QUESTION, StrRange::new(0, 1)),
                    _Token::new(TOK_OPEN_PARENTHESIS, StrRange::new(1, 1)),
                    _Token::new(TOK_AT, StrRange::new(2, 1)),
                    _Token::new(TOK_DOT, StrRange::new(3, 1)),
                    _Token::new(TOK_KEY, StrRange::new(4, "a가".chars().map(|c| c.len_utf8()).sum())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(8, 1)),
                    _Token::new(TOK_LITTLE, StrRange::new(9, 1)),
                    _Token::new(TOK_KEY, StrRange::new(10, "41".len())),
                    _Token::new(TOK_DOT, StrRange::new(12, 1)),
                    _Token::new(TOK_KEY, StrRange::new(13, "01".len())),
                    _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(15, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?(@.a <4a.01)",
            (
                vec![
                    _Token::new(TOK_QUESTION, StrRange::new(0, 1)),
                    _Token::new(TOK_OPEN_PARENTHESIS, StrRange::new(1, 1)),
                    _Token::new(TOK_AT, StrRange::new(2, 1)),
                    _Token::new(TOK_DOT, StrRange::new(3, 1)),
                    _Token::new(TOK_KEY, StrRange::new(4, "a".len())),
                    _Token::new(TOK_WHITESPACE, StrRange::new(5, 1)),
                    _Token::new(TOK_LITTLE, StrRange::new(6, 1)),
                    _Token::new(TOK_KEY, StrRange::new(7, "4a".len())),
                    _Token::new(TOK_DOT, StrRange::new(9, 1)),
                    _Token::new(TOK_KEY, StrRange::new(10, "01".len())),
                    _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(12, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?($.c>@.d)",
            (
                vec![
                    _Token::new(TOK_QUESTION, StrRange::new(0, 1)),
                    _Token::new(TOK_OPEN_PARENTHESIS, StrRange::new(1, 1)),
                    _Token::new(TOK_ABSOLUTE, StrRange::new(2, 1)),
                    _Token::new(TOK_DOT, StrRange::new(3, 1)),
                    _Token::new(TOK_KEY, StrRange::new(4, 1)),
                    _Token::new(TOK_GREATER, StrRange::new(5, 1)),
                    _Token::new(TOK_AT, StrRange::new(6, 1)),
                    _Token::new(TOK_DOT, StrRange::new(7, 1)),
                    _Token::new(TOK_KEY, StrRange::new(8, 1)),
                    _Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(9, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$[:]",
            (
                vec![
                    _Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    _Token::new(TOK_OPEN_ARRAY, StrRange::new(1, 1)),
                    _Token::new(TOK_SPLIT, StrRange::new(2, 1)),
                    _Token::new(TOK_CLOSE_ARRAY, StrRange::new(3, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$['single\'quote']"#,
            (
                vec![
                    _Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    _Token::new(TOK_OPEN_ARRAY, StrRange::new(1, 1)),
                    _Token::new(TOK_SINGLE_QUOTED, StrRange::new(2, r#"'single\'quote'"#.len())),
                    _Token::new(TOK_CLOSE_ARRAY, StrRange::new(17, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$['single\'1','single\'2']"#,
            (
                vec![
                    _Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    _Token::new(TOK_OPEN_ARRAY, StrRange::new(1, 1)),
                    _Token::new(TOK_SINGLE_QUOTED, StrRange::new(2, r#"'single\'1'"#.len())),
                    _Token::new(TOK_COMMA, StrRange::new(13, 1)),
                    _Token::new(TOK_SINGLE_QUOTED, StrRange::new(14, r#"'single\'2'"#.len())),
                    _Token::new(TOK_CLOSE_ARRAY, StrRange::new(25, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$["double\"quote"]"#,
            (
                vec![
                    _Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    _Token::new(TOK_OPEN_ARRAY, StrRange::new(1, 1)),
                    _Token::new(TOK_DOUBLE_QUOTED, StrRange::new(2, r#""double\"quote""#.len())),
                    _Token::new(TOK_CLOSE_ARRAY, StrRange::new(17, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );
    }
}