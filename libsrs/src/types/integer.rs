use std::any::Any;
use crate::types::core::{SrsElement, SrsType};

#[derive(Debug)]
pub struct SrsInteger {
    pub value: i64,
}

impl SrsElement for SrsInteger {
    fn is_list(&self) -> bool {
        false
    }

    fn get_type(&self) -> SrsType {
        SrsType::INTEGER
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

