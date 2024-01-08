use std::collections::HashMap;
use crate::types::core::{SrsElement, SrsValue, SrsValueRef};

pub struct SrsMemory<'a> {
    memory: HashMap<String, Box<dyn SrsElement>>,
    mother: Option<&'a Box<SrsMemory<'a>>>
}

impl<'a> SrsMemory<'a> {
    pub fn new() -> Self {
        SrsMemory {
            memory: HashMap::new(),
            mother: None
        }
    }

    pub fn add_to(&mut self, mem: &'a Box<SrsMemory>)  {
        if self.mother.is_none() {
            self.mother = Some(mem);
        }
    }

    pub fn get(&self, key: &String) -> SrsValueRef {
        let v = self.memory.get(key);
        if v.is_some() {
            v
        } else if self.mother.is_some() {
            self.mother.as_ref().unwrap().get(key)
        } else {
            None
        }
    }

    pub fn add(&mut self, key: String, value: SrsValue) {
        self.memory.insert(key.clone(), value.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use crate::types::core::SrsElement;
    use crate::types::integer::SrsInteger;
    use crate::types::memory::SrsMemory;

    #[test]
    fn test_basic_usage() {
        let mut mem = SrsMemory::new();
        let i: Box<dyn SrsElement> = Box::new(SrsInteger{value: 2});
        mem.add("i".to_string(), Some(i));
        let result = mem.get(&"i".to_string());
        match result.unwrap().as_any().downcast_ref::<SrsInteger>() {
            Some(ri) => assert_eq!(2, ri.value),
            None => panic!("incorrect type")
        }
    }

    #[test]
    fn test_chain_data_in_child() {
        let mother = SrsMemory::new();
        let binding = Box::new(mother);
        let mut mem = SrsMemory::new();
        mem.add_to(&binding);

        let i: Box<dyn SrsElement> = Box::new(SrsInteger{value: 2});
        mem.add("i".to_string(), Some(i));
        let result = mem.get(&"i".to_string());
        match result.unwrap().as_any().downcast_ref::<SrsInteger>() {
            Some(ri) => assert_eq!(2, ri.value),
            None => panic!("incorrect type")
        }
    }

    #[test]
    fn test_chain_data_in_mother() {
        let mother = SrsMemory::new();
        let mut binding = Box::new(mother);
        let i: Box<dyn SrsElement> = Box::new(SrsInteger{value: 2});
        binding.add("i".to_string(), Some(i));


        let mut mem = SrsMemory::new();
        mem.add_to(&binding);

        let result = mem.get(&"i".to_string());
        match result.unwrap().as_any().downcast_ref::<SrsInteger>() {
            Some(ri) => assert_eq!(2, ri.value),
            None => panic!("incorrect type")
        }
    }

    #[test]
    fn test_chain_data_value_not_in() {
        let mother = SrsMemory::new();
        let binding = Box::new(mother);
        let mut mem = SrsMemory::new();
        mem.add_to(&binding);

        let result = mem.get(&"i".to_string());
        assert!(result.is_none());
    }
}