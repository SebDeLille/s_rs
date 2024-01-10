use std::any::Any;
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
}
