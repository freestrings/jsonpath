use std::fmt;

#[derive(Debug)]
pub struct SerdeError {
    msg: String,
}

impl<'a> SerdeError {
    pub fn new(msg: String) -> Self {
        SerdeError { msg: msg }
    }

    pub fn from_str(msg: &str) -> Self {
        SerdeError { msg: msg.to_string() }
    }
}

impl serde::de::Error for SerdeError {
    #[cold]
    fn custom<T: fmt::Display>(msg: T) -> SerdeError {
        SerdeError { msg: msg.to_string() }
    }
}

impl serde::ser::Error for SerdeError {
    #[cold]
    fn custom<T: fmt::Display>(msg: T) -> SerdeError {
        SerdeError { msg: msg.to_string() }
    }
}

impl std::error::Error for SerdeError {}

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}