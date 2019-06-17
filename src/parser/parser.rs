use super::tokenizer::*;

const DUMMY: usize = 0;

type ParseResult<T> = Result<T, String>;

mod utils {
    pub fn string_to_isize<F>(string: &String, msg_handler: F) -> Result<isize, String>
        where F: Fn() -> String {
        match string.as_str().parse::<isize>() {
            Ok(n) => Ok(n),
            _ => Err(msg_handler())
        }
    }

    pub fn string_to_f64<F>(string: &String, msg_handler: F) -> Result<f64, String>
        where F: Fn() -> String {
        match string.as_str().parse::<f64>() {
            Ok(n) => Ok(n),
            _ => Err(msg_handler())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseToken {
    // '$'
    Absolute,
    // '@'
    Relative,
    // '.'
    In,
    // '..'
    Leaves,
    // '*'
    All,

    Key(String),
    // []
    Array,
    // 메타토큰
    ArrayEof,
    // ?( filter )
    Filter(FilterToken),
    // 1 : 2
    Range(Option<isize>, Option<isize>),
    // 1, 2, 3
    Union(Vec<isize>),

    Number(f64),

    Bool(bool),

    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterToken {
    Equal,
    NotEqual,
    Little,
    LittleOrEqual,
    Greater,
    GreaterOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    token: ParseToken,
}

pub struct Parser;

impl Parser {
    pub fn compile(input: &str) -> ParseResult<Node> {
        let mut tokenizer = TokenReader::new(input);
        Ok(Self::json_path(&mut tokenizer)?)
    }

    fn json_path(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#json_path");
        match tokenizer.next_token() {
            Ok(Token::Absolute(_)) => {
                let node = Self::node(ParseToken::Absolute);
                Self::paths(node, tokenizer)
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn paths(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#paths");
        match tokenizer.peek_token() {
            Ok(Token::Dot(_)) => {
                Self::eat_token(tokenizer);
                Self::paths_dot(prev, tokenizer)
            }
            Ok(Token::OpenArray(_)) => {
                Self::eat_token(tokenizer);
                Self::eat_whitespace(tokenizer);
                let node = Self::array(prev, tokenizer)?;
                Self::paths(node, tokenizer)
            }
            _ => {
                Ok(prev)
            }
        }
    }

    fn paths_dot(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#paths_dot");
        let node = Self::path(prev, tokenizer)?;
        match tokenizer.peek_token() {
            Ok(Token::Equal(_))
            | Ok(Token::NotEqual(_))
            | Ok(Token::Little(_))
            | Ok(Token::LittleOrEqual(_))
            | Ok(Token::Greater(_))
            | Ok(Token::GreaterOrEqual(_))
            | Ok(Token::And(_))
            | Ok(Token::Or(_)) => {
                Ok(node)
            }
            _ => {
                Self::paths(node, tokenizer)
            }
        }
    }

    fn path(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#path");
        match tokenizer.peek_token() {
            Ok(Token::Dot(_)) => {
                Self::path_leaves(prev, tokenizer)
            }
            Ok(Token::Asterisk(_)) => {
                Self::path_in_all(prev, tokenizer)
            }
            Ok(Token::Key(_, _)) => {
                Self::path_in_key(prev, tokenizer)
            }
            Ok(Token::OpenArray(_)) => {
                Self::eat_token(tokenizer);
                Self::array(prev, tokenizer)
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn path_leaves(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#path_leaves");
        Self::eat_token(tokenizer);
        match tokenizer.peek_token() {
            Ok(Token::Asterisk(_)) => {
                Self::path_leaves_all(prev, tokenizer)
            }
            Ok(Token::OpenArray(_)) => {
                let mut leaves_node = Self::node(ParseToken::Leaves);
                leaves_node.left = Some(Box::new(prev));
                Ok(Self::paths(leaves_node, tokenizer)?)
            }
            _ => {
                Self::path_leaves_key(prev, tokenizer)
            }
        }
    }

    fn path_leaves_key(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#path_leaves_key");
        Ok(Node {
            token: ParseToken::Leaves,
            left: Some(Box::new(prev)),
            right: Some(Box::new(Self::key(tokenizer)?)),
        })
    }

    fn path_leaves_all(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#path_leaves_all");
        Self::eat_token(tokenizer);
        Ok(Node {
            token: ParseToken::Leaves,
            left: Some(Box::new(prev)),
            right: Some(Box::new(Self::node(ParseToken::All))),
        })
    }

    fn path_in_all(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#path_in_all");
        Self::eat_token(tokenizer);
        Ok(Node {
            token: ParseToken::In,
            left: Some(Box::new(prev)),
            right: Some(Box::new(Self::node(ParseToken::All))),
        })
    }

    fn path_in_key(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#path_in_key");
        Ok(Node {
            token: ParseToken::In,
            left: Some(Box::new(prev)),
            right: Some(Box::new(Self::key(tokenizer)?)),
        })
    }

    fn key(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#key");
        match tokenizer.next_token() {
            Ok(Token::Key(_, v)) => {
                Ok(Self::node(ParseToken::Key(v)))
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn boolean(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#boolean");
        match tokenizer.next_token() {
            Ok(Token::Key(_, v)) => {
                Ok(Self::node(ParseToken::Bool(v.eq_ignore_ascii_case("true"))))
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn array_quota_value(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#array_quota_value");
        match tokenizer.next_token() {
            Ok(Token::SingleQuoted(_, val))
            | Ok(Token::DoubleQuoted(_, val)) => {
                Ok(Self::node(ParseToken::Key(val)))
            }
            Err(TokenError::Eof) => {
                Ok(Self::node(ParseToken::Eof))
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn array_start(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#array_start");
        match tokenizer.peek_token() {
            Ok(Token::Question(_)) => {
                Self::eat_token(tokenizer);
                Ok(Node {
                    token: ParseToken::Array,
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(Self::filter(tokenizer)?)),
                })
            }
            Ok(Token::Asterisk(_)) => {
                Self::eat_token(tokenizer);
                Ok(Node {
                    token: ParseToken::Array,
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(Self::node(ParseToken::All))),
                })
            }
            _ => {
                Ok(Node {
                    token: ParseToken::Array,
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(Self::array_value(tokenizer)?)),
                })
            }
        }
    }

    fn array(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#array");
        let ret = Self::array_start(prev, tokenizer)?;
        Self::eat_whitespace(tokenizer);
        Self::close_token(ret, Token::CloseArray(DUMMY), tokenizer)
    }

    fn array_value_key(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#array_value_key");
        match tokenizer.next_token() {
            Ok(Token::Key(pos, ref val)) => {
                let digit = utils::string_to_isize(val, || tokenizer.err_msg_with_pos(pos))?;
                Self::eat_whitespace(tokenizer);

                match tokenizer.peek_token() {
                    Ok(Token::Comma(_)) => {
                        Self::union(digit, tokenizer)
                    }
                    Ok(Token::Split(_)) => {
                        Self::range_from(digit, tokenizer)
                    }
                    _ => {
                        Ok(Self::node(ParseToken::Number(digit as f64)))
                    }
                }
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }


    fn array_value(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#array_value");
        match tokenizer.peek_token() {
            Ok(Token::Key(_, _)) => {
                Self::array_value_key(tokenizer)
            }
            Ok(Token::Split(_)) => {
                Self::eat_token(tokenizer);
                Self::range_to(tokenizer)
            }
            Ok(Token::DoubleQuoted(_, _))
            | Ok(Token::SingleQuoted(_, _)) => {
                Self::array_quota_value(tokenizer)
            }
            Err(TokenError::Eof) => {
                Ok(Self::node(ParseToken::Eof))
            }
            _ => {
                Self::eat_token(tokenizer);
                Err(tokenizer.err_msg())
            }
        }
    }

    fn union(num: isize, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#union");
        let mut values = vec![num];
        while match tokenizer.peek_token() {
            Ok(Token::Comma(_)) => true,
            _ => false
        } {
            Self::eat_token(tokenizer);
            Self::eat_whitespace(tokenizer);
            match tokenizer.next_token() {
                Ok(Token::Key(pos, ref val)) => {
                    let digit = utils::string_to_isize(val, || tokenizer.err_msg_with_pos(pos))?;
                    values.push(digit);
                }
                _ => {
                    return Err(tokenizer.err_msg());
                }
            }
        }
        Ok(Self::node(ParseToken::Union(values)))
    }

    fn range_from(num: isize, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#range_from");
        Self::eat_token(tokenizer);
        Self::eat_whitespace(tokenizer);
        match tokenizer.peek_token() {
            Ok(Token::Key(_, _)) => {
                Self::range(num, tokenizer)
            }
            _ => {
                Ok(Self::node(ParseToken::Range(Some(num), None)))
            }
        }
    }

    fn range_to(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#range_to");
        match tokenizer.peek_token() {
            Ok(Token::CloseArray(_)) => {
                return Ok(Self::node(ParseToken::Range(None, None)))
            }
            _ => {}
        }

        match tokenizer.next_token() {
            Ok(Token::Key(pos, ref val)) => {
                let digit = utils::string_to_isize(val, || tokenizer.err_msg_with_pos(pos))?;
                Ok(Self::node(ParseToken::Range(None, Some(digit))))
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn range(num: isize, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#range");
        match tokenizer.next_token() {
            Ok(Token::Key(pos, ref val)) => {
                let digit = utils::string_to_isize(val, || tokenizer.err_msg_with_pos(pos))?;
                Ok(Self::node(ParseToken::Range(Some(num), Some(digit))))
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn filter(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#filter");
        match tokenizer.next_token() {
            Ok(Token::OpenParenthesis(_)) => {
                let ret = Self::exprs(tokenizer)?;
                Self::eat_whitespace(tokenizer);
                Self::close_token(ret, Token::CloseParenthesis(DUMMY), tokenizer)
            }
            Err(TokenError::Eof) => {
                Ok(Self::node(ParseToken::Eof))
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn exprs(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        Self::eat_whitespace(tokenizer);
        debug!("#exprs");
        let node = match tokenizer.peek_token() {
            Ok(Token::OpenParenthesis(_)) => {
                Self::eat_token(tokenizer);
                trace!("\t-exprs - open_parenthesis");
                let ret = Self::exprs(tokenizer)?;
                Self::eat_whitespace(tokenizer);
                Self::close_token(ret, Token::CloseParenthesis(DUMMY), tokenizer)?
            }
            _ => {
                trace!("\t-exprs - else");
                Self::expr(tokenizer)?
            }
        };
        Self::eat_whitespace(tokenizer);
        Self::condition_expr(node, tokenizer)
    }

    fn condition_expr(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#condition_expr");
        match tokenizer.peek_token() {
            Ok(Token::And(_)) => {
                Self::eat_token(tokenizer);
                Ok(Node {
                    token: ParseToken::Filter(FilterToken::And),
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(Self::exprs(tokenizer)?)),
                })
            }
            Ok(Token::Or(_)) => {
                Self::eat_token(tokenizer);
                Ok(Node {
                    token: ParseToken::Filter(FilterToken::Or),
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(Self::exprs(tokenizer)?)),
                })
            }
            _ => {
                Ok(prev)
            }
        }
    }

    fn expr(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#expr");

        let has_prop_candidate = match tokenizer.peek_token() {
            Ok(Token::At(_)) => true,
            _ => false
        };

        let node = Self::term(tokenizer)?;
        Self::eat_whitespace(tokenizer);

        if match tokenizer.peek_token() {
            Ok(Token::Equal(_))
            | Ok(Token::NotEqual(_))
            | Ok(Token::Little(_))
            | Ok(Token::LittleOrEqual(_))
            | Ok(Token::Greater(_))
            | Ok(Token::GreaterOrEqual(_)) => true,
            _ => false
        } {
            Self::op(node, tokenizer)
        } else if has_prop_candidate {
            Ok(node)
        } else {
            return Err(tokenizer.err_msg());
        }
    }

    fn term_num(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#term_num");
        match tokenizer.next_token() {
            Ok(Token::Key(pos, val)) => {
                match tokenizer.peek_token() {
                    Ok(Token::Dot(_)) => {
                        Self::term_num_float(val.as_str(), tokenizer)
                    }
                    _ => {
                        let number = utils::string_to_f64(&val, || tokenizer.err_msg_with_pos(pos))?;
                        Ok(Self::node(ParseToken::Number(number)))
                    }
                }
            }
            Err(TokenError::Eof) => {
                Ok(Self::node(ParseToken::Eof))
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn term_num_float(mut num: &str, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#term_num_float");
        Self::eat_token(tokenizer);
        match tokenizer.next_token() {
            Ok(Token::Key(pos, frac)) => {
                let mut f = String::new();
                f.push_str(&mut num);
                f.push('.');
                f.push_str(frac.as_str());
                let number = utils::string_to_f64(&f, || tokenizer.err_msg_with_pos(pos))?;
                Ok(Self::node(ParseToken::Number(number)))
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }

    fn peek_key(tokenizer: &mut TokenReader) -> Option<String> {
        if let Ok(Token::Key(_, k)) = tokenizer.peek_token() {
            Some(k.clone())
        } else {
            None
        }
    }

    fn term(tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#term");

        if tokenizer.peek_is(AT) {
            Self::eat_token(tokenizer);
            let node = Self::node(ParseToken::Relative);

            return match tokenizer.peek_token() {
                Ok(Token::Whitespace(_, _)) => {
                    Self::eat_whitespace(tokenizer);
                    Ok(node)
                }
                _ => {
                    Self::paths(node, tokenizer)
                }
            };
        }

        if tokenizer.peek_is(ABSOLUTE) {
            return Self::json_path(tokenizer);
        }

        if tokenizer.peek_is(DOUBLE_QUOTA) || tokenizer.peek_is(SINGLE_QUOTA) {
            return Self::array_quota_value(tokenizer);
        }

        if tokenizer.peek_is(KEY) {
            return match Self::peek_key(tokenizer) {
                Some(key) => match key.chars().next() {
                    Some(ch) => match ch {
                        '-' | '0'...'9' => Self::term_num(tokenizer),
                        _ => Self::boolean(tokenizer)
                    }
                    _ => Err(tokenizer.err_msg())
                },
                _ => Err(tokenizer.err_msg())
            };
        }

        return Err(tokenizer.err_msg());
    }

    fn op(prev: Node, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#op");
        let token = match tokenizer.next_token() {
            Ok(Token::Equal(_)) => {
                ParseToken::Filter(FilterToken::Equal)
            }
            Ok(Token::NotEqual(_)) => {
                ParseToken::Filter(FilterToken::NotEqual)
            }
            Ok(Token::Little(_)) => {
                ParseToken::Filter(FilterToken::Little)
            }
            Ok(Token::LittleOrEqual(_)) => {
                ParseToken::Filter(FilterToken::LittleOrEqual)
            }
            Ok(Token::Greater(_)) => {
                ParseToken::Filter(FilterToken::Greater)
            }
            Ok(Token::GreaterOrEqual(_)) => {
                ParseToken::Filter(FilterToken::GreaterOrEqual)
            }
            Err(TokenError::Eof) => {
                ParseToken::Eof
            }
            _ => {
                return Err(tokenizer.err_msg());
            }
        };

        Self::eat_whitespace(tokenizer);

        Ok(Node {
            token,
            left: Some(Box::new(prev)),
            right: Some(Box::new(Self::term(tokenizer)?)),
        })
    }

    fn eat_whitespace(tokenizer: &mut TokenReader) {
        while let Ok(Token::Whitespace(_, _)) = tokenizer.peek_token() {
            let _ = tokenizer.next_token();
        }
    }

    fn eat_token(tokenizer: &mut TokenReader) {
        let _ = tokenizer.next_token();
    }

    fn node(token: ParseToken) -> Node {
        Node { left: None, right: None, token }
    }

    fn close_token(ret: Node, token: Token, tokenizer: &mut TokenReader) -> ParseResult<Node> {
        debug!("#close_token");
        match tokenizer.next_token() {
            Ok(ref t) if t.partial_eq(token) => {
                Ok(ret)
            }
            _ => {
                Err(tokenizer.err_msg())
            }
        }
    }
}

pub trait NodeVisitor {
    fn visit(&mut self, node: &Node) {
        match &node.token {
            ParseToken::Absolute
            | ParseToken::Relative
            | ParseToken::All
            | ParseToken::Key(_)
            | ParseToken::Range(_, _)
            | ParseToken::Union(_)
            | ParseToken::Number(_)
            | ParseToken::Bool(_) => {
                self.visit_token(&node.token);
            }
            ParseToken::In
            | ParseToken::Leaves => {
                match &node.left {
                    Some(n) => self.visit(&*n),
                    _ => {}
                }

                self.visit_token(&node.token);

                match &node.right {
                    Some(n) => self.visit(&*n),
                    _ => {}
                }
            }
            ParseToken::Array => {
                match &node.left {
                    Some(n) => self.visit(&*n),
                    _ => {}
                }

                self.visit_token(&node.token);

                match &node.right {
                    Some(n) => self.visit(&*n),
                    _ => {}
                }
                self.visit_token(&ParseToken::ArrayEof);
            }
            ParseToken::Filter(FilterToken::And)
            | ParseToken::Filter(FilterToken::Or) => {
                match &node.left {
                    Some(n) => self.visit(&*n),
                    _ => {}
                }

                match &node.right {
                    Some(n) => self.visit(&*n),
                    _ => {}
                }

                self.visit_token(&node.token);
            }
            ParseToken::Filter(_) => {
                match &node.left {
                    Some(n) => self.visit(&*n),
                    _ => {}
                }

                self.end_term();

                match &node.right {
                    Some(n) => self.visit(&*n),
                    _ => {}
                }

                self.end_term();

                self.visit_token(&node.token);
            }
            _ => {}
        }
    }

    fn visit_token(&mut self, token: &ParseToken);
    fn end_term(&mut self) {}
}