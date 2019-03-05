use std::result;

use super::tokenizer::{
    PreloadedTokenizer,
    Token,
    TokenError,
};

const DUMMY: usize = 0;

type Result<T> = result::Result<T, String>;

mod utils {
    use std::result;

    pub fn vec_to_int<F>(vec: &Vec<char>, msg_handler: F) -> result::Result<isize, String>
        where F: Fn() -> String {
        match vec.iter().map(|c| *c).collect::<String>().as_str().parse::<isize>() {
            Ok(n) => Ok(n),
            _ => Err(msg_handler())
        }
    }

    pub fn vec_to_float<F>(vec: &Vec<char>, msg_handler: F) -> result::Result<f64, String>
        where F: Fn() -> String {
        match vec.iter().map(|c| *c).collect::<String>().as_str().parse::<f64>() {
            Ok(n) => Ok(n),
            _ => Err(msg_handler())
        }
    }

    pub fn vec_to_string(vec: &Vec<char>) -> String {
        vec.iter().map(|c| *c).collect::<String>()
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

pub struct Parser<'a> {
    tokenizer: PreloadedTokenizer<'a>
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser { tokenizer: PreloadedTokenizer::new(input) }
    }

    pub fn compile(&mut self) -> Result<Node> {
        Ok(self.json_path()?)
    }

    pub fn parse<V: NodeVisitor>(&mut self, visitor: &mut V) -> Result<()> {
        let node = self.json_path()?;
        visitor.visit(node);
        Ok(())
    }

    fn json_path(&mut self) -> Result<Node> {
        debug!("#json_path");
        match self.tokenizer.next_token() {
            Ok(Token::Absolute(_)) => {
                let node = self.node(ParseToken::Absolute);
                self.paths(node)
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn paths(&mut self, prev: Node) -> Result<Node> {
        debug!("#paths");
        match self.tokenizer.peek_token() {
            Ok(Token::Dot(_)) => {
                self.eat_token();
                self.paths_dot(prev)
            }
            Ok(Token::OpenArray(_)) => {
                self.eat_token();
                self.eat_whitespace();
                let node = self.array(prev)?;
                self.paths(node)
            }
            _ => {
                Ok(prev)
            }
        }
    }

    fn paths_dot(&mut self, prev: Node) -> Result<Node> {
        debug!("#paths_dot");
        let node = self.path(prev)?;
        match self.tokenizer.peek_token() {
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
                self.paths(node)
            }
        }
    }

    fn path(&mut self, prev: Node) -> Result<Node> {
        debug!("#path");
        match self.tokenizer.peek_token() {
            Ok(Token::Dot(_)) => {
                self.path_leaves(prev)
            }
            Ok(Token::Asterisk(_)) => {
                self.path_in_all(prev)
            }
            Ok(Token::Key(_, _)) => {
                self.path_in_key(prev)
            }
            Ok(Token::OpenArray(_)) => {
                self.eat_token();
                self.array(prev)
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn path_leaves(&mut self, prev: Node) -> Result<Node> {
        debug!("#path_leaves");
        self.eat_token();
        match self.tokenizer.peek_token() {
            Ok(Token::Asterisk(_)) => {
                self.path_leaves_all(prev)
            }
            Ok(Token::OpenArray(_)) => {
                let mut leaves_node = self.node(ParseToken::Leaves);
                leaves_node.left = Some(Box::new(prev));
                Ok(self.paths(leaves_node)?)
            }
            _ => {
                self.path_leaves_key(prev)
            }
        }
    }

    fn path_leaves_key(&mut self, prev: Node) -> Result<Node> {
        debug!("#path_leaves_key");
        Ok(Node {
            token: ParseToken::Leaves,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.key()?)),
        })
    }

    fn path_leaves_all(&mut self, prev: Node) -> Result<Node> {
        debug!("#path_leaves_all");
        self.eat_token();
        Ok(Node {
            token: ParseToken::Leaves,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.node(ParseToken::All))),
        })
    }

    fn path_in_all(&mut self, prev: Node) -> Result<Node> {
        debug!("#path_in_all");
        self.eat_token();
        Ok(Node {
            token: ParseToken::In,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.node(ParseToken::All))),
        })
    }

    fn path_in_key(&mut self, prev: Node) -> Result<Node> {
        debug!("#path_in_key");
        Ok(Node {
            token: ParseToken::In,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.key()?)),
        })
    }

    fn key(&mut self) -> Result<Node> {
        debug!("#key");
        match self.tokenizer.next_token() {
            Ok(Token::Key(_, v)) => {
                Ok(self.node(ParseToken::Key(utils::vec_to_string(&v))))
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn array_quota_value(&mut self) -> Result<Node> {
        debug!("#array_quota_value");
        match self.tokenizer.next_token() {
            Ok(Token::SingleQuoted(_, ref vec))
            | Ok(Token::DoubleQuoted(_, ref vec)) => {
                Ok(self.node(ParseToken::Key(utils::vec_to_string(vec))))
            }
            Err(TokenError::Eof) => {
                Ok(self.node(ParseToken::Eof))
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn array_start(&mut self, prev: Node) -> Result<Node> {
        debug!("#array_start");
        match self.tokenizer.peek_token() {
            Ok(Token::Question(_)) => {
                self.eat_token();
                Ok(Node {
                    token: ParseToken::Array,
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.filter()?)),
                })
            }
            Ok(Token::Asterisk(_)) => {
                self.eat_token();
                Ok(Node {
                    token: ParseToken::Array,
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.node(ParseToken::All))),
                })
            }
            _ => {
                Ok(Node {
                    token: ParseToken::Array,
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.array_value()?)),
                })
            }
        }
    }

    fn array(&mut self, prev: Node) -> Result<Node> {
        debug!("#array");
        let ret = self.array_start(prev)?;
        self.eat_whitespace();
        self.close_token(ret, Token::CloseArray(DUMMY))
    }

    fn array_value_key(&mut self) -> Result<Node> {
        debug!("#array_value_key");
        match self.tokenizer.next_token() {
            Ok(Token::Key(pos, ref vec)) => {
                let digit = utils::vec_to_int(vec,
                                              || self.tokenizer.err_msg_with_pos(pos))?;
                self.eat_whitespace();

                match self.tokenizer.peek_token() {
                    Ok(Token::Comma(_)) => {
                        self.union(digit)
                    }
                    Ok(Token::Split(_)) => {
                        self.range_from(digit)
                    }
                    _ => {
                        Ok(self.node(ParseToken::Number(digit as f64)))
                    }
                }
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }


    fn array_value(&mut self) -> Result<Node> {
        debug!("#array_value");
        match self.tokenizer.peek_token() {
            Ok(Token::Key(_, _)) => {
                self.array_value_key()
            }
            Ok(Token::Split(_)) => {
                self.eat_token();
                self.range_to()
            }
            Ok(Token::DoubleQuoted(_, _))
            | Ok(Token::SingleQuoted(_, _)) => {
                self.array_quota_value()
            }
            Err(TokenError::Eof) => {
                Ok(self.node(ParseToken::Eof))
            }
            _ => {
                self.eat_token();
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn union(&mut self, num: isize) -> Result<Node> {
        debug!("#union");
        let mut values = vec![num];
        while match self.tokenizer.peek_token() {
            Ok(Token::Comma(_)) => true,
            _ => false
        } {
            self.eat_token();
            self.eat_whitespace();
            match self.tokenizer.next_token() {
                Ok(Token::Key(pos, ref vec)) => {
                    let digit = utils::vec_to_int(vec,
                                                  || self.tokenizer.err_msg_with_pos(pos))?;
                    values.push(digit);
                }
                _ => {
                    return Err(self.tokenizer.err_msg());
                }
            }
        }
        Ok(self.node(ParseToken::Union(values)))
    }

    fn range_from(&mut self, num: isize) -> Result<Node> {
        debug!("#range_from");
        self.eat_token();
        self.eat_whitespace();
        match self.tokenizer.peek_token() {
            Ok(Token::Key(_, _)) => {
                self.range(num)
            }
            _ => {
                Ok(self.node(ParseToken::Range(Some(num), None)))
            }
        }
    }

    fn range_to(&mut self) -> Result<Node> {
        debug!("#range_to");
        match self.tokenizer.next_token() {
            Ok(Token::Key(pos, ref vec)) => {
                let digit = utils::vec_to_int(vec,
                                              || self.tokenizer.err_msg_with_pos(pos))?;
                Ok(self.node(ParseToken::Range(None, Some(digit))))
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn range(&mut self, num: isize) -> Result<Node> {
        debug!("#range");
        match self.tokenizer.next_token() {
            Ok(Token::Key(pos, ref vec)) => {
                let digit = utils::vec_to_int(vec,
                                              || self.tokenizer.err_msg_with_pos(pos))?;
                Ok(self.node(ParseToken::Range(Some(num), Some(digit))))
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn filter(&mut self) -> Result<Node> {
        debug!("#filter");
        match self.tokenizer.next_token() {
            Ok(Token::OpenParenthesis(_)) => {
                let ret = self.exprs()?;
                self.eat_whitespace();
                self.close_token(ret, Token::CloseParenthesis(DUMMY))
            }
            Err(TokenError::Eof) => {
                Ok(self.node(ParseToken::Eof))
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn exprs(&mut self) -> Result<Node> {
        self.eat_whitespace();
        debug!("#exprs");
        let node = match self.tokenizer.peek_token() {
            Ok(Token::OpenParenthesis(_)) => {
                self.eat_token();
                trace!("\t-exprs - open_parenthesis");
                let ret = self.exprs()?;
                self.eat_whitespace();
                self.close_token(ret, Token::CloseParenthesis(DUMMY))?
            }
            _ => {
                trace!("\t-exprs - else");
                self.expr()?
            }
        };
        self.eat_whitespace();
        self.condition_expr(node)
    }

    fn condition_expr(&mut self, prev: Node) -> Result<Node> {
        debug!("#condition_expr");
        match self.tokenizer.peek_token() {
            Ok(Token::And(_)) => {
                self.eat_token();
                Ok(Node {
                    token: ParseToken::Filter(FilterToken::And),
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.exprs()?)),
                })
            }
            Ok(Token::Or(_)) => {
                self.eat_token();
                Ok(Node {
                    token: ParseToken::Filter(FilterToken::Or),
                    left: Some(Box::new(prev)),
                    right: Some(Box::new(self.exprs()?)),
                })
            }
            _ => {
                Ok(prev)
            }
        }
    }

    fn expr(&mut self) -> Result<Node> {
        debug!("#expr");

        let has_prop_candidate = match self.tokenizer.peek_token() {
            Ok(Token::At(_)) => true,
            _ => false
        };

        let node = self.term()?;
        self.eat_whitespace();

        if match self.tokenizer.peek_token() {
            Ok(Token::Equal(_))
            | Ok(Token::NotEqual(_))
            | Ok(Token::Little(_))
            | Ok(Token::LittleOrEqual(_))
            | Ok(Token::Greater(_))
            | Ok(Token::GreaterOrEqual(_)) => true,
            _ => false
        } {
            self.op(node)
        } else if has_prop_candidate {
            Ok(node)
        } else {
            return Err(self.tokenizer.err_msg());
        }
    }

    fn term_num(&mut self) -> Result<Node> {
        debug!("#term_num");
        match self.tokenizer.next_token() {
            Ok(Token::Key(pos, mut vec)) => {
                match self.tokenizer.peek_token() {
                    Ok(Token::Dot(_)) => {
                        self.term_num_float(&mut vec)
                    }
                    _ => {
                        let number = utils::vec_to_float(&vec,
                                                         || self.tokenizer.err_msg_with_pos(pos))?;
                        Ok(self.node(ParseToken::Number(number)))
                    }
                }
            }
            Err(TokenError::Eof) => {
                Ok(self.node(ParseToken::Eof))
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn term_num_float(&mut self, mut num: &mut Vec<char>) -> Result<Node> {
        debug!("#term_num_float");
        self.eat_token();
        match self.tokenizer.next_token() {
            Ok(Token::Key(pos, mut frac)) => {
                let mut f = vec![];
                f.append(&mut num);
                f.push('.');
                f.append(&mut frac);
                let number = utils::vec_to_float(&f,
                                                 || self.tokenizer.err_msg_with_pos(pos))?;
                Ok(self.node(ParseToken::Number(number)))
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn term(&mut self) -> Result<Node> {
        debug!("#term");
        match self.tokenizer.peek_token() {
            Ok(Token::At(_)) => {
                self.eat_token();
                let node = self.node(ParseToken::Relative);

                match self.tokenizer.peek_token() {
                    Ok(Token::Whitespace(_, _)) => {
                        self.eat_whitespace();
                        Ok(node)
                    }
                    _ => {
                        self.paths(node)
                    }
                }
            }
            Ok(Token::Absolute(_)) => {
                self.json_path()
            }
            Ok(Token::DoubleQuoted(_, _))
            | Ok(Token::SingleQuoted(_, _)) => {
                self.array_quota_value()
            }
            Ok(Token::Key(_, _)) => {
                self.term_num()
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }

    fn op(&mut self, prev: Node) -> Result<Node> {
        debug!("#op");
        let token = match self.tokenizer.next_token() {
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
                return Err(self.tokenizer.err_msg());
            }
        };

        self.eat_whitespace();

        Ok(Node {
            token,
            left: Some(Box::new(prev)),
            right: Some(Box::new(self.term()?)),
        })
    }

    fn eat_whitespace(&mut self) {
        while let Ok(Token::Whitespace(_, _)) = self.tokenizer.peek_token() {
            let _ = self.tokenizer.next_token();
        }
    }

    fn eat_token(&mut self) {
        let _ = self.tokenizer.next_token();
    }

    fn node(&mut self, token: ParseToken) -> Node {
        Node { left: None, right: None, token: token }
    }

    fn close_token(&mut self, ret: Node, token: Token) -> Result<Node> {
        debug!("#close_token");
        match self.tokenizer.next_token() {
            Ok(ref t) if t.partial_eq(token) => {
                Ok(ret)
            }
            _ => {
                Err(self.tokenizer.err_msg())
            }
        }
    }
}

pub trait NodeVisitor {
    fn visit(&mut self, node: Node) {
        match node.token {
            ParseToken::Absolute
            | ParseToken::Relative
            | ParseToken::All
            | ParseToken::Key(_) => {
                self.visit_token(node.token);
            }
            ParseToken::In
            | ParseToken::Leaves => {
                node.left.map(|n| self.visit(*n));
                self.visit_token(node.token);
                node.right.map(|n| self.visit(*n));
            }
            | ParseToken::Range(_, _)
            | ParseToken::Union(_)
            | ParseToken::Number(_) => {
                self.visit_token(node.token);
            }

            | ParseToken::Array => {
                node.left.map(|n| self.visit(*n));
                self.visit_token(node.token);
                node.right.map(|n| self.visit(*n));
                self.visit_token(ParseToken::ArrayEof);
            }
            ParseToken::Filter(FilterToken::And)
            | ParseToken::Filter(FilterToken::Or) => {
                node.left.map(|n| self.visit(*n));
                node.right.map(|n| self.visit(*n));
                self.visit_token(node.token);
            }
            ParseToken::Filter(_) => {
                node.left.map(|n| self.visit(*n));
                self.end_term();
                node.right.map(|n| self.visit(*n));
                self.end_term();
                self.visit_token(node.token);
            }
            _ => {}
        }
    }

    fn visit_token(&mut self, token: ParseToken);
    fn end_term(&mut self) {}
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use super::*;

    struct NodeVisitorTestImpl<'a> {
        input: &'a str,
        stack: Vec<ParseToken>,
    }

    impl<'a> NodeVisitorTestImpl<'a> {
        fn new(input: &'a str) -> Self {
            NodeVisitorTestImpl { input, stack: Vec::new() }
        }

        fn visit(&mut self) -> result::Result<Vec<ParseToken>, String> {
            let tokenizer = PreloadedTokenizer::new(self.input);
            let mut parser = Parser { tokenizer };
            parser.parse(self)?;
            Ok(self.stack.split_off(0))
        }
    }

    impl<'a> NodeVisitor for NodeVisitorTestImpl<'a> {
        fn visit_token(&mut self, token: ParseToken) {
            self.stack.push(token);
        }
    }

    fn setup() {
        let _ = env_logger::try_init();
    }

    fn run(input: &str) -> result::Result<Vec<ParseToken>, String> {
        let mut interpreter = NodeVisitorTestImpl::new(input);
        interpreter.visit()
    }

    #[test]
    fn parse_path() {
        setup();

        assert_eq!(run("$.aa"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::In,
            ParseToken::Key("aa".to_owned())
        ]));

        assert_eq!(run("$.00.a"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::In,
            ParseToken::Key("00".to_owned()),
            ParseToken::In,
            ParseToken::Key("a".to_owned())
        ]));

        assert_eq!(run("$.00.韓창.seok"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::In,
            ParseToken::Key("00".to_owned()),
            ParseToken::In,
            ParseToken::Key("韓창".to_owned()),
            ParseToken::In,
            ParseToken::Key("seok".to_owned())
        ]));

        assert_eq!(run("$.*"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::In,
            ParseToken::All
        ]));

        assert_eq!(run("$..*"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::Leaves,
            ParseToken::All
        ]));

        assert_eq!(run("$..[0]"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::Leaves,
            ParseToken::Array,
            ParseToken::Number(0.0),
            ParseToken::ArrayEof
        ]));

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

        assert_eq!(run("$.book[?(@.isbn)]"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::In,
            ParseToken::Key("book".to_string()),
            ParseToken::Array,
            ParseToken::Relative,
            ParseToken::In,
            ParseToken::Key("isbn".to_string()),
            ParseToken::ArrayEof
        ]));

        //
        // Array도 컨텍스트 In으로 간주 할거라서 중첩되면 하나만
        //
        assert_eq!(run("$.[*]"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::Array,
            ParseToken::All,
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$.a[*]"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::All,
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$.a[*].가"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::All,
            ParseToken::ArrayEof,
            ParseToken::In, ParseToken::Key("가".to_owned())
        ]));

        assert_eq!(run("$.a[0][1]"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::Number(0_f64),
            ParseToken::ArrayEof,
            ParseToken::Array,
            ParseToken::Number(1_f64),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$.a[1,2]"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::Union(vec![1, 2]),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$.a[10:]"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::Range(Some(10), None),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$.a[:11]"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::Range(None, Some(11)),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$.a[-12:13]"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::Range(Some(-12), Some(13)),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$.a[?(1>2)]"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::Number(1_f64), ParseToken::Number(2_f64), ParseToken::Filter(FilterToken::Greater),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$.a[?($.b>3)]"), Ok(vec![
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Array,
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("b".to_owned()), ParseToken::Number(3_f64), ParseToken::Filter(FilterToken::Greater),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$[?($.c>@.d && 1==2)]"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::Array,
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("c".to_owned()),
            ParseToken::Relative, ParseToken::In, ParseToken::Key("d".to_owned()),
            ParseToken::Filter(FilterToken::Greater),
            ParseToken::Number(1_f64), ParseToken::Number(2_f64), ParseToken::Filter(FilterToken::Equal),
            ParseToken::Filter(FilterToken::And),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$[?($.c>@.d&&(1==2||3>=4))]"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::Array,
            ParseToken::Absolute, ParseToken::In, ParseToken::Key("c".to_owned()),
            ParseToken::Relative, ParseToken::In, ParseToken::Key("d".to_owned()),
            ParseToken::Filter(FilterToken::Greater),
            ParseToken::Number(1_f64), ParseToken::Number(2_f64), ParseToken::Filter(FilterToken::Equal),
            ParseToken::Number(3_f64), ParseToken::Number(4_f64), ParseToken::Filter(FilterToken::GreaterOrEqual),
            ParseToken::Filter(FilterToken::Or),
            ParseToken::Filter(FilterToken::And),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$[?(@.a<@.b)]"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::Array,
            ParseToken::Relative, ParseToken::In, ParseToken::Key("a".to_owned()),
            ParseToken::Relative, ParseToken::In, ParseToken::Key("b".to_owned()),
            ParseToken::Filter(FilterToken::Little),
            ParseToken::ArrayEof
        ]));

        assert_eq!(run("$[*][*][*]"), Ok(vec![
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
        ]));

        assert_eq!(run("$['a']['bb']"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::Array,
            ParseToken::Key("a".to_string()),
            ParseToken::ArrayEof,
            ParseToken::Array,
            ParseToken::Key("bb".to_string()),
            ParseToken::ArrayEof
        ]));

        match run("$[") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$[]") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$[a]") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$[?($.a)]") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$[?(@.a > @.b]") {
            Ok(_) => panic!(),
            _ => {}
        }

        match run("$[?(@.a < @.b&&(@.c < @.d)]") {
            Ok(_) => panic!(),
            _ => {}
        }
    }

    #[test]
    fn parse_array_float() {
        setup();

        assert_eq!(run("$[?(1.1<2.1)]"), Ok(vec![
            ParseToken::Absolute,
            ParseToken::Array,
            ParseToken::Number(1.1), ParseToken::Number(2.1), ParseToken::Filter(FilterToken::Little),
            ParseToken::ArrayEof
        ]));

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