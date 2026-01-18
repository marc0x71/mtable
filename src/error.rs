use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub enum TableError<T> {
    InvalidString(String),
    InvalidInput(char),
    LocationOccupied(usize),
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
            TableError::LocationOccupied(pos) => write!(f, "Location already occupied: {pos}"),
        }
    }
}

impl<T: Debug> std::error::Error for TableError<T> {}
