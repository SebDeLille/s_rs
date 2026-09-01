use std::fmt::{Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SrsErrorKind {
    Undefined,
    UnknownType,
    UnexpectedEnd,
    UnbalancedParen,
    UnsupportedChar,
    UncompletedString,
    ParseError,
    TypeMismatch,
    NotEnoughArguments,
    UnknownIdentifier,
}

#[derive(Debug)]
pub struct SrsError {
    pub kind: SrsErrorKind,
    pub message: String,
}

pub type SrsResult<T> = Result<T, SrsError>;

fn default_message(kind: SrsErrorKind) -> &'static str {
    match kind {
        SrsErrorKind::UnknownType => "Unknown type",
        SrsErrorKind::Undefined => "Undefined status",
        SrsErrorKind::UnexpectedEnd => "Unexpected end of input",
        SrsErrorKind::UnbalancedParen => "Unbalanced parenthesis",
        SrsErrorKind::UnsupportedChar => "Unsupported character",
        SrsErrorKind::UncompletedString => "Uncompleted string",
        SrsErrorKind::ParseError => "Parse error",
        SrsErrorKind::TypeMismatch => "Type mismatch",
        SrsErrorKind::NotEnoughArguments => "Not enough arguments",
        SrsErrorKind::UnknownIdentifier => "Unknown identifier",
    }
}

impl SrsError {
    pub fn new(kind: SrsErrorKind) -> Self {
        SrsError {
            message: default_message(kind).to_string(),
            kind,
        }
    }

    pub fn with_message(kind: SrsErrorKind, message: impl Into<String>) -> Self {
        SrsError {
            kind,
            message: message.into(),
        }
    }
}

impl From<ParseIntError> for SrsError {
    fn from(_value: ParseIntError) -> Self {
        SrsError::new(SrsErrorKind::ParseError)
    }
}

impl From<ParseFloatError> for SrsError {
    fn from(_value: ParseFloatError) -> Self {
        SrsError::new(SrsErrorKind::ParseError)
    }
}

impl Default for SrsError {
    fn default() -> Self {
        SrsError::new(SrsErrorKind::Undefined)
    }
}

impl Display for SrsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
