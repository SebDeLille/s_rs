use std::any::Any;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::types::core::{SrsElement, SrsType, SrsValue, SrsValueRef};
use crate::types::error::{SrsError, SrsResult};


pub struct SrsList {
    pub car: SrsValue,
    pub cdr: SrsValue,
}

impl Display for SrsList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LIST")
    }
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    pub fn car(&self) -> SrsValue {
        self.car.clone()
    }

    pub fn cdr(&self) -> SrsValue {
        self.cdr.clone()
    }

    pub fn add_tail(&mut self, value: SrsValue) -> SrsResult<()> {
        let mut list = self;
        loop {
            if list.car.is_none() {
                list.car = value.clone();
                return Ok(())
            } else if list.cdr.is_none() {
                list.cdr = Some(Rc::new(SrsList {
                    car: value.clone(),
                    cdr: None
                }));
                return Ok(())
            } else if list.cdr.as_ref().unwrap().is_list() {
                match list.cdr.as_mut().unwrap().as_any_mut().downcast_mut::<SrsList>() {
                    Some(l) => list = l,
                    None => return Err(SrsError::default())
                }
            }
            else {
                return Err(SrsError::default())
            }
        }
    }
}

pub struct Iterator {
    pub current_list: SrsValue,
}

impl Iterator {

    pub fn next(&mut self) -> SrsValue {
        match self.current_list {
            Some(value) => {
                match value.as_any().downcast_ref::<SrsList>() {
                    Some(l) => {
                        let a = l.car();
                        self.current_list = l.cdr();
                        a
                    }
                    None => None
                }
            }
            None => None
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
    fn test_add_tail(){
        let mut node1 = SrsList {
            car: Some(Rc::new(SrsInteger{value: 5})),
            cdr: None
        };

        let node2: SrsValue  = Some(Rc::new (SrsList {
            car: Some(Rc::new(SrsInteger{value: 15})),
            cdr: None
        }));

        match node1.add_tail(node2) {
            Ok(_) => assert!(node1.is_list()),
            Err(_) => panic!()
        }
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
