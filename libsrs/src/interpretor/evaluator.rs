use crate::types::core::{SrsElement, SrsType, SrsValue, SrsValueRef};
use crate::types::error::SrsError;
use crate::types::id::SrsId;
use crate::types::integer::SrsInteger;
use crate::types::list::SrsList;

pub struct Evaluator {}

impl Evaluator {

    pub fn eval(&self, element: Option<&Box<dyn SrsElement>>) -> Result<SrsValue, SrsError> {
        match element.unwrap().get_type() {
            SrsType::LIST => self.eval_list(element),
            SrsType::INTEGER => self.eval_integer(element),
            SrsType::ID => self.eval_id(element),
            _ => Err(SrsError{})
        }
    }

    fn eval_list(&self, list: SrsValueRef) -> Result<SrsValue, SrsError> {
        match list.unwrap().as_any().downcast_ref::<SrsList>() {
            Some(l) => Ok(None),
            None => Err(SrsError{})
        }

    }

    fn eval_integer(&self, i: SrsValueRef) -> Result<SrsValue, SrsError> {
        match i.unwrap().as_any().downcast_ref::<SrsInteger>() {
            Some(int) => Ok(Some(Box::new(SrsInteger{value: int.value}))),
            None => Err(SrsError{})
        }
    }

    fn eval_id(&self, id: SrsValueRef) -> Result<SrsValue, SrsError> {
        match id.unwrap().as_any().downcast_ref::<SrsId>() {
            Some(local_id) => Ok(None),
            None => Err(SrsError{})
        }
    }
}