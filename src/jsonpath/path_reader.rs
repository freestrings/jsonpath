use std::result;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Eof,
    Position(usize),
}

pub struct PathReader<'a> {
    input: &'a str,
    pos: usize,
    len: usize,
}

impl<'a> PathReader<'a> {
    pub fn new(input: &'a str) -> Self {
        let len = input.chars().by_ref().map(|c| c.len_utf8()).sum();
        PathReader {
            input,
            pos: 0,
            len,
        }
    }

    pub fn peek_char(&mut self) -> result::Result<(usize, char), Error> {
        let ch = self.input.chars().next().ok_or(Error::Eof)?;
        Ok((self.pos, ch))
    }

    pub fn look_next_char(&self) -> result::Result<(usize, char), Error> {
        match self.input.chars().skip(1).next() {
            Some(ch) => Ok((self.pos + 1, ch)),
            _ => Err(Error::Position(self.pos))
        }
    }

    pub fn take_while<F>(&mut self, fun: F) -> result::Result<(usize, Vec<char>), Error>
        where
            F: Fn(&char) -> bool
    {
        let vec: Vec<char> = self.input.chars()
            .by_ref()
            .take_while(fun)
            .collect();

        let char_len: usize = vec.iter().by_ref().map(|c| c.len_utf8()).sum();
        self.pos += char_len;
        debug!("self.pos:{}, char_len:{}, self.len:{}, => {:?}", self.pos, char_len, self.len, vec);

        if vec.is_empty() {
            Ok((self.pos, vec))
        } else if self.len <= self.pos {
            Err(Error::Position(self.len))
        } else {
            self.input = &self.input[char_len..];
            Ok((self.pos, vec))
        }
    }

    pub fn next_char(&mut self) -> result::Result<(usize, char), Error> {
        let (_, ch) = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        let ret = Ok((self.pos, ch));
        self.pos += ch.len_utf8();
        ret
    }
}