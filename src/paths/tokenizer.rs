use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::result::Result;

use super::str_reader::{ReaderError, StrRange, StrReader};
use super::tokens::Token;

trait TokenRule {
    fn token_char(&self) -> char;
    fn token(&self, input: &mut StrReader<'_>, span: StrRange, ch: char) -> Result<Token, TokenError>;
}

impl Debug for dyn TokenRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("TokenRule '{}'", self.token_char()))
    }
}

struct DollaToken;

impl TokenRule for DollaToken {
    fn token_char(&self) -> char {
        '$'
    }

    fn token(&self, input: &mut StrReader<'_>, _: StrRange, _: char) -> Result<Token, TokenError> {
        let read = input.take_while(|c| match c {
            _ if !c.is_alphanumeric() => false,
            _ => !c.is_whitespace(),
        }).map_err(to_token_error)?;

        if read.offset == 0 {
            Ok(Token::Absolute(read))
        } else {
            Ok(Token::Key(read))
        }
    }
}

trait QuotaToken {
    fn quote(&self, input: &mut StrReader<'_>, ch: char) -> Result<StrRange, TokenError> {
        let span = input.take_while(|c| *c != ch).map_err(to_token_error)?;
        let val = input.read(&span);
        if let Some('\\') = val.chars().last() {
            input.next_char().map_err(to_token_error)?;
            let remain_span = input.take_while(|c| *c != ch).map_err(to_token_error)?;
            input.next_char().map_err(to_token_error)?;
            Ok(StrRange::new(span.pos, remain_span.offset))
        } else {
            input.next_char().map_err(to_token_error)?;
            Ok(span)
        }
    }
}

struct SingleQuotaToken;

impl QuotaToken for SingleQuotaToken {}

impl TokenRule for SingleQuotaToken {
    fn token_char(&self) -> char {
        '\''
    }

    fn token(&self, input: &mut StrReader<'_>, _: StrRange, ch: char) -> Result<Token, TokenError> {
        Ok(Token::SingleQuoted(self.quote(input, ch)?))
    }
}

struct DoubleQuotaToken;

impl QuotaToken for DoubleQuotaToken {}

impl TokenRule for DoubleQuotaToken {
    fn token_char(&self) -> char {
        '"'
    }

    fn token(&self, input: &mut StrReader<'_>, _: StrRange, ch: char) -> Result<Token, TokenError> {
        Ok(Token::DoubleQuoted(self.quote(input, ch)?))
    }
}

struct EqualToken;

impl TokenRule for EqualToken {
    fn token_char(&self) -> char {
        '='
    }

    fn token(&self, input: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            '=' => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::Equal(span))
            }
            _ => Err(TokenError::Position(span.pos)),
        }
    }
}

struct ExclamationToken;

impl TokenRule for ExclamationToken {
    fn token_char(&self) -> char {
        '!'
    }

    fn token(&self, input: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            '=' => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::NotEqual(span))
            }
            _ => Err(TokenError::Position(span.pos)),
        }
    }
}

struct LittleToken;

impl TokenRule for LittleToken {
    fn token_char(&self) -> char {
        '<'
    }

    fn token(&self, input: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            '=' => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::LittleOrEqual(span))
            }
            _ => Ok(Token::Little(span)),
        }
    }
}

struct GreaterToken;

impl TokenRule for GreaterToken {
    fn token_char(&self) -> char {
        '>'
    }

    fn token(&self, input: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            '=' => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::GreaterOrEqual(span))
            }
            _ => Ok(Token::Greater(span)),
        }
    }
}

struct AmpersandToken;

impl TokenRule for AmpersandToken {
    fn token_char(&self) -> char {
        '&'
    }

    fn token(&self, input: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            '&' => {
                let _ = input.next_char().map_err(to_token_error);
                Ok(Token::And(span))
            }
            _ => Err(TokenError::Position(span.pos)),
        }
    }
}

struct PipeToken;

impl TokenRule for PipeToken {
    fn token_char(&self) -> char {
        '|'
    }

    fn token(&self, input: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        let ch = input.peek_char().map_err(to_token_error)?;
        match ch {
            '|' => {
                input.next_char().map_err(to_token_error)?;
                Ok(Token::Or(span))
            }
            _ => Err(TokenError::Position(span.pos)),
        }
    }
}

struct DotToken;

impl TokenRule for DotToken {
    fn token_char(&self) -> char {
        '.'
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::Dot(span))
    }
}

struct AsteriskToken;

impl TokenRule for AsteriskToken {
    fn token_char(&self) -> char {
        '*'
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::Asterisk(span))
    }
}

struct LArrayToken;

impl TokenRule for LArrayToken {
    fn token_char(&self) -> char {
        '['
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::OpenArray(span))
    }
}

struct RArrayToken;

impl TokenRule for RArrayToken {
    fn token_char(&self) -> char {
        ']'
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::CloseArray(span))
    }
}

struct LParaenToken;

impl TokenRule for LParaenToken {
    fn token_char(&self) -> char {
        '('
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::OpenParenthesis(span))
    }
}

struct RParaenToken;

impl TokenRule for RParaenToken {
    fn token_char(&self) -> char {
        ')'
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::CloseParenthesis(span))
    }
}

struct AtToken;

impl TokenRule for AtToken {
    fn token_char(&self) -> char {
        '@'
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::At(span))
    }
}

struct QuestionToken;

impl TokenRule for QuestionToken {
    fn token_char(&self) -> char {
        '?'
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::Question(span))
    }
}

struct CommaToken;

impl TokenRule for CommaToken {
    fn token_char(&self) -> char {
        ','
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::Comma(span))
    }
}

struct SemicolonToken;

impl TokenRule for SemicolonToken {
    fn token_char(&self) -> char {
        ':'
    }

    fn token(&self, _: &mut StrReader<'_>, span: StrRange, _: char) -> Result<Token, TokenError> {
        Ok(Token::Split(span))
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
    token_rules: HashMap<char, Box<dyn TokenRule>>
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        trace!("input: {}", input);
        let mut instance = Tokenizer {
            input: StrReader::new(input),
            token_rules: HashMap::new()
        };

        let token_rule = DollaToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = SingleQuotaToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = DoubleQuotaToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = EqualToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = ExclamationToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = LittleToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = GreaterToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = AmpersandToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = PipeToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = DotToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = AsteriskToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = LArrayToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = RArrayToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = LParaenToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = RParaenToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = AtToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = QuestionToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = CommaToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        let token_rule = SemicolonToken {};
        instance.token_rules.insert(token_rule.token_char(), Box::new(token_rule));

        instance
    }

    fn add_token_rule<T>(&mut self, token_rule: T) where T: TokenRule + 'static {
        self.token_rules.insert(token_rule.token_char(), Box::new(token_rule));
    }

    fn whitespace(&mut self) -> Result<Token, TokenError> {
        let span = self
            .input
            .take_while(|c| c.is_whitespace())
            .map_err(to_token_error)?;
        Ok(Token::Whitespace(span))
    }

    fn other(&mut self) -> Result<Token, TokenError> {
        let fun = |c: &char| {
            match c {
                _ if !c.is_alphanumeric() => false,
                _ => !c.is_whitespace(),
            }
        };
        let span = self.input.take_while(fun).map_err(to_token_error)?;
        Ok(Token::Key(span))
    }

    fn read_token(&mut self, span: StrRange, ch: char) -> Result<Token, TokenError> {
        if let Some(rule) = self.token_rules.get(&ch) {
            rule.token(&mut self.input, span, ch)
        } else if ch.is_whitespace() {
            self.whitespace()
        } else {
            self.other()
        }
    }

    pub fn next_token(&mut self) -> Result<Token, TokenError> {
        let (span, ch) = self.input.next_char().map_err(to_token_error)?;
        match self.read_token(span, ch) {
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }

    fn current_pos(&self) -> usize {
        self.input.current_pos()
    }

    fn read_span(&self, span: &StrRange) -> &'a str {
        self.input.read(span)
    }
}

#[derive(Debug)]
pub(super) struct TokenReader<'a> {
    tokenizer: Tokenizer<'a>,
    curr_pos: usize,
    err: Option<TokenError>,
    peeked: Option<Result<Token, TokenError>>,
}

impl<'a> TokenReader<'a> {
    pub fn new(input: &'a str) -> Self {
        TokenReader {
            tokenizer: Tokenizer::new(input),
            curr_pos: 0,
            err: None,
            peeked: None,
        }
    }

    pub fn read_value(&self, str_range: &StrRange) -> &'a str {
        self.tokenizer.read_span(str_range)
    }

    pub fn peek_token(&mut self) -> Result<&Token, &TokenError> {
        let tokenizer = &mut self.tokenizer;
        let prev_pos = self.curr_pos;
        let peeked = self.peeked.get_or_insert_with(|| {
            let mut token = tokenizer.next_token();
            if let Ok(token) = &mut token {
                let token = token.reset_span(StrRange::new(prev_pos, tokenizer.current_pos() - prev_pos));
                return Ok(token);
            }
            token
        });
        self.curr_pos = tokenizer.current_pos();
        peeked.as_ref()
    }

    pub fn next_token(&mut self) -> Result<Token, TokenError> {
        match self.peeked.take() {
            Some(v) => v,
            None => {
                let prev_pos = self.curr_pos;
                let tokenizer = &mut self.tokenizer;
                let mut token = tokenizer.next_token();
                if let Ok(token) = &mut token {
                    let current_pos = tokenizer.current_pos();
                    let token = token.reset_span(StrRange::new(prev_pos, current_pos - prev_pos));
                    self.curr_pos = current_pos;
                    return Ok(token);
                }
                token
            }
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
    use paths::tokens::Token;

    fn setup() {
        let _ = env_logger::try_init();
    }

    fn collect_token(input: &str) -> (Vec<Token>, Option<TokenError>) {
        let mut tokenizer = TokenReader::new(input);
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
        let mut tokenizer = TokenReader::new("$.a");
        match tokenizer.next_token() {
            Ok(t) => assert_eq!(Token::Absolute(StrRange::new(0, 1)), t),
            _ => panic!(),
        }

        match tokenizer.peek_token() {
            Ok(t) => assert_eq!(&Token::Dot(StrRange::new(1, 1)), t),
            _ => panic!(),
        }

        match tokenizer.peek_token() {
            Ok(t) => assert_eq!(&Token::Dot(StrRange::new(1, 1)), t),
            _ => panic!(),
        }

        match tokenizer.next_token() {
            Ok(t) => assert_eq!(Token::Dot(StrRange::new(1, 1)), t),
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
                    Token::Absolute(StrRange::new(0, 1)),
                    Token::Dot(StrRange::new(1, 1)),
                    Token::Key(StrRange::new(2, 2)),
                    Token::Dot(StrRange::new(4, 1)),
                    Token::Key(StrRange::new(5, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$.   []",
            (
                vec![
                    Token::Absolute(StrRange::new(0, 1)),
                    Token::Dot(StrRange::new(1, 1)),
                    Token::Whitespace(StrRange::new(2, 3)),
                    Token::OpenArray(StrRange::new(5, 1)),
                    Token::CloseArray(StrRange::new(6, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..",
            (
                vec![Token::Absolute(StrRange::new(0, 1)), Token::Dot(StrRange::new(1, 1)), Token::Dot(StrRange::new(2, 1))],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..ab",
            (
                vec![
                    Token::Absolute(StrRange::new(0, 1)),
                    Token::Dot(StrRange::new(1, 1)),
                    Token::Dot(StrRange::new(2, 1)),
                    Token::Key(StrRange::new(3, "ab".len())),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..가 [",
            (
                vec![
                    Token::Absolute(StrRange::new(0, 1)),
                    Token::Dot(StrRange::new(1, 1)),
                    Token::Dot(StrRange::new(2, 1)),
                    Token::Key(StrRange::new(3, '가'.len_utf8())),
                    Token::Whitespace(StrRange::new(6, 1)),
                    Token::OpenArray(StrRange::new(7, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "[-1, 2 ]",
            (
                vec![
                    Token::OpenArray(StrRange::new(0, 1)),
                    Token::Key(StrRange::new(1, "-1".len())),
                    Token::Comma(StrRange::new(3, 1)),
                    Token::Whitespace(StrRange::new(4, 1)),
                    Token::Key(StrRange::new(5, "2".len())),
                    Token::Whitespace(StrRange::new(6, 1)),
                    Token::CloseArray(StrRange::new(7, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "[ 1 2 , 3 \"abc\" : -10 ]",
            (
                vec![
                    Token::OpenArray(StrRange::new(0, 1)),
                    Token::Whitespace(StrRange::new(1, 1)),
                    Token::Key(StrRange::new(2, "1".len())),
                    Token::Whitespace(StrRange::new(3, 1)),
                    Token::Key(StrRange::new(4, "2".len())),
                    Token::Whitespace(StrRange::new(5, 1)),
                    Token::Comma(StrRange::new(6, 1)),
                    Token::Whitespace(StrRange::new(7, 1)),
                    Token::Key(StrRange::new(8, "3".len())),
                    Token::Whitespace(StrRange::new(9, 1)),
                    Token::DoubleQuoted(StrRange::new(10, "\"abc\"".len())),
                    Token::Whitespace(StrRange::new(15, 1)),
                    Token::Split(StrRange::new(16, 1)),
                    Token::Whitespace(StrRange::new(17, 1)),
                    Token::Key(StrRange::new(18, "-10".len())),
                    Token::Whitespace(StrRange::new(21, 1)),
                    Token::CloseArray(StrRange::new(22, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?(@.a가 <41.01)",
            (
                vec![
                    Token::Question(StrRange::new(0, 1)),
                    Token::OpenParenthesis(StrRange::new(1, 1)),
                    Token::At(StrRange::new(2, 1)),
                    Token::Dot(StrRange::new(3, 1)),
                    Token::Key(StrRange::new(4, "a가".chars().map(|c| c.len_utf8()).sum())),
                    Token::Whitespace(StrRange::new(8, 1)),
                    Token::Little(StrRange::new(9, 1)),
                    Token::Key(StrRange::new(10, "41".len())),
                    Token::Dot(StrRange::new(12, 1)),
                    Token::Key(StrRange::new(13, "01".len())),
                    Token::CloseParenthesis(StrRange::new(15, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?(@.a <4a.01)",
            (
                vec![
                    Token::Question(StrRange::new(0, 1)),
                    Token::OpenParenthesis(StrRange::new(1, 1)),
                    Token::At(StrRange::new(2, 1)),
                    Token::Dot(StrRange::new(3, 1)),
                    Token::Key(StrRange::new(4, "a".len())),
                    Token::Whitespace(StrRange::new(5, 1)),
                    Token::Little(StrRange::new(6, 1)),
                    Token::Key(StrRange::new(7, "4a".len())),
                    Token::Dot(StrRange::new(9, 1)),
                    Token::Key(StrRange::new(10, "01".len())),
                    Token::CloseParenthesis(StrRange::new(12, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?($.c>@.d)",
            (
                vec![
                    Token::Question(StrRange::new(0, 1)),
                    Token::OpenParenthesis(StrRange::new(1, 1)),
                    Token::Absolute(StrRange::new(2, 1)),
                    Token::Dot(StrRange::new(3, 1)),
                    Token::Key(StrRange::new(4, 1)),
                    Token::Greater(StrRange::new(5, 1)),
                    Token::At(StrRange::new(6, 1)),
                    Token::Dot(StrRange::new(7, 1)),
                    Token::Key(StrRange::new(8, 1)),
                    Token::CloseParenthesis(StrRange::new(9, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$[:]",
            (
                vec![
                    Token::Absolute(StrRange::new(0, 1)),
                    Token::OpenArray(StrRange::new(1, 1)),
                    Token::Split(StrRange::new(2, 1)),
                    Token::CloseArray(StrRange::new(3, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$['single\'quote']"#,
            (
                vec![
                    Token::Absolute(StrRange::new(0, 1)),
                    Token::OpenArray(StrRange::new(1, 1)),
                    Token::SingleQuoted(StrRange::new(2, r#"'single\'quote'"#.len())),
                    Token::CloseArray(StrRange::new(17, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$['single\'1','single\'2']"#,
            (
                vec![
                    Token::Absolute(StrRange::new(0, 1)),
                    Token::OpenArray(StrRange::new(1, 1)),
                    Token::SingleQuoted(StrRange::new(2, r#"'single\'1'"#.len())),
                    Token::Comma(StrRange::new(13, 1)),
                    Token::SingleQuoted(StrRange::new(14, r#"'single\'2'"#.len())),
                    Token::CloseArray(StrRange::new(25, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$["double\"quote"]"#,
            (
                vec![
                    Token::Absolute(StrRange::new(0, 1)),
                    Token::OpenArray(StrRange::new(1, 1)),
                    Token::DoubleQuoted(StrRange::new(2, r#""double\"quote""#.len())),
                    Token::CloseArray(StrRange::new(17, 1)),
                ],
                Some(TokenError::Eof),
            ),
        );
    }
}