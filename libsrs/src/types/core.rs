use std::any::Any;
use std::fmt::Display;

#[derive(Debug)]
pub enum SrsType { LIST, VECTOR, INTEGER, FLOAT, STRING, ID }


pub trait SrsElement: Display {
    fn is_list(&self) -> bool;
    fn get_type(&self) -> SrsType;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type SrsValue = Option<Box<dyn SrsElement>>;
pub type SrsValueRef<'a> = Option<&'a Box<dyn SrsElement>>;
