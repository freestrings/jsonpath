use std::result::Result;

#[derive(Debug, PartialEq)]
pub enum ReaderError {
    Eof,
}

pub struct PathReader<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> PathReader<'a> {
    pub fn new(input: &'a str) -> Self {
        PathReader { input, pos: 0 }
    }

    pub fn peek_char(&self) -> Result<(usize, char), ReaderError> {
        let ch = self.input.chars().next().ok_or(ReaderError::Eof)?;
        Ok((self.pos + ch.len_utf8(), ch))
    }

    pub fn take_while<F>(
        &mut self,
        fun: F,
    ) -> Result<(usize, String), ReaderError>
    where
        F: Fn(&char) -> bool,
    {
        let mut char_len: usize = 0;
        let mut ret = String::new();
        for c in self.input.chars().by_ref() {
            if !fun(&c) {
                break;
            }
            char_len += c.len_utf8();
            ret.push(c);
        }

        self.pos += char_len;
        self.input = &self.input[char_len..];
        Ok((self.pos, ret))
    }

    pub fn next_char(&mut self) -> Result<(usize, char), ReaderError> {
        let (_, ch) = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        let ret = Ok((self.pos, ch));
        self.pos += ch.len_utf8();
        ret
    }

    pub fn current_pos(&self) -> usize {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let reader = PathReader::new("abc");
        assert_eq!(reader.input, "abc");
        assert_eq!(reader.pos, 0);
    }

    #[test]
    fn test_peek_char() {
        let reader = PathReader::new("abc");
        assert_eq!(reader.peek_char(), Ok((1, 'a')));
        let empty_reader = PathReader::new("");
        assert_eq!(empty_reader.peek_char(), Err(ReaderError::Eof));
    }

    #[test]
    fn test_take_while() {
        let mut reader = PathReader::new("abc");
        assert_eq!(reader.take_while(|c| *c != 'c'), Ok((2, "ab".to_string())));
        assert_eq!(reader.take_while(|c| *c != 'c'), Ok((2, "".to_string()))); // already at 'c'
        let mut empty_reader = PathReader::new("");
        assert_eq!(empty_reader.take_while(|_| true), Ok((0, "".to_string())));
        let mut reader = PathReader::new("abc");
        assert_eq!(reader.take_while(|_| false), Ok((0, "".to_string())));
    }

    #[test]
    fn test_next_char() {
        let mut reader = PathReader::new("abc");
        assert_eq!(reader.next_char(), Ok((0, 'a')));
        assert_eq!(reader.next_char(), Ok((1, 'b')));
        assert_eq!(reader.next_char(), Ok((2, 'c')));
        assert_eq!(reader.next_char(), Err(ReaderError::Eof));
    }

    #[test]
    fn test_current_pos() {
        let mut reader = PathReader::new("abc");
        assert_eq!(reader.current_pos(), 0);
        reader.next_char().unwrap();
        assert_eq!(reader.current_pos(), 1);
    }
}
