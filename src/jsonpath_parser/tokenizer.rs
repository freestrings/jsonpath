use std::fmt::{Debug, Formatter};
use std::result::Result;

use super::str_reader::{
    ReaderError,
    StrRange,
    StrReader
};
use super::Token;

use self::std_token_str::*;

pub(crate) mod std_token_str {
    pub const CH_DOLLA: char = '$';
    pub const CH_DOT: char = '.';
    pub const CH_ASTERISK: char = '*';
    pub const CH_OPEN_ARRAY: char = '[';
    pub const CH_CLOSE_ARRAY: char = ']';
    pub const CH_OPEN_PARENTHESIS: char = '(';
    pub const CH_CLOSE_PARENTHESIS: char = ')';
    pub const CH_AT: char = '@';
    pub const CH_QUESTION: char = '?';
    pub const CH_COMMA: char = ',';
    pub const CH_SEMICOLON: char = ':';
    pub const CH_EQUAL: char = '=';
    pub const CH_AMPERSAND: char = '&';
    pub const CH_PIPE: char = '|';
    pub const CH_LITTLE: char = '<';
    pub const CH_GREATER: char = '>';
    pub const CH_EXCLAMATION: char = '!';
    pub const CH_SINGLE_QUOTE: char = '\'';
    pub const CH_DOUBLE_QUOTE: char = '"';

    // tokens
    pub const TOK_ABSOLUTE: &str = "$";
    pub const TOK_DOT: &str = ".";
    pub const TOK_AT: &str = "@";
    pub const TOK_OPEN_ARRAY: &str = "[";
    pub const TOK_CLOSE_ARRAY: &str = "]";
    pub const TOK_ASTERISK: &str = "*";
    pub const TOK_QUESTION: &str = "?";
    pub const TOK_COMMA: &str = ",";
    pub const TOK_SPLIT: &str = ":";
    pub const TOK_OPEN_PARENTHESIS: &str = "(";
    pub const TOK_CLOSE_PARENTHESIS: &str = ")";
    pub const TOK_KEY: &str = "___KEY___";
    pub const TOK_DOUBLE_QUOTED: &str = "\"";
    pub const TOK_SINGLE_QUOTED: &str = "'";
    pub const TOK_EQUAL: &str = "==";
    pub const TOK_GREATER_OR_EQUAL: &str = ">=";
    pub const TOK_GREATER: &str = ">";
    pub const TOK_LITTLE: &str = "<";
    pub const TOK_LITTLE_OR_EQUAL: &str = "<=";
    pub const TOK_NOT_EQUAL: &str = "!=";
    pub const TOK_AND: &str = "&&";
    pub const TOK_OR: &str = "||";
    pub const TOK_WHITESPACE: &str = "___WHITESPACE___";

    // parser tokens
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
}

const DOLLA_TOKEN_RULE: DollaTokenRule = DollaTokenRule {};
const SINGLE_QUOTA_TOKEN_RULE: SingleQuotaTokenRule = SingleQuotaTokenRule {};
const DOUBLE_QUOTA_TOKEN_RULE: DoubleQuotaTokenRule = DoubleQuotaTokenRule {};
const EQUAL_TOKEN_RULE: EqualTokenRule = EqualTokenRule {};
const EXCLAMATION_TOKEN_RULE: ExclamationTokenRule = ExclamationTokenRule {};
const LITTLE_TOKEN_RULE: LittleTokenRule = LittleTokenRule {};
const GREATER_TOKEN_RULE: GreaterTokenRule = GreaterTokenRule {};
const AMPERSAND_TOKEN_RULE: AmpersandTokenRule = AmpersandTokenRule {};
const PIPE_TOKEN_RULE: PipeTokenRule = PipeTokenRule {};
const DOT_TOKEN_RULE: DotTokenRule = DotTokenRule {};
const ASTERISK_TOKEN_RULE: AsteriskTokenRule = AsteriskTokenRule {};
const OPEN_ARRAY_TOKEN_RULE: OpenArrayTokenRule = OpenArrayTokenRule {};
const CLOSE_ARRAY_TOKEN_RULE: CloseArrayTokenRule = CloseArrayTokenRule {};
const OPEN_PARENTHESIS_TOKEN_RULE: OpenParenthesisTokenRule = OpenParenthesisTokenRule {};
const CLOSE_PARENTHESIS_TOKEN_RULE: CloseParenthesisTokenRule = CloseParenthesisTokenRule {};
const AT_TOKEN_RULE: AtTokenRule = AtTokenRule {};
const QUESTION_TOKEN_RULE: QuestionTokenRule = QuestionTokenRule {};
const COMMA_TOKEN_RULE: CommaTokenRule = CommaTokenRule {};
const SEMICOLON_TOKEN_RULE: SemicolonTokenRule = SemicolonTokenRule {};

pub(crate) trait TokenRules {
    fn read_token<'a>(
        &self,
        ch: &char,
        input: &mut StrReader<'_>,
        range: StrRange,
    ) -> Option<Result<Token<'a>, TokenError>>;
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
    ) -> Option<Result<Token<'a>, TokenError>> {
        match ch {
            &CH_DOLLA => Some(DOLLA_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_SINGLE_QUOTE => Some(SINGLE_QUOTA_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_DOUBLE_QUOTE => Some(DOUBLE_QUOTA_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_EQUAL => Some(EQUAL_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_EXCLAMATION => Some(EXCLAMATION_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_LITTLE => Some(LITTLE_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_GREATER => Some(GREATER_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_AMPERSAND => Some(AMPERSAND_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_PIPE => Some(PIPE_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_DOT => Some(DOT_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_ASTERISK => Some(ASTERISK_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_OPEN_ARRAY => Some(OPEN_ARRAY_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_CLOSE_ARRAY => Some(CLOSE_ARRAY_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_OPEN_PARENTHESIS => Some(OPEN_PARENTHESIS_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_CLOSE_PARENTHESIS => Some(CLOSE_PARENTHESIS_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_AT => Some(AT_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_QUESTION => Some(QUESTION_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_COMMA => Some(COMMA_TOKEN_RULE.compute_token(input, range, ch)),
            &CH_SEMICOLON => Some(SEMICOLON_TOKEN_RULE.compute_token(input, range, ch)),
            _ => None
        }
    }
}

pub trait TokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        range: StrRange,
        ch: &char,
    ) -> Result<Token<'a>, TokenError>;
}

// CH_DOLLA
struct DollaTokenRule;

impl TokenRule for DollaTokenRule {
    fn compute_token<'a>(
        &self,
        input: &mut StrReader<'_>,
        _: StrRange,
        _: &char,
    ) -> Result<Token<'a>, TokenError> {
        let read = input.take_while(|c| match c {
            _ if !c.is_alphanumeric() => false,
            _ => !c.is_whitespace(),
        }).map_err(to_token_error)?;

        if read.offset == 0 {
            Ok(Token::new(TOK_ABSOLUTE, read))
        } else {
            Ok(Token::new(TOK_KEY, read))
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
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_SINGLE_QUOTED, self.quote(input, ch)?))
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
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_DOUBLE_QUOTED, self.quote(input, ch)?))
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
    ) -> Result<Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::new(TOK_EQUAL, range))
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
    ) -> Result<Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::new(TOK_NOT_EQUAL, range))
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
    ) -> Result<Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::new(TOK_LITTLE_OR_EQUAL, range))
            }
            _ => Ok(Token::new(TOK_LITTLE, range)),
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
    ) -> Result<Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_EQUAL => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::new(TOK_GREATER_OR_EQUAL, range))
            }
            _ => Ok(Token::new(TOK_GREATER, range)),
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
    ) -> Result<Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_AMPERSAND => {
                let _ = input.next_char().map_err(to_token_error);
                Ok(Token::new(TOK_AND, range))
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
    ) -> Result<Token<'a>, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            CH_PIPE => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::new(TOK_OR, range))
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
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_DOT, range))
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
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_ASTERISK, range))
    }
}

// CH_LARRAY
struct OpenArrayTokenRule;

impl TokenRule for OpenArrayTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_OPEN_ARRAY, range))
    }
}

// CH_RARRAY
struct CloseArrayTokenRule;

impl TokenRule for CloseArrayTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_CLOSE_ARRAY, range))
    }
}

// CH_LPAREN
struct OpenParenthesisTokenRule;

impl TokenRule for OpenParenthesisTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_OPEN_PARENTHESIS, range))
    }
}

// CH_RPAREN
struct CloseParenthesisTokenRule;

impl TokenRule for CloseParenthesisTokenRule {
    fn compute_token<'a>(
        &self,
        _: &mut StrReader<'_>,
        range: StrRange,
        _: &char
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_CLOSE_PARENTHESIS, range))
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
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_AT, range))
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
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_QUESTION, range))
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
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_COMMA, range))
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
    ) -> Result<Token<'a>, TokenError> {
        Ok(Token::new(TOK_SPLIT, range))
    }
}

#[derive(Debug)]
struct StdDefaultRule;

impl TokenRule for StdDefaultRule {
    fn compute_token<'a>(
        &self, input:
        &mut StrReader<'_>,
        _: StrRange,
        _: &char
    ) -> Result<Token<'a>, TokenError> {
        let fun = |c: &char| {
            match c {
                _ if !c.is_alphanumeric() => false,
                _ => !c.is_whitespace(),
            }
        };
        let range = input.take_while(fun).map_err(to_token_error)?;
        Ok(Token::new(TOK_KEY, range))
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
    std_default_rule: StdDefaultRule
}

impl<'a, 'b> Tokenizer<'a> {
    pub fn new(input: &'a str, token_rules: Box<dyn TokenRules>) -> Self {
        trace!("input: {}", input);
        Tokenizer {
            input: StrReader::new(input),
            token_rule: token_rules,
            std_default_rule: StdDefaultRule {}
        }
    }

    fn whitespace(&mut self) -> Result<Token<'b>, TokenError> {
        let range = self
            .input
            .take_while(|c| c.is_whitespace())
            .map_err(to_token_error)?;
        Ok(Token::new(TOK_WHITESPACE, range))
    }

    fn read_token(&mut self, range: StrRange, ch: char) -> Result<Token<'b>, TokenError> {
        if let Some(ret) = self.token_rule.read_token(&ch, &mut self.input, range.clone()) {
            return ret;
        }

        if ch.is_whitespace() {
            return self.whitespace();
        }

        self.std_default_rule.compute_token(&mut self.input, range, &ch)
    }

    pub fn next_token(&mut self) -> Result<Token<'b>, TokenError> {
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
pub struct TokenReader<'a, 'b> {
    tokenizer: Tokenizer<'a>,
    curr_pos: usize,
    peeked: Option<Result<Token<'b>, TokenError>>,
}

impl<'a, 'b> TokenReader<'a, 'b> {
    pub(crate) fn new(input: &'a str, token_rules: Box<dyn TokenRules>) -> Self {
        TokenReader {
            tokenizer: Tokenizer::new(input, token_rules),
            curr_pos: 0,
            peeked: None,
        }
    }

    pub(crate) fn read_value(&self, str_range: &StrRange) -> &'a str {
        self.tokenizer.read_range(str_range)
    }

    pub(crate)  fn peek_token(&mut self) -> Result<&Token<'b>, &TokenError> {
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

    pub(crate)  fn next_token(&mut self) -> Result<Token<'b>, TokenError> {
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

    pub(crate)  fn eat_token(&mut self) {
        let _ = self.next_token();
    }

    pub(crate)  fn eat_whitespace(&mut self) {
        while let Ok(Token { key: TOK_WHITESPACE, .. }) = self.peek_token() {
            self.eat_token();
        }
    }

    pub(crate)  fn to_error(&self) -> TokenError {
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
    use jsonpath_parser::std_token_str::*;
    use jsonpath_parser::str_reader::StrRange;
    use jsonpath_parser::Token;
    use jsonpath_parser::tokenizer::{StdTokenRules, TokenError, Tokenizer, TokenReader};

    fn setup() {
        let _ = env_logger::try_init();
    }

    fn collect_token(input: &str) -> (Vec<Token>, Option<TokenError>) {
        let mut tokenizer = TokenReader {
            tokenizer: Tokenizer::new(input, Box::new(StdTokenRules {})),
            curr_pos: 0,
            peeked: None,
        };
        let mut vec = vec![];
        loop {
            match tokenizer.next_token() {
                Ok(t) => vec.push(t),
                Err(e) => return (vec, Some(e)),
            }
        }
    }

    fn run(input: &str, expected: (Vec<Token>, Option<TokenError>)) {
        let (vec, err) = collect_token(input);
        assert_eq!((vec, err), expected, "\"{}\"", input);
    }

    #[test]
    fn peek() {
        let mut tokenizer = TokenReader {
            tokenizer: Tokenizer::new("$.a", Box::new(StdTokenRules {})),
            curr_pos: 0,
            peeked: None,
        };
        match tokenizer.next_token() {
            Ok(t) => assert_eq!(Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)), t),
            _ => panic!(),
        }

        match tokenizer.peek_token() {
            Ok(t) => assert_eq!(&Token::new(TOK_DOT, StrRange::new(1, 1)), t),
            _ => panic!(),
        }

        match tokenizer.peek_token() {
            Ok(t) => assert_eq!(&Token::new(TOK_DOT, StrRange::new(1, 1)), t),
            _ => panic!(),
        }

        match tokenizer.next_token() {
            Ok(t) => assert_eq!(Token::new(TOK_DOT, StrRange::new(1, 1)), t),
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
                    Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    Token::new(TOK_DOT, StrRange::new(1, 1)),
                    Token::new(TOK_KEY, StrRange::new(2, 2)),
                    Token::new(TOK_DOT, StrRange::new(4, 1)),
                    Token::new(TOK_KEY, StrRange::new(5, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$.   []",
            (
                vec![
                    Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    Token::new(TOK_DOT, StrRange::new(1, 1)),
                    Token::new(TOK_WHITESPACE, StrRange::new(2, 3)),
                    Token::new(TOK_OPEN_ARRAY, StrRange::new(5, 1)),
                    Token::new(TOK_CLOSE_ARRAY, StrRange::new(6, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..",
            (
                vec![Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                     Token::new(TOK_DOT, StrRange::new(1, 1)),
                     Token::new(TOK_DOT, StrRange::new(2, 1))],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..ab",
            (
                vec![
                    Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    Token::new(TOK_DOT, StrRange::new(1, 1)),
                    Token::new(TOK_DOT, StrRange::new(2, 1)),
                    Token::new(TOK_KEY, StrRange::new(3, "ab".len())),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..가 [",
            (
                vec![
                    Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    Token::new(TOK_DOT, StrRange::new(1, 1)),
                    Token::new(TOK_DOT, StrRange::new(2, 1)),
                    Token::new(TOK_KEY, StrRange::new(3, '가'.len_utf8())),
                    Token::new(TOK_WHITESPACE, StrRange::new(6, 1)),
                    Token::new(TOK_OPEN_ARRAY, StrRange::new(7, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "[-1, 2 ]",
            (
                vec![
                    Token::new(TOK_OPEN_ARRAY, StrRange::new(0, 1)),
                    Token::new(TOK_KEY, StrRange::new(1, "-1".len())),
                    Token::new(TOK_COMMA, StrRange::new(3, 1)),
                    Token::new(TOK_WHITESPACE, StrRange::new(4, 1)),
                    Token::new(TOK_KEY, StrRange::new(5, "2".len())),
                    Token::new(TOK_WHITESPACE, StrRange::new(6, 1)),
                    Token::new(TOK_CLOSE_ARRAY, StrRange::new(7, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "[ 1 2 , 3 \"abc\" : -10 ]",
            (
                vec![
                    Token::new(TOK_OPEN_ARRAY, StrRange::new(0, 1)),
                    Token::new(TOK_WHITESPACE, StrRange::new(1, 1)),
                    Token::new(TOK_KEY, StrRange::new(2, "1".len())),
                    Token::new(TOK_WHITESPACE, StrRange::new(3, 1)),
                    Token::new(TOK_KEY, StrRange::new(4, "2".len())),
                    Token::new(TOK_WHITESPACE, StrRange::new(5, 1)),
                    Token::new(TOK_COMMA, StrRange::new(6, 1)),
                    Token::new(TOK_WHITESPACE, StrRange::new(7, 1)),
                    Token::new(TOK_KEY, StrRange::new(8, "3".len())),
                    Token::new(TOK_WHITESPACE, StrRange::new(9, 1)),
                    Token::new(TOK_DOUBLE_QUOTED, StrRange::new(10, "\"abc\"".len())),
                    Token::new(TOK_WHITESPACE, StrRange::new(15, 1)),
                    Token::new(TOK_SPLIT, StrRange::new(16, 1)),
                    Token::new(TOK_WHITESPACE, StrRange::new(17, 1)),
                    Token::new(TOK_KEY, StrRange::new(18, "-10".len())),
                    Token::new(TOK_WHITESPACE, StrRange::new(21, 1)),
                    Token::new(TOK_CLOSE_ARRAY, StrRange::new(22, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?(@.a가 <41.01)",
            (
                vec![
                    Token::new(TOK_QUESTION, StrRange::new(0, 1)),
                    Token::new(TOK_OPEN_PARENTHESIS, StrRange::new(1, 1)),
                    Token::new(TOK_AT, StrRange::new(2, 1)),
                    Token::new(TOK_DOT, StrRange::new(3, 1)),
                    Token::new(TOK_KEY, StrRange::new(4, "a가".chars().map(|c| c.len_utf8()).sum())),
                    Token::new(TOK_WHITESPACE, StrRange::new(8, 1)),
                    Token::new(TOK_LITTLE, StrRange::new(9, 1)),
                    Token::new(TOK_KEY, StrRange::new(10, "41".len())),
                    Token::new(TOK_DOT, StrRange::new(12, 1)),
                    Token::new(TOK_KEY, StrRange::new(13, "01".len())),
                    Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(15, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?(@.a <4a.01)",
            (
                vec![
                    Token::new(TOK_QUESTION, StrRange::new(0, 1)),
                    Token::new(TOK_OPEN_PARENTHESIS, StrRange::new(1, 1)),
                    Token::new(TOK_AT, StrRange::new(2, 1)),
                    Token::new(TOK_DOT, StrRange::new(3, 1)),
                    Token::new(TOK_KEY, StrRange::new(4, "a".len())),
                    Token::new(TOK_WHITESPACE, StrRange::new(5, 1)),
                    Token::new(TOK_LITTLE, StrRange::new(6, 1)),
                    Token::new(TOK_KEY, StrRange::new(7, "4a".len())),
                    Token::new(TOK_DOT, StrRange::new(9, 1)),
                    Token::new(TOK_KEY, StrRange::new(10, "01".len())),
                    Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(12, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?($.c>@.d)",
            (
                vec![
                    Token::new(TOK_QUESTION, StrRange::new(0, 1)),
                    Token::new(TOK_OPEN_PARENTHESIS, StrRange::new(1, 1)),
                    Token::new(TOK_ABSOLUTE, StrRange::new(2, 1)),
                    Token::new(TOK_DOT, StrRange::new(3, 1)),
                    Token::new(TOK_KEY, StrRange::new(4, 1)),
                    Token::new(TOK_GREATER, StrRange::new(5, 1)),
                    Token::new(TOK_AT, StrRange::new(6, 1)),
                    Token::new(TOK_DOT, StrRange::new(7, 1)),
                    Token::new(TOK_KEY, StrRange::new(8, 1)),
                    Token::new(TOK_CLOSE_PARENTHESIS, StrRange::new(9, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$[:]",
            (
                vec![
                    Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    Token::new(TOK_OPEN_ARRAY, StrRange::new(1, 1)),
                    Token::new(TOK_SPLIT, StrRange::new(2, 1)),
                    Token::new(TOK_CLOSE_ARRAY, StrRange::new(3, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$['single\'quote']"#,
            (
                vec![
                    Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    Token::new(TOK_OPEN_ARRAY, StrRange::new(1, 1)),
                    Token::new(TOK_SINGLE_QUOTED, StrRange::new(2, r#"'single\'quote'"#.len())),
                    Token::new(TOK_CLOSE_ARRAY, StrRange::new(17, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$['single\'1','single\'2']"#,
            (
                vec![
                    Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    Token::new(TOK_OPEN_ARRAY, StrRange::new(1, 1)),
                    Token::new(TOK_SINGLE_QUOTED, StrRange::new(2, r#"'single\'1'"#.len())),
                    Token::new(TOK_COMMA, StrRange::new(13, 1)),
                    Token::new(TOK_SINGLE_QUOTED, StrRange::new(14, r#"'single\'2'"#.len())),
                    Token::new(TOK_CLOSE_ARRAY, StrRange::new(25, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$["double\"quote"]"#,
            (
                vec![
                    Token::new(TOK_ABSOLUTE, StrRange::new(0, 1)),
                    Token::new(TOK_OPEN_ARRAY, StrRange::new(1, 1)),
                    Token::new(TOK_DOUBLE_QUOTED, StrRange::new(2, r#""double\"quote""#.len())),
                    Token::new(TOK_CLOSE_ARRAY, StrRange::new(17, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );
    }
}
