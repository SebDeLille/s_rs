use std::num::ParseIntError;

#[derive(Debug)]
pub struct SrsError;

impl From<ParseIntError> for SrsError {
    fn from(_value: ParseIntError) -> Self {
        SrsError{}
    }
}

pub type SrsResult<T> = Result<T, SrsError>;