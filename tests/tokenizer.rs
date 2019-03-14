extern crate jsonpath_lib as jsonpath;

use jsonpath::prelude::*;

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
    let mut tokenizer = PreloadedTokenizer::new("$.a");
    match tokenizer.next_token() {
        Ok(t) => assert_eq!(Token::Absolute(0), t),
        _ => panic!()
    }

    match tokenizer.peek_token() {
        Ok(t) => assert_eq!(&Token::Dot(1), t),
        _ => panic!()
    }

    match tokenizer.peek_token() {
        Ok(t) => assert_eq!(&Token::Dot(1), t),
        _ => panic!()
    }

    match tokenizer.next_token() {
        Ok(t) => assert_eq!(Token::Dot(1), t),
        _ => panic!()
    }
}

#[test]
fn token() {
    run("$.01.a",
        (
            vec![
                Token::Absolute(0),
                Token::Dot(1),
                Token::Key(2, "01".to_string()),
                Token::Dot(4),
                Token::Key(5, "a".to_string())
            ]
            , Some(TokenError::Eof)
        ));

    run("$.   []",
        (
            vec![
                Token::Absolute(0),
                Token::Dot(1),
                Token::Whitespace(2, 2),
                Token::OpenArray(5),
                Token::CloseArray(6)
            ]
            , Some(TokenError::Eof)
        ));

    run("$..",
        (
            vec![
                Token::Absolute(0),
                Token::Dot(1),
                Token::Dot(2),
            ]
            , Some(TokenError::Eof)
        ));

    run("$..ab",
        (
            vec![
                Token::Absolute(0),
                Token::Dot(1),
                Token::Dot(2),
                Token::Key(3, "ab".to_string())
            ]
            , Some(TokenError::Eof)
        ));

    run("$..가 [",
        (
            vec![
                Token::Absolute(0),
                Token::Dot(1),
                Token::Dot(2),
                Token::Key(3, "가".to_string()),
                Token::Whitespace(6, 0),
                Token::OpenArray(7),
            ]
            , Some(TokenError::Eof)
        ));

    run("[-1, 2 ]",
        (
            vec![
                Token::OpenArray(0),
                Token::Key(1, "-1".to_string()),
                Token::Comma(3),
                Token::Whitespace(4, 0),
                Token::Key(5, "2".to_string()),
                Token::Whitespace(6, 0),
                Token::CloseArray(7),
            ]
            , Some(TokenError::Eof)
        ));

    run("[ 1 2 , 3 \"abc\" : -10 ]",
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
            ]
            , Some(TokenError::Eof)
        ));

    run("?(@.a가 <41.01)",
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
            ]
            , Some(TokenError::Eof)
        ));

    run("?(@.a <4a.01)",
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
            ]
            , Some(TokenError::Eof)
        ));

    run("?($.c>@.d)", (
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
            Token::CloseParenthesis(9)
        ]
        , Some(TokenError::Eof)
    ));
}