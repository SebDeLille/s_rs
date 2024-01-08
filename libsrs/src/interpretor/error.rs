use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct SrsError {
    msg: String,
}

impl SrsError {
    pub fn new(s: String) -> Self {
        SrsError {
            msg: s,
        }
    }

    pub fn get_message(&self) -> &String {
        &self.msg
    }
}

impl Display for SrsError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for SrsError {
    fn description(&self) -> &str {
        todo!()
    }

    fn cause(&self) -> Option<&dyn Error> {
        todo!()
    }
}