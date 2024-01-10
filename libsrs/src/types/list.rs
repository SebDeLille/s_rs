use std::any::Any;
use crate::types::core::{SrsElement, SrsType, SrsValue, SrsValueRef};
use crate::types::error::SrsError;

pub struct SrsList {
    pub car: SrsValue,
    pub cdr: SrsValue,
}

impl SrsElement for SrsList {
    fn is_list(&self) -> bool {
        if self.cdr.is_none() {
            true
        } else {
            self.cdr.as_ref().unwrap().is_list()
        }
    }

    fn get_type(&self) -> SrsType {
        SrsType::LIST
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for SrsList {
    fn default() -> Self {
        SrsList {
            car: None,
            cdr: None
        }
    }
}
impl SrsList {
    pub fn new(car: SrsValue, cdr: SrsValue) -> Self {
        SrsList {
            car,
            cdr,
        }
    }

    pub fn car(&self) -> SrsValueRef {
        self.car.as_ref()
    }

    pub fn cdr(&self) -> SrsValueRef {
        self.cdr.as_ref()
    }
}

pub struct Iterator<'a> {
    pub current_list: Option<&'a Box<dyn SrsElement>>,
}

impl Iterator<'_> {
    pub fn has_next(&self) -> bool {
        match self.current_list {
            Some(v) => {
                if self.current_list.unwrap().is_list() {
                    let t = v.as_any();
                    match t.downcast_ref::<SrsList>() {
                        Some(list) => list.car.is_some(),
                        None => false
                    }
                } else {
                    true
                }
            }
            None => false
        }
    }

    pub fn next(&mut self) -> Result<SrsValueRef, SrsError> {
        if self.current_list.is_none() {
            return Ok(None);
        }

        if self.current_list.unwrap().is_list() {
            match self.current_list.unwrap().as_any().downcast_ref::<SrsList>() {
                Some(l) => {
                    let a = l.car();
                    self.current_list = l.cdr();
                    Ok(a)
                }
                None => Err(crate::types::error::SrsError {})
            }
        } else {
            let a = self.current_list;
            self.current_list = None;
            Ok(a)
        }
    }
}
