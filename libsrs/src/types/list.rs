use std::any::Any;
use std::fmt::{Display, Formatter};
use std::iter::Iterator;
use std::rc::Rc;
use crate::types::core::{SrsElement, SrsType, SrsValue, SrsValueRef};
use crate::types::error::{SrsError, SrsErrorKind, SrsResult};


pub struct SrsList {
    pub value: Vec<Rc<dyn SrsElement>>,
}

impl Display for SrsList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LIST")
    }
}

impl SrsElement for SrsList {
    fn is_list(&self) -> bool {
        true
    }

    fn get_type(&self) -> SrsType {
        SrsType::LIST
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for SrsList {
    fn default() -> Self {
        SrsList {
            value: Vec::new()
        }
    }
}

impl SrsList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn car(&self) -> SrsValue {
        if self.value.is_empty() {
            None
        } else {
            Some(self.value[0].clone())
        }
    }

    pub fn cdr(&self) -> Option<Vec<Rc<dyn SrsElement>>> {
        if self.value.len() < 2 {
            None
        } else {
            Some(self.value[1..]
                .iter()
                .map(|x| x.clone())
                .collect())
        }
    }

    pub fn add_tail(&mut self, value: SrsValue) -> SrsResult<()> {
        match value {
            Some(v) => {
                self.value.push(v);
                Ok(())
            }
            None => Err(SrsError::new(SrsErrorKind::Undefined))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::types::core::{SrsElement, SrsValue};
    use crate::types::integer::SrsInteger;
    use crate::types::list::SrsList;
    use crate::types::list::Iterator;

    #[test]
    fn test_add_tail() {
        let mut list = SrsList::default();
        let i1: SrsValue = Some(Rc::new(SrsInteger::from(5)));
        list.add_tail(i1).unwrap();

        assert_eq!(5, list.value[0].as_any().downcast_ref::<SrsInteger>().unwrap().value);
    }

    #[test]
    fn test_add_tail_iterator() {
        // let mut list = SrsList::default();
        // let i1: SrsValue = Some(Rc::new(SrsInteger::from(5)));
        // let i2: SrsValue = Some(Rc::new(SrsInteger::from(17)));
        //
        // if list.add_tail(i1).is_err() {
        //     panic!("Unable to add element");
        // }
        //
        // if list.add_tail(i2).is_err() {
        //     panic!("Unable to add element");
        // }
        //
        // let tmp: SrsValue = Some(Rc::new(list));
        // let mut it = Iterator {current_list: tmp};
        // let mut results = Vec::new();
        // while let Some(r) = it.next() {
        //     results.push(r.unwrap().as_any().downcast_ref::<SrsInteger>().unwrap().value);
        // }
        // assert_eq!(17, results.pop().unwrap());
        // assert_eq!(5, results.pop().unwrap());
    }
}
