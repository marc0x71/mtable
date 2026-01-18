use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq)]
pub enum TableError<T> {
    InvalidString(String),
    InvalidInput(char),
    AmbiguousPattern(char),
    InvalidRange,
    ValueAlreadyDefined { current: T, requested: T },
}

impl<T: Debug> std::fmt::Display for TableError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableError::InvalidInput(ch) => write!(f, "Invalid input character: '{}'", ch),
            TableError::InvalidString(s) => write!(f, "Invalid string (non-ASCII): '{}'", s),
            TableError::InvalidRange => write!(f, "Invalid range: unclosed or empty bracket"),
            TableError::ValueAlreadyDefined { current, requested } => {
                write!(
                    f,
                    "Value already defined: current={:?}, requested={:?}",
                    current, requested
                )
            }
            TableError::AmbiguousPattern(ch) => write!(f, "Ambiguous pattern found: '{ch}'"),
        }
    }
}

impl<T: Debug> std::error::Error for TableError<T> {}

#[derive(Debug, PartialEq)]
pub enum LexerError {
    InvalidString(String),
    UnknownChar { char: char, position: usize },
    UnexpectedEnd { position: usize }, // se ti serve
}

impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::InvalidString(s) => write!(f, "Invalid string (non-ASCII): '{s}'"),
            LexerError::UnknownChar { char, position } => {
                write!(f, "Unknown char '{char}' at position {position}")
            }
            LexerError::UnexpectedEnd { position } => {
                write!(f, "Unexpected end at position {position}")
            }
        }
    }
}
impl std::error::Error for LexerError {}
