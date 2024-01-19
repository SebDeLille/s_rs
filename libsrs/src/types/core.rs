use std::any::Any;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum SrsType { LIST, VECTOR, INTEGER, FLOAT, STRING, ID }

pub trait SrsElement: Display + ToString {
    fn is_list(&self) -> bool;
    fn get_type(&self) -> SrsType;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

}

pub type SrsValue = Option<Rc<dyn SrsElement>>;
pub type SrsValueRef<'a> = Option<&'a Box<dyn SrsElement>>;


#[macro_export]
macro_rules! to_type {
    ($value: ident, $target_type: ident) => {
        $value.as_ref().unwrap().as_any().downcast_ref::<$target_type>()
    };
}