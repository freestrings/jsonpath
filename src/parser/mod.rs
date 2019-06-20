pub mod parser;
mod path_reader;
pub(crate) mod tokenizer;

#[cfg(test)]
mod parser_tests {
    use parser::parser::{FilterToken, NodeVisitor, ParseToken, Parser};

    struct NodeVisitorTestImpl<'a> {
        input: &'a str,
        stack: Vec<ParseToken>,
    }

    impl<'a> NodeVisitorTestImpl<'a> {
        fn new(input: &'a str) -> Self {
            NodeVisitorTestImpl {
                input,
                stack: Vec::new(),
            }
        }

        fn start(&mut self) -> Result<Vec<ParseToken>, String> {
            let node = Parser::compile(self.input)?;
            self.visit(&node);
            Ok(self.stack.split_off(0))
        }
    }

    impl<'a> NodeVisitor for NodeVisitorTestImpl<'a> {
        fn visit_token(&mut self, token: &ParseToken) {
            self.stack.push(token.clone());
        }
    }

    fn setup() {
        let _ = env_logger::try_init();
    }

    fn run(input: &str) -> Result<Vec<ParseToken>, String> {
        let mut interpreter = NodeVisitorTestImpl::new(input);
        interpreter.start()
    }

    #[test]
    fn parse_error() {
        setup();

        fn invalid(path: &str) {
            if let Err(_) = run(path) {
                assert!(true);
            } else {
                assert!(false);
            }
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
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("aa".to_owned())
            ])
        );

        assert_eq!(
            run("$.00.a"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("00".to_owned()),
                ParseToken::In,
                ParseToken::Key("a".to_owned())
            ])
        );

        assert_eq!(
            run("$.00.韓창.seok"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("00".to_owned()),
                ParseToken::In,
                ParseToken::Key("韓창".to_owned()),
                ParseToken::In,
                ParseToken::Key("seok".to_owned())
            ])
        );

        assert_eq!(
            run("$.*"),
            Ok(vec![ParseToken::Absolute, ParseToken::In, ParseToken::All])
        );

        assert_eq!(
            run("$..*"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Leaves,
                ParseToken::All
            ])
        );

        assert_eq!(
            run("$..[0]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Leaves,
                ParseToken::Array,
                ParseToken::Number(0.0),
                ParseToken::ArrayEof
            ])
        );

        match run("$.") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$..") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$. a") {
            Ok(_) => panic!(),
            _ => {}
        }
    }

    #[test]
    fn parse_array_sytax() {
        setup();

        assert_eq!(
            run("$.book[?(@.isbn)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("book".to_string()),
                ParseToken::Array,
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key("isbn".to_string()),
                ParseToken::ArrayEof
            ])
        );

        //
        // Array도 컨텍스트 In으로 간주 할거라서 중첩되면 하나만
        //
        assert_eq!(
            run("$.[*]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[*]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[*].가"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof,
                ParseToken::In,
                ParseToken::Key("가".to_owned())
            ])
        );

        assert_eq!(
            run("$.a[0][1]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::Number(0_f64),
                ParseToken::ArrayEof,
                ParseToken::Array,
                ParseToken::Number(1_f64),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[1,2]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::Union(vec![1, 2]),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[10:]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::Range(Some(10), None, None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[:11]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::Range(None, Some(11), None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[-12:13]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::Range(Some(-12), Some(13), None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[0:3:2]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(Some(0), Some(3), Some(2)),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[:3:2]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, Some(3), Some(2)),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[:]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, None, None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[::]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, None, None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[::2]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, None, Some(2)),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$["a", 'b']"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Keys(vec!["a".to_string(), "b".to_string()]),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[?(1>2)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::Number(1_f64),
                ParseToken::Number(2_f64),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[?($.b>3)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Array,
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("b".to_owned()),
                ParseToken::Number(3_f64),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[?($.c>@.d && 1==2)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("c".to_owned()),
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key("d".to_owned()),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::Number(1_f64),
                ParseToken::Number(2_f64),
                ParseToken::Filter(FilterToken::Equal),
                ParseToken::Filter(FilterToken::And),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[?($.c>@.d&&(1==2||3>=4))]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("c".to_owned()),
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key("d".to_owned()),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::Number(1_f64),
                ParseToken::Number(2_f64),
                ParseToken::Filter(FilterToken::Equal),
                ParseToken::Number(3_f64),
                ParseToken::Number(4_f64),
                ParseToken::Filter(FilterToken::GreaterOrEqual),
                ParseToken::Filter(FilterToken::Or),
                ParseToken::Filter(FilterToken::And),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[?(@.a<@.b)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key("a".to_owned()),
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key("b".to_owned()),
                ParseToken::Filter(FilterToken::Little),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[*][*][*]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof,
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof,
                ParseToken::Array,
                ParseToken::All,
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$['a']['bb']"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Key("a".to_string()),
                ParseToken::ArrayEof,
                ParseToken::Array,
                ParseToken::Key("bb".to_string()),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$.a[?(@.e==true)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::In,
                ParseToken::Key("a".to_string()),
                ParseToken::Array,
                ParseToken::Relative,
                ParseToken::In,
                ParseToken::Key("e".to_string()),
                ParseToken::Bool(true),
                ParseToken::Filter(FilterToken::Equal),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$[?(@ > 1)]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Relative,
                ParseToken::Number(1_f64),
                ParseToken::Filter(FilterToken::Greater),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run("$[:]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Range(None, None, None),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$['single\'quote']"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Key("single'quote".to_string()),
                ParseToken::ArrayEof
            ])
        );

        assert_eq!(
            run(r#"$["single\"quote"]"#),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Key(r#"single"quote"#.to_string()),
                ParseToken::ArrayEof
            ])
        );
    }

    #[test]
    fn parse_array_float() {
        setup();

        assert_eq!(
            run("$[?(1.1<2.1)]"),
            Ok(vec![
                ParseToken::Absolute,
                ParseToken::Array,
                ParseToken::Number(1.1),
                ParseToken::Number(2.1),
                ParseToken::Filter(FilterToken::Little),
                ParseToken::ArrayEof
            ])
        );

        match run("$[1.1]") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$[?(1.1<.2)]") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$[?(1.1<2.)]") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$[?(1.1<2.a)]") {
            Ok(_) => panic!(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tokenizer_tests {
    use parser::tokenizer::{Token, TokenError, TokenReader, Tokenizer};

    fn setup() {
        let _ = env_logger::try_init();
    }

    fn collect_token(input: &str) -> (Vec<Token>, Option<TokenError>) {
        let mut tokenizer = Tokenizer::new(input);
        let mut vec = vec![];
        loop {
            match tokenizer.next_token() {
                Ok(t) => vec.push(t),
                Err(e) => return (vec, Some(e)),
            }
        }
    }

    fn run(input: &str, expected: (Vec<Token>, Option<TokenError>)) {
        let (vec, err) = collect_token(input.clone());
        assert_eq!((vec, err), expected, "\"{}\"", input);
    }

    #[test]
    fn peek() {
        let mut tokenizer = TokenReader::new("$.a");
        match tokenizer.next_token() {
            Ok(t) => assert_eq!(Token::Absolute(0), t),
            _ => panic!(),
        }

        match tokenizer.peek_token() {
            Ok(t) => assert_eq!(&Token::Dot(1), t),
            _ => panic!(),
        }

        match tokenizer.peek_token() {
            Ok(t) => assert_eq!(&Token::Dot(1), t),
            _ => panic!(),
        }

        match tokenizer.next_token() {
            Ok(t) => assert_eq!(Token::Dot(1), t),
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
                    Token::Absolute(0),
                    Token::Dot(1),
                    Token::Key(2, "01".to_string()),
                    Token::Dot(4),
                    Token::Key(5, "a".to_string()),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$.   []",
            (
                vec![
                    Token::Absolute(0),
                    Token::Dot(1),
                    Token::Whitespace(2, 2),
                    Token::OpenArray(5),
                    Token::CloseArray(6),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..",
            (
                vec![Token::Absolute(0), Token::Dot(1), Token::Dot(2)],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..ab",
            (
                vec![
                    Token::Absolute(0),
                    Token::Dot(1),
                    Token::Dot(2),
                    Token::Key(3, "ab".to_string()),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$..가 [",
            (
                vec![
                    Token::Absolute(0),
                    Token::Dot(1),
                    Token::Dot(2),
                    Token::Key(3, "가".to_string()),
                    Token::Whitespace(6, 0),
                    Token::OpenArray(7),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "[-1, 2 ]",
            (
                vec![
                    Token::OpenArray(0),
                    Token::Key(1, "-1".to_string()),
                    Token::Comma(3),
                    Token::Whitespace(4, 0),
                    Token::Key(5, "2".to_string()),
                    Token::Whitespace(6, 0),
                    Token::CloseArray(7),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "[ 1 2 , 3 \"abc\" : -10 ]",
            (
                vec![
                    Token::OpenArray(0),
                    Token::Whitespace(1, 0),
                    Token::Key(2, "1".to_string()),
                    Token::Whitespace(3, 0),
                    Token::Key(4, "2".to_string()),
                    Token::Whitespace(5, 0),
                    Token::Comma(6),
                    Token::Whitespace(7, 0),
                    Token::Key(8, "3".to_string()),
                    Token::Whitespace(9, 0),
                    Token::DoubleQuoted(10, "abc".to_string()),
                    Token::Whitespace(15, 0),
                    Token::Split(16),
                    Token::Whitespace(17, 0),
                    Token::Key(18, "-10".to_string()),
                    Token::Whitespace(21, 0),
                    Token::CloseArray(22),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?(@.a가 <41.01)",
            (
                vec![
                    Token::Question(0),
                    Token::OpenParenthesis(1),
                    Token::At(2),
                    Token::Dot(3),
                    Token::Key(4, "a가".to_string()),
                    Token::Whitespace(8, 0),
                    Token::Little(9),
                    Token::Key(10, "41".to_string()),
                    Token::Dot(12),
                    Token::Key(13, "01".to_string()),
                    Token::CloseParenthesis(15),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?(@.a <4a.01)",
            (
                vec![
                    Token::Question(0),
                    Token::OpenParenthesis(1),
                    Token::At(2),
                    Token::Dot(3),
                    Token::Key(4, "a".to_string()),
                    Token::Whitespace(5, 0),
                    Token::Little(6),
                    Token::Key(7, "4a".to_string()),
                    Token::Dot(9),
                    Token::Key(10, "01".to_string()),
                    Token::CloseParenthesis(12),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "?($.c>@.d)",
            (
                vec![
                    Token::Question(0),
                    Token::OpenParenthesis(1),
                    Token::Absolute(2),
                    Token::Dot(3),
                    Token::Key(4, "c".to_string()),
                    Token::Greater(5),
                    Token::At(6),
                    Token::Dot(7),
                    Token::Key(8, "d".to_string()),
                    Token::CloseParenthesis(9),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            "$[:]",
            (
                vec![
                    Token::Absolute(0),
                    Token::OpenArray(1),
                    Token::Split(2),
                    Token::CloseArray(3),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$['single\'quote']"#,
            (
                vec![
                    Token::Absolute(0),
                    Token::OpenArray(1),
                    Token::SingleQuoted(2, "single\'quote".to_string()),
                    Token::CloseArray(17),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$['single\'1','single\'2']"#,
            (
                vec![
                    Token::Absolute(0),
                    Token::OpenArray(1),
                    Token::SingleQuoted(2, "single\'1".to_string()),
                    Token::Comma(13),
                    Token::SingleQuoted(14, "single\'2".to_string()),
                    Token::CloseArray(25),
                ],
                Some(TokenError::Eof),
            ),
        );

        run(
            r#"$["double\"quote"]"#,
            (
                vec![
                    Token::Absolute(0),
                    Token::OpenArray(1),
                    Token::DoubleQuoted(2, "double\"quote".to_string()),
                    Token::CloseArray(17),
                ],
                Some(TokenError::Eof),
            ),
        );
    }
}
