use std::fmt::{Debug, Formatter};
use std::result::Result;

use super::str_reader::{
    ReaderError,
    StrRange,
    StrReader
};
use super::tokens::{
    *,
    constants::*,
};

pub(crate) trait TokenRules {
    fn read_token<'a>(
        &self,
        ch: &char,
        input: &mut StrReader<'_>,
        range: StrRange,
    ) -> Option<Result<_Token<'a>, TokenError>>;
}

impl Debug for dyn TokenRules {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("TokenRules"))
    }
}

#[derive(Debug, Default)]
pub(super) struct StdTokenRules;

impl TokenRules for StdTokenRules {
    fn read_token<'a>(
        &self,
        ch: &char,
        input: &mut StrReader<'_>,
        range: StrRange,
    ) -> Option<Result<_Token<'a>, TokenError>> {
        match ch {
            &CH_DOLLA => Some(DollaTokenRule {}.compute_token(input, range, ch)),
            &CH_SINGLE_QUOTE => Some(SingleQuotaTokenRule {}.compute_token(input, range, ch)),
            &CH_DOUBLE_QUOTE => Some(DoubleQuotaTokenRule {}.compute_token(input, range, ch)),
            &CH_EQUAL => Some(EqualTokenRule {}.compute_token(input, range, ch)),
            &CH_EXCLAMATION => Some(ExclamationTokenRule {}.compute_token(input, range, ch)),
            &CH_LITTLE => Some(LittleTokenRule {}.compute_token(input, range, ch)),
            &CH_GREATER => Some(GreaterTokenRule {}.compute_token(input, range, ch)),
            &CH_AMPERSAND => Some(AmpersandTokenRule {}.compute_token(input, range, ch)),
            &CH_PIPE => Some(PipeTokenRule {}.compute_token(input, range, ch)),
            &CH_DOT => Some(DotTokenRule {}.compute_token(input, range, ch)),
            &CH_ASTERISK => Some(AsteriskTokenRule {}.compute_token(input, range, ch)),
            &CH_LARRAY => Some(LArrayTokenRule {}.compute_token(input, range, ch)),
            &CH_RARRAY => Some(RArrayTokenRule {}.compute_token(input, range, ch)),
            &CH_LPAREN => Some(LParaenTokenRule {}.compute_token(input, range, ch)),
            &CH_RPAREN => Some(RParaenTokenRule {}.compute_token(input, range, ch)),
            &CH_AT => Some(AtTokenRule {}.compute_token(input, range, ch)),
            &CH_QUESTION => Some(QuestionTokenRule {}.compute_token(input, range, ch)),
            &CH_COMMA => Some(CommaTokenRule {}.compute_token(input, range, ch)),
            &CH_SEMICOLON => Some(SemicolonTokenRule {}.compute_token(input, range, ch)),
            _ => None
        }
    }
}

pub(crate) trait TokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        ch: &char,
    ) -> Result<_Token<'a>, TokenError>;
}

// CH_DOLLA
struct DollaTokenRule;

impl TokenRule for DollaTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        _: StrRange,
        _: &char,
    ) -> Result<_Token<'a>, TokenError> {
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
    fn quote(
        &self,
        input: &mut StrReader<'_>,
        ch: &char,
    ) -> Result<StrRange, TokenError> {
        let range = input.take_while(|c| c != ch).map_err(to_token_error)?;
        let val = input.read(&range);
        if let Some('\\') = val.chars().last() {
            input.next_char().map_err(to_token_error)?;
            let remain_range = input.take_while(|c| c != ch).map_err(to_token_error)?;
            input.next_char().map_err(to_token_error)?;
            Ok(StrRange::new(range.pos, remain_range.offset))
        } else {
            input.next_char().map_err(to_token_error)?;
            Ok(range)
        }
    }
}

// CH_SINGLE_QUOTE
struct SingleQuotaTokenRule;

impl QuotaToken for SingleQuotaTokenRule {}

impl TokenRule for SingleQuotaTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        _: StrRange,
        ch: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_SINGLE_QUOTED, self.quote(input, ch)?))
    }
}

// CH_DOUBLE_QUOTE
struct DoubleQuotaTokenRule;

impl QuotaToken for DoubleQuotaTokenRule {}

impl TokenRule for DoubleQuotaTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        _: StrRange,
        ch: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_DOUBLE_QUOTED, self.quote(input, ch)?))
    }
}

// CH_EQUAL
struct EqualTokenRule;

impl TokenRule for EqualTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
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

// CH_EXCLAMATION
struct ExclamationTokenRule;

impl TokenRule for ExclamationTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
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

// CH_LITTLE
struct LittleTokenRule;

impl TokenRule for LittleTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
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

// CH_GREATER
struct GreaterTokenRule;

impl TokenRule for GreaterTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
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

// CH_AMPERSAND
struct AmpersandTokenRule;

impl TokenRule for AmpersandTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
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

// CH_PIPE
struct PipeTokenRule;

impl TokenRule for PipeTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
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

// CH_DOT
struct DotTokenRule;

impl TokenRule for DotTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_DOT, range))
    }
}

// CH_ASTERISK
struct AsteriskTokenRule;

impl TokenRule for AsteriskTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_ASTERISK, range))
    }
}

// CH_LARRAY
struct LArrayTokenRule;

impl TokenRule for LArrayTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_OPEN_ARRAY, range))
    }
}

// CH_RARRAY
struct RArrayTokenRule;

impl TokenRule for RArrayTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_CLOSE_ARRAY, range))
    }
}

// CH_LPAREN
struct LParaenTokenRule;

impl TokenRule for LParaenTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_OPEN_PARENTHESIS, range))
    }
}

// CH_RPAREN
struct RParaenTokenRule;

impl TokenRule for RParaenTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_CLOSE_PARENTHESIS, range))
    }
}

// CH_AT
struct AtTokenRule;

impl TokenRule for AtTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_AT, range))
    }
}

// CH_QUESTION
struct QuestionTokenRule;

impl TokenRule for QuestionTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_QUESTION, range))
    }
}

// CH_COMMA
struct CommaTokenRule;

impl TokenRule for CommaTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_COMMA, range))
    }
}

// CH_SEMICOLON
struct SemicolonTokenRule;

impl TokenRule for SemicolonTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
        Ok(_Token::new(TOK_SPLIT, range))
    }
}

#[derive(Debug)]
struct StdOtherRule;

impl TokenRule for StdOtherRule {
    fn compute_token<'a>(
        &self, input:
        &mut StrReader<'_>,
        _: StrRange,
        _: &char
    ) -> Result<_Token<'a>, TokenError> {
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
        Self::new_with_token_rules(input, Box::new(StdTokenRules::default()))
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
        if let Some(ret) = self.token_rule.read_token(&ch, &mut self.input, range.clone()) {
            return ret;
        } else if ch.is_whitespace() {
            self.whitespace()
        } else {
            self.std_other_rule.compute_token(&mut self.input, range, &ch)
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
pub(crate) struct TokenReader<'a, 'b> {
    tokenizer: Tokenizer<'a>,
    curr_pos: usize,
    // err: Option<TokenError>,
    peeked: Option<Result<_Token<'b>, TokenError>>,
}

impl<'a, 'b> TokenReader<'a, 'b> {
    pub fn new(input: &'a str) -> Self {
        TokenReader {
            tokenizer: Tokenizer::new(input),
            curr_pos: 0,
            // err: None,
            peeked: None,
        }
    }

    pub fn new_with_token_rules(input: &'a str, token_rules: Box<dyn TokenRules>) -> Self {
        TokenReader {
            tokenizer: Tokenizer::new_with_token_rules(input, token_rules),
            curr_pos: 0,
            // err: None,
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
                let token = token.replace_range(
                    StrRange::new(prev_pos, tokenizer.current_pos() - prev_pos)
                );
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
                    let token = token.replace_range(
                        StrRange::new(prev_pos, current_pos - prev_pos)
                    );
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
    use paths::tokens::constants::*;

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