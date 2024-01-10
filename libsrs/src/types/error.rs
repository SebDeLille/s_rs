use std::num::ParseIntError;

#[derive(Debug)]
pub struct SrsError;

impl From<ParseIntError> for SrsError {
    fn from(value: ParseIntError) -> Self {
        SrsError{}
    }
}