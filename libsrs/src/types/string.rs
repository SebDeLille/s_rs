use std::any::Any;
use std::fmt::{Display, Formatter};
use crate::types::core::{SrsElement, SrsType};

pub struct SrsString {
    pub value: String,
}

impl SrsString {
    pub fn new(value: String) -> Self {
        SrsString {
            value,
        }
    }
}

impl Display for SrsString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl SrsElement for SrsString {
    fn is_list(&self) -> bool {
        false
    }

    fn get_type(&self) -> SrsType {
        SrsType::STRING
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
