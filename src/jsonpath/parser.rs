use std::result;

use super::path_reader::Error;

use super::tokenizer::{
    self,
    Token,
    Tokenizer,
};
use super::utils;

const ABSOLUTE: &'static str = "Absolute";
const RELATIVE: &'static str = "Relative";
const RELATIVES: &'static str = "Relatives";
const RELATIVE_VALUES: &'static str = "RelativeValues";
const ALL_VALUES: &'static str = "AllValues";
const ARRAY: &'static str = "Array";
const FILTER_GROUP: &'static str = "FilterGroup";
const KEY: &'static str = "Key";
const TOKEN: &'static str = "Token";

// TODO String -> Error type
type Result<T> = result::Result<T, String>;

#[derive(Debug, Clone, PartialEq)]
enum Context {
    Absolute,
    Relative,
    Relatives,
    RelativeValues,
    AllValues,
    Array(ArrayValue),
    FilterGroup(Vec<Filter>),
    Key(String),
    Token(Token),
}

impl Context {
    pub fn alias_of(&self, type_str: &'static str) -> bool {
        self.to_alias() == type_str
    }

    fn to_alias(&self) -> &'static str {
        match self {
            Context::Absolute => ABSOLUTE,
            Context::Relative => RELATIVE,
            Context::Relatives => RELATIVES,
            Context::RelativeValues => RELATIVE_VALUES,
            Context::AllValues => ALL_VALUES,
            Context::Array(_) => ARRAY,
            Context::FilterGroup(_) => FILTER_GROUP,
            Context::Key(_) => KEY,
            Context::Token(_) => TOKEN
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ArrayValue {
    UnionString(Vec<String>),
    UnionNum(Vec<isize>),
    Split(Option<isize>, Option<isize>),
}

#[derive(Debug, Clone, PartialEq)]
enum FilterOp {
    Equal,
    GraterOrEqual,
    Grater,
    Little,
    LittleOrEqual,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq)]
enum FilterPath {
    Relative,
    //Relatives,
    Key(String),
}

#[derive(Debug, Clone, PartialEq)]
enum FilterContext {
    Path(Vec<FilterPath>),
    NumValue(isize),
    StrValue(String),
    Op(FilterOp),
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
enum FilterValue {
    Path(Vec<FilterPath>),
    NumValue(isize),
    StrValue(String),
}

#[derive(Debug, Clone, PartialEq)]
enum Filter {
    And(FilterValue, FilterOp, FilterValue),
    Or(FilterValue, FilterOp, FilterValue),
}

fn is_token_match<F>(v: Option<&Context>, fun: F) -> bool
    where F: Fn(&Token) -> bool
{
    match v {
        Some(Context::Token(t)) => fun(t),
        _ => false
    }
}

fn is_match<F>(v: Option<&Context>, fun: F) -> bool
    where F: Fn(&Context) -> bool
{
    match v {
        Some(t) => fun(t),
        _ => false
    }
}

fn take_while<F>(stack: &mut Vec<Context>, fun: F) -> Result<Vec<Context>>
    where F: Fn(&Context) -> Result<bool>
{
    let mut ret: Vec<Context> = Vec::new();
    loop {
        if if let Some(ctx) = stack.last() { !fun(&ctx)? } else { true } {
            break;
        }
        ret.push(stack.pop().unwrap());
    }
    Ok(ret)
}

fn parse_close_array(stack: &mut Vec<Context>) -> Result<()> {
    fn parse_union(stack: &mut Vec<Context>, peek_value: &Context) -> Result<()> {
        let mut comm_count = 0;
        let mut quota_count = match peek_value {
            Context::Token(Token::DoubleQuoted(_, _))
            | Context::Token(Token::SingleQuoted(_, _)) => 1,
            _ => 0
        };

        let mut values: Vec<Vec<char>> = match &peek_value {
            Context::Token(Token::Key(_, vec))
            | Context::Token(Token::SingleQuoted(_, vec))
            | Context::Token(Token::DoubleQuoted(_, vec)) => vec![vec.to_vec()],
            _ => Vec::new()
        };

        while let Some(Context::Token(token)) = stack.pop() {
            match &token {
                Token::OpenArray(_) => break,
                Token::Comma(_) => comm_count += 1,
                Token::Key(_, vec) => values.push(vec.to_vec()),
                Token::DoubleQuoted(_, vec)
                | Token::SingleQuoted(_, vec) => {
                    values.push(vec.to_vec());
                    quota_count += 1
                }
                _ => {
                    return Err(format!("parse_union: {:?}", token));
                }
            }
        };

        if quota_count > 0 {
            let v: Vec<String> = values.iter()
                .rev()
                .map(|vec| utils::vec_to_string(vec))
                .collect();

            return if comm_count == v.len() - 1 {
                stack.push(Context::Array(ArrayValue::UnionString(v)));
                Ok(())
            } else {
                Err(format!("parse_union string: invalid syntax. comma: {}, values: {}", comm_count, v.len()))
            };
        } else {
            let mut v: Vec<isize> = vec![];
            for value in values.iter().rev() {
                v.push(utils::vec_to_number(&value)?);
            }
            return if comm_count == v.len() - 1 {
                stack.push(Context::Array(ArrayValue::UnionNum(v)));
                Ok(())
            } else {
                Err(format!("parse_union number: invalid syntax. comma: {}, values: {}", comm_count, v.len()))
            };
        }
    }

    fn parse_split(stack: &mut Vec<Context>, peek_value: isize) -> Result<()> {
        stack.pop();
        let split_ctx = match stack.last() {
            Some(Context::Token(Token::Key(_, _))) => {
                if let Context::Token(Token::Key(_, vec)) = stack.pop().unwrap() {
                    ArrayValue::Split(Some(utils::vec_to_number(&vec)?), Some(peek_value))
                } else {
                    return Err(format!("parse_split: invalid syntax"));
                }
            }
            _ => ArrayValue::Split(None, Some(peek_value))
        };

        match stack.pop() {
            Some(Context::Token(Token::OpenArray(_))) => {
                stack.push(Context::Array(split_ctx));
                Ok(())
            }
            other => return Err(format!("parse_split: invalid syntax. {:?}", other))
        }
    }

    fn parse_split_from(stack: &mut Vec<Context>) -> Result<()> {
        let split_ctx = match stack.pop() {
            Some(Context::Token(Token::Key(_, vec))) => {
                ArrayValue::Split(Some(utils::vec_to_number(&vec)?), None)
            }
            _ => return Err(format!("parse_split_from: invalid syntax"))
        };

        if match stack.last() {
            Some(Context::Token(Token::OpenArray(_))) => true,
            _ => false
        } {
            stack.push(Context::Array(split_ctx));
            Ok(())
        } else {
            Err(format!("parse_split_from: invalid syntax. {:?}", stack.last()))
        }
    }

    fn parse_filter_group(stack: &mut Vec<Context>, peek_value: Context) -> Result<()> {
        match stack.pop() {
            Some(Context::Token(Token::OpenArray(_))) => {
                stack.push(peek_value);
                Ok(())
            }
            other => return Err(format!("parse_filter_group: invalid syntax: {:?}", other))
        }
    }

    while let Some(ctx) = stack.pop() {
        info!("\t{:?}", ctx);

        match ctx {
            Context::Token(Token::OpenArray(_)) => break,
            Context::Token(Token::DoubleQuoted(_, _))
            | Context::Token(Token::SingleQuoted(_, _))
            | Context::Token(Token::Key(_, _))
            /**/if is_token_match(stack.last(), |t| t.alias_of(tokenizer::COMMA)) => {
                parse_union(stack, &ctx)?;
                break;
            }
            Context::Token(Token::Key(_, ref vec))
            /**/if is_token_match(stack.last(), |t| t.alias_of(tokenizer::SPLIT)) => {
                parse_split(stack, utils::vec_to_number(&vec)?)?;
                break;
            }
            Context::Token(Token::Split(_))
            /**/if is_token_match(stack.last(), |t| t.alias_of(tokenizer::KEY)) => {
                parse_split_from(stack)?;
                break;
            }
            Context::FilterGroup(_) => {
                parse_filter_group(stack, ctx)?;
                break;
            }
            _ => return Err(format!("parse_close_array: invalid syntax. {:?}", ctx))
        }
    }
    Ok(())
}

fn parse_close_paren(stack: &mut Vec<Context>) -> Result<()> {
    fn parse_filter_value(c: Option<FilterContext>) -> Result<FilterValue> {
        match c { // l_value
            Some(FilterContext::Path(vec)) => Ok(FilterValue::Path(vec.clone())),
            Some(FilterContext::NumValue(v)) => Ok(FilterValue::NumValue(v)),
            Some(FilterContext::StrValue(v)) => Ok(FilterValue::StrValue(v)),
            _ => return Err(format!("parse_close_paren: invalid syntax. {:?}", c))
        }
    }

    fn parse_filter_op(c: Option<FilterContext>) -> Result<FilterOp> {
        match c { // op
            Some(FilterContext::Op(op)) => Ok(op.clone()),
            _ => return Err(format!("parse_filter_op: invalid syntax. {:?}", c))
        }
    }

    fn parse_filter(stack: &mut Vec<Context>, local_stack: &mut Vec<FilterContext>) -> Result<()> {
        stack.pop(); // Token::Question

        let mut filters: Vec<Filter> = Vec::new();
        loop {
            if let Some(v) = local_stack.pop() {
                filters.push(match v {
                    FilterContext::And => {
                        Filter::And(parse_filter_value(local_stack.pop())?,
                                    parse_filter_op(local_stack.pop())?,
                                    parse_filter_value(local_stack.pop())?)
                    }
                    FilterContext::Or => {
                        Filter::Or(parse_filter_value(local_stack.pop())?,
                                   parse_filter_op(local_stack.pop())?,
                                   parse_filter_value(local_stack.pop())?)
                    }
                    _ => {
                        Filter::And(parse_filter_value(Some(v))?,
                                    parse_filter_op(local_stack.pop())?,
                                    parse_filter_value(local_stack.pop())?)
                    }
                });
            } else {
                break;
            }
        }

        if filters.is_empty() {
            Err("parse_filter: invalid syntax.".to_owned())
        } else {
            stack.push(Context::FilterGroup(filters));
            Ok(())
        }
    }

    fn parse_path(stack: &mut Vec<Context>, local_stack: &mut Vec<FilterContext>, value: String) -> Result<()> {
        let mut filter_pathes: Vec<FilterPath> = take_while(
            stack,
            |ctx| match ctx {
                Context::Token(Token::At(_)) => Ok(false),
                Context::Relative | Context::Token(Token::Key(_, _)) => Ok(true),
                _ => Err(format!("parse_path: invalid syntax. {:?}", ctx))
            },
        )?.iter()
            .map(|ctx| match ctx {
                Context::Token(Token::Key(_, ref vec)) => FilterPath::Key(utils::vec_to_string(&vec)),
                _ => FilterPath::Relative
            })
            .collect();

        if filter_pathes.is_empty() {
            return Err(format!("parse_path: invalid syntax. empty filter."));
        }

        stack.pop(); // Token::At

        filter_pathes.push(FilterPath::Key(value));
        local_stack.push(FilterContext::Path(filter_pathes));

        Ok(())
    }

    let mut local_stack: Vec<FilterContext> = Vec::new();

    while let Some(ctx) = stack.pop() {
        info!("\t - {:?}", ctx);

        match ctx {
            Context::Token(Token::OpenParenthesis(_))
            /**/if is_token_match(stack.last(), |t| t.alias_of(tokenizer::QUESTION)) => {
                parse_filter(stack, &mut local_stack)?;
                break;
            }

            Context::Token(Token::OpenParenthesis(_)) => unimplemented!(),

            Context::Token(Token::DoubleQuoted(_, ref vec))
            | Context::Token(Token::SingleQuoted(_, ref vec)) => {
                local_stack.push(FilterContext::StrValue(utils::vec_to_string(&vec)));
            }

            Context::Token(Token::Key(_, ref vec))
            /**/if is_match(stack.last(), |c| c.alias_of(RELATIVE) || c.alias_of(RELATIVES)) => {
                parse_path(stack, &mut local_stack, utils::vec_to_string(vec))?;
            }

            Context::Key(ref value)
            /**/if is_match(stack.last(), |c| c.alias_of(RELATIVE) || c.alias_of(RELATIVES)) => {
                parse_path(stack, &mut local_stack, value.clone())?;
            }

            Context::Token(Token::Key(_, ref vec)) => local_stack.push(FilterContext::NumValue(utils::vec_to_number(&vec)?)),

            Context::Token(Token::And(_)) => local_stack.push(FilterContext::And),
            Context::Token(Token::Or(_)) => local_stack.push(FilterContext::Or),

            Context::Token(Token::Equal(_)) => local_stack.push(FilterContext::Op(FilterOp::Equal)),
            Context::Token(Token::NotEqual(_)) => local_stack.push(FilterContext::Op(FilterOp::NotEqual)),
            Context::Token(Token::Little(_)) => local_stack.push(FilterContext::Op(FilterOp::Little)),
            Context::Token(Token::LittleOrEqual(_)) => local_stack.push(FilterContext::Op(FilterOp::LittleOrEqual)),
            Context::Token(Token::Greater(_)) => local_stack.push(FilterContext::Op(FilterOp::Grater)),
            Context::Token(Token::GreaterOrEqual(_)) => local_stack.push(FilterContext::Op(FilterOp::GraterOrEqual)),

            _ => unreachable!("{:?} %{:?}", ctx, stack)
        }
    }

    Ok(())
}

fn parse_path_key(stack: &mut Vec<Context>, token: Token) -> Result<()> {
    if let Some(Context::Token(Token::Key(_, ref vec))) = stack.pop() {
        stack.push(Context::Key(utils::vec_to_string(vec)));
    } else {
        return Err(format!("parse_path_key: unreachable"));
    }

    stack.push(Context::Token(token));
    Ok(())
}

struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    stack: Vec<Context>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(input),
            stack: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<Vec<Context>> {
        let iter = self.tokenizer.by_ref().into_iter();

        while let Some(token) = iter.next() {
            info!("{:?}, %{:?}", token, self.stack);

            match token {
                Token::Absolute(_) => self.stack.push(Context::Absolute),

                Token::Relative(_)
                /**/if is_match(self.stack.last(), |c| c.alias_of(RELATIVE)) => {
                    self.stack.pop();
                    self.stack.push(Context::Relatives)
                }

                Token::Relative(_) => self.stack.push(Context::Relative),

                Token::Asterisk(_)
                /**/if is_match(self.stack.last(), |c| c.alias_of(RELATIVE)) => {
                    self.stack.pop();
                    self.stack.push(Context::RelativeValues);
                }

                Token::Asterisk(_)
                /**/if is_match(self.stack.last(), |c| c.alias_of(RELATIVES)) => {
                    self.stack.pop();
                    self.stack.push(Context::AllValues);
                }

                Token::Asterisk(_) => self.stack.push(Context::Token(token)),

                Token::OpenArray(_)
                /**/ if is_token_match(self.stack.last(), |t| t.alias_of(tokenizer::KEY)) => {
                    parse_path_key(&mut self.stack, token)?
                }

                Token::OpenArray(_)
                | Token::OpenParenthesis(_)
                | Token::Question(_)
                | Token::Comma(_)
                | Token::At(_)
                | Token::Split(_)
                | Token::Key(_, _)
                | Token::DoubleQuoted(_, _)
                | Token::SingleQuoted(_, _)
                | Token::And(_)
                | Token::Or(_)
                | Token::Little(_)
                | Token::LittleOrEqual(_)
                | Token::Greater(_)
                | Token::GreaterOrEqual(_)
                | Token::Equal(_)
                | Token::NotEqual(_) => self.stack.push(Context::Token(token)),

                Token::CloseParenthesis(_) => parse_close_paren(&mut self.stack)?,

                Token::CloseArray(_) => parse_close_array(&mut self.stack)?,

                _ => {}
            }
        }

        match iter.get_error() {
            Some(Error::Position(p)) => Err(format!("Parse error. position:'{}'", p)),
            _ => Ok(self.stack.to_vec())
        }
    }


    fn token_position(&self, token: &Token) -> usize {
        match token {
            Token::Absolute(pos) => *pos,
            Token::Relative(pos) => *pos,
            Token::At(pos) => *pos,
            Token::OpenArray(pos) => *pos,
            Token::CloseArray(pos) => *pos,
            Token::Asterisk(pos) => *pos,
            Token::Question(pos) => *pos,
            Token::Comma(pos) => *pos,
            Token::Split(pos) => *pos,
            Token::OpenParenthesis(pos) => *pos,
            Token::CloseParenthesis(pos) => *pos,
            Token::Key(pos, _) => *pos,
            Token::DoubleQuoted(pos, _) => *pos,
            Token::SingleQuoted(pos, _) => *pos,
            Token::Equal(pos) => *pos,
            Token::GreaterOrEqual(pos) => *pos,
            Token::Greater(pos) => *pos,
            Token::Little(pos) => *pos,
            Token::LittleOrEqual(pos) => *pos,
            Token::NotEqual(pos) => *pos,
            Token::And(pos) => *pos,
            Token::Or(pos) => *pos,
            Token::Float(pos, _) => *pos,
            Token::Whitespace(pos) => *pos
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use super::*;

    fn run(input: &str, expected: Vec<Context>, err: Option<String>) {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(vec) => assert_eq!(vec, expected, "\"{}\"", input),
            Err(e) => if err.is_some() {
                assert_eq!(e, err.unwrap());
            } else {
                panic!("Error: {:?}", e);
            }
        }
    }

    #[test]
    fn filter() {
        env_logger::init();

        run("$..book[?(@.price<10)]", vec![
            Context::Absolute,
            Context::Relatives,
            Context::Key("book".to_owned()),
            Context::FilterGroup(vec![
                Filter::And(
                    FilterValue::Path(vec![
                        FilterPath::Relative,
                        FilterPath::Key("price".to_owned()),
                    ]),
                    FilterOp::Little, FilterValue::NumValue(10),
                ),
            ])
        ], None);

        run("$..book[?(@.price<)]",
            vec![],
            Some(format!("Parse error. position:'{}'", 17)));

        run("$..book[?(@.price<10 && @.name == \"한\" || @.total > 100 )]", vec![
            Context::Absolute,
            Context::Relatives,
            Context::Key("book".to_owned()),
            Context::FilterGroup(vec![
                Filter::And(
                    FilterValue::Path(vec![
                        FilterPath::Relative,
                        FilterPath::Key("price".to_owned()),
                    ]),
                    FilterOp::Little,
                    FilterValue::NumValue(10),
                ),
                Filter::And(
                    FilterValue::Path(vec![
                        FilterPath::Relative,
                        FilterPath::Key("name".to_owned()),
                    ]),
                    FilterOp::Equal,
                    FilterValue::StrValue("한".to_owned()),
                ),
                Filter::Or(
                    FilterValue::Path(vec![
                        FilterPath::Relative,
                        FilterPath::Key("total".to_owned()),
                    ]),
                    FilterOp::Grater,
                    FilterValue::NumValue(100),
                ),
            ])
        ], None);
    }
}