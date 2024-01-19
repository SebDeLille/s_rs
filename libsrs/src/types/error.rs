use std::fmt::{Display, Formatter};
use std::num::ParseIntError;

#[derive(Debug)]
pub enum SrsErrorKind {
    Undefined,
    UnknownType,
    DowncastFail
}
#[derive(Debug)]
pub struct SrsError {
    pub kind: SrsErrorKind
}

pub type SrsResult<T> = Result<T, SrsError>;

impl From<ParseIntError> for SrsError {
    fn from(_value: ParseIntError) -> Self {
        SrsError{kind: SrsErrorKind::Undefined}
    }
}

impl Default for SrsError {
    fn default() -> Self {
        SrsError{kind: SrsErrorKind::Undefined}
    }
}

impl Display for SrsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg: &str = match self.kind {
            SrsErrorKind::UnknownType => "Unknown type",
            SrsErrorKind::DowncastFail => "Unable to downcast SrsElement to target",
            SrsErrorKind::Undefined => "Undefined status"
        };
        write!(f, "{}", msg)
    }
}
