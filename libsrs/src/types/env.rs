use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::types::core::SrsValue;

pub struct Env {
    vars: RefCell<HashMap<String, SrsValue>>,
    parent: Option<Rc<Env>>,
}

pub type EnvRef = Rc<Env>;

impl Env {
    pub fn root() -> EnvRef {
        Rc::new(Env {
            vars: RefCell::new(HashMap::new()),
            parent: None,
        })
    }

    pub fn child(parent: &EnvRef) -> EnvRef {
        Rc::new(Env {
            vars: RefCell::new(HashMap::new()),
            parent: Some(Rc::clone(parent)),
        })
    }

    pub fn get(&self, key: &str) -> Option<SrsValue> {
        if let Some(value) = self.vars.borrow().get(key).cloned() {
            Some(value)
        } else if let Some(parent) = &self.parent {
            parent.get(key)
        } else {
            None
        }
    }

    pub fn define(&self, key: impl Into<String>, value: SrsValue) {
        self.vars.borrow_mut().insert(key.into(), value);
    }
}

impl Default for Env {
    fn default() -> Self {
        Env {
            vars: RefCell::new(HashMap::new()),
            parent: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_usage() {
        let env = Env::root();
        env.define("i", SrsValue::Integer(2));
        let result = env.get("i");
        assert_eq!(Some(SrsValue::Integer(2)), result);
    }

    #[test]
    fn test_chain_data_in_child() {
        let parent = Env::root();
        let child = Env::child(&parent);
        child.define("i", SrsValue::Integer(2));
        let result = child.get("i");
        assert_eq!(Some(SrsValue::Integer(2)), result);
    }

    #[test]
    fn test_chain_data_in_parent() {
        let parent = Env::root();
        parent.define("i", SrsValue::Integer(2));

        let child = Env::child(&parent);
        let result = child.get("i");
        assert_eq!(Some(SrsValue::Integer(2)), result);
    }

    #[test]
    fn test_chain_data_value_not_in() {
        let parent = Env::root();
        let child = Env::child(&parent);
        let result = child.get("i");
        assert!(result.is_none());
    }
}
