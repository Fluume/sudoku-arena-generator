use std::fmt;

/// Error returned when parsing a board fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input did not contain exactly 81 characters.
    WrongLength { expected: usize, actual: usize },
    /// A character was not a digit 1-9 nor a recognized blank marker (`0` or `.`).
    InvalidChar { index: usize, found: char },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::WrongLength { expected, actual } => {
                write!(f, "expected {expected} characters, found {actual}")
            }
            ParseError::InvalidChar { index, found } => {
                write!(f, "invalid character '{found}' at position {index}")
            }
        }
    }
}

impl std::error::Error for ParseError {}
