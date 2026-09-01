use std::collections::HashMap;
use crate::types::core::SrsValue;

pub struct SrsMemory<'a> {
    memory: HashMap<String, SrsValue>,
    mother: Option<&'a SrsMemory<'a>>,
}

impl<'a> SrsMemory<'a> {
    pub fn new() -> Self {
        SrsMemory {
            memory: HashMap::new(),
            mother: None,
        }
    }

    pub fn child(parent: &'a SrsMemory<'a>) -> Self {
        SrsMemory {
            memory: HashMap::new(),
            mother: Some(parent),
        }
    }

    pub fn get(&self, key: &str) -> Option<&SrsValue> {
        if let Some(value) = self.memory.get(key) {
            Some(value)
        } else if let Some(mother) = self.mother {
            mother.get(key)
        } else {
            None
        }
    }

    pub fn add(&mut self, key: impl Into<String>, value: SrsValue) {
        self.memory.insert(key.into(), value);
    }
}

impl Default for SrsMemory<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_usage() {
        let mut mem = SrsMemory::new();
        mem.add("i", SrsValue::Integer(2));
        let result = mem.get("i");
        assert_eq!(Some(&SrsValue::Integer(2)), result);
    }

    #[test]
    fn test_chain_data_in_child() {
        let mother = SrsMemory::new();
        let mut mem = SrsMemory::child(&mother);
        mem.add("i", SrsValue::Integer(2));
        let result = mem.get("i");
        assert_eq!(Some(&SrsValue::Integer(2)), result);
    }

    #[test]
    fn test_chain_data_in_mother() {
        let mut mother = SrsMemory::new();
        mother.add("i", SrsValue::Integer(2));

        let mem = SrsMemory::child(&mother);
        let result = mem.get("i");
        assert_eq!(Some(&SrsValue::Integer(2)), result);
    }

    #[test]
    fn test_chain_data_value_not_in() {
        let mother = SrsMemory::new();
        let mem = SrsMemory::child(&mother);
        let result = mem.get("i");
        assert!(result.is_none());
    }
}
