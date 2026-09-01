use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum SrsValue {
    Nil,
    Integer(i64),
    Float(f64),
    String(String),
    Id(String),
    Bool(bool),
    List(Vec<SrsValue>),
    Vector(Vec<SrsValue>),
}

impl Display for SrsValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SrsValue::Nil => write!(f, "NIL"),
            SrsValue::Integer(i) => write!(f, "{}", i),
            SrsValue::Float(x) => write!(f, "{}", x),
            SrsValue::String(s) => write!(f, "\"{}\"", s),
            SrsValue::Id(s) => write!(f, "{}", s),
            SrsValue::Bool(b) => write!(f, "#{}", if *b { 't' } else { 'f' }),
            SrsValue::List(v) => {
                write!(f, "(")?;
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            SrsValue::Vector(v) => {
                write!(f, "#(")?;
                for (i, e) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl SrsValue {
    pub fn is_list(&self) -> bool {
        matches!(self, SrsValue::List(_))
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            SrsValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            SrsValue::Float(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_id(&self) -> Option<&str> {
        match self {
            SrsValue::Id(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<SrsValue>> {
        match self {
            SrsValue::List(v) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!("NIL".to_string(), SrsValue::Nil.to_string());
        assert_eq!("42".to_string(), SrsValue::Integer(42).to_string());
        assert_eq!("#t".to_string(), SrsValue::Bool(true).to_string());
        assert_eq!("#f".to_string(), SrsValue::Bool(false).to_string());
        assert_eq!(
            "(1 2 3)".to_string(),
            SrsValue::List(vec![
                SrsValue::Integer(1),
                SrsValue::Integer(2),
                SrsValue::Integer(3),
            ])
            .to_string()
        );
    }
}
