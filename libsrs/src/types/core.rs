use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Lexical environment: variable bindings + optional parent scope.
/// Wrapped in Rc<RefCell<_>> so closures can capture and share it.
#[derive(Debug)]
pub struct Env {
    pub bindings: RefCell<HashMap<String, SrsValue>>,
    pub parent: Option<Rc<Env>>,
}

impl Env {
    pub fn new(parent: Option<Rc<Env>>) -> Rc<Env> {
        Rc::new(Env {
            bindings: RefCell::new(HashMap::new()),
            parent,
        })
    }

    pub fn get(&self, name: &str) -> Option<SrsValue> {
        if let Some(v) = self.bindings.borrow().get(name) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.get(name))
    }

    pub fn define(&self, name: String, value: SrsValue) {
        self.bindings.borrow_mut().insert(name, value);
    }

    /// Mutates an existing binding in the nearest enclosing scope.
    /// Returns false if the name is unbound anywhere in the chain.
    pub fn set(&self, name: &str, value: SrsValue) -> bool {
        if self.bindings.borrow().contains_key(name) {
            self.bindings.borrow_mut().insert(name.to_string(), value);
            return true;
        }
        match &self.parent {
            Some(p) => p.set(name, value),
            None => false,
        }
    }
}

/// User-defined closure: parameter list, variadic rest param (if any), body, captured env.
#[derive(Clone)]
pub struct Lambda {
    pub params: Vec<String>,
    pub rest: Option<String>,
    pub body: Vec<SrsValue>,
    pub env: Rc<Env>,
}

impl fmt::Debug for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lambda")
            .field("params", &self.params)
            .field("rest", &self.rest)
            .field("body", &self.body)
            .finish()
    }
}

/// Native (builtin) procedure implemented in Rust.
pub type NativeFn = fn(&[SrsValue]) -> Result<SrsValue, String>;

#[derive(Clone)]
pub struct Native {
    pub name: &'static str,
    pub func: NativeFn,
}

impl fmt::Debug for Native {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Native").field("name", &self.name).finish()
    }
}

#[derive(Debug, Clone)]
pub enum SrsValue {
    Integer(i64),
    Float(f64),
    Rational(i64, i64),
    Boolean(bool),
    Character(char),
    /// Mutable string, shared for `string-set!` semantics.
    String(Rc<RefCell<String>>),
    Symbol(String),
    Nil,
    /// Mutable cons cell, shared for `set-car!`/`set-cdr!` and cyclic structures.
    Pair(Rc<RefCell<(SrsValue, SrsValue)>>),
    /// Mutable vector, shared for `vector-set!`.
    Vector(Rc<RefCell<Vec<SrsValue>>>),
    /// Result of `set!`, `define`, etc. — has no printable representation.
    Unspecified,
    /// Result of reading past end of input.
    Eof,
    Procedure(Rc<Lambda>),
    Native(Native),
    /// Delayed computation for `delay`/`force`.
    Promise(Rc<RefCell<PromiseState>>),
}

impl fmt::Display for SrsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SrsValue::Integer(n) => write!(f, "{}", n),
            SrsValue::Float(x) => write!(f, "{}", x),
            SrsValue::Rational(num, den) => write!(f, "{}/{}", num, den),
            SrsValue::Boolean(true) => write!(f, "#t"),
            SrsValue::Boolean(false) => write!(f, "#f"),
            SrsValue::Character(c) => write!(f, "#\\{}", format_char(*c)),
            SrsValue::String(s) => write!(f, "\"{}\"", s.borrow()),
            SrsValue::Symbol(s) => write!(f, "{}", s),
            SrsValue::Nil => write!(f, "()"),
            SrsValue::Pair(_) => {
                write!(f, "(")?;
                let mut first = true;
                let mut cur = self.clone();
                loop {
                    match cur {
                        SrsValue::Pair(p) => {
                            let (car, cdr) = p.borrow().clone();
                            if !first {
                                write!(f, " ")?;
                            }
                            first = false;
                            write!(f, "{}", car)?;
                            cur = cdr;
                        }
                        SrsValue::Nil => break,
                        other => {
                            write!(f, " . {}", other)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
            SrsValue::Vector(v) => {
                write!(f, "#(")?;
                for (i, item) in v.borrow().iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            SrsValue::Unspecified => Ok(()),
            SrsValue::Eof => write!(f, "#<eof>"),
            SrsValue::Procedure(_) => write!(f, "#<procedure>"),
            SrsValue::Native(n) => write!(f, "#<procedure:{}>", n.name),
            SrsValue::Promise(_) => write!(f, "#<promise>"),
        }
    }
}

fn format_char(c: char) -> String {
    match c {
        ' ' => "space".to_string(),
        '\n' => "newline".to_string(),
        '\t' => "tab".to_string(),
        '\r' => "return".to_string(),
        '\0' => "null".to_string(),
        c => c.to_string(),
    }
}

#[derive(Debug, Clone)]
pub enum PromiseState {
    Forced(SrsValue),
    Delayed(Vec<SrsValue>, Rc<Env>),
}
