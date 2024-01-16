use std::any::Any;
use std::fmt::{Display, Formatter};
use crate::types::core::{SrsElement, SrsType};

pub struct SrsId {
    pub value: String,
}

impl SrsId {
    pub fn new(value: String) -> Self {
        SrsId {
            value,
        }
    }
}

impl Display for SrsId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl SrsElement for SrsId {
    fn is_list(&self) -> bool {
        false
    }

    fn get_type(&self) -> SrsType {
        SrsType::ID
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
