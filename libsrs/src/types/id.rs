use std::any::Any;
use crate::types::core::{SrsElement, SrsType};

pub struct SrsId {
    pub value: String,
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
}
