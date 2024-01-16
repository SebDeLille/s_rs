use std::any::Any;
use std::fmt::{Display, Formatter};
use crate::types::core::{SrsElement, SrsType};

#[derive(Debug)]
pub struct SrsInteger {
    pub value: i64,
}

impl Display for SrsInteger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl From<i64> for SrsInteger {
    fn from(value: i64) -> Self {
        SrsInteger {value}
    }
}
impl SrsInteger {
    pub fn new(i: i64) -> Self {
        SrsInteger {
            value: i
        }
    }
}

