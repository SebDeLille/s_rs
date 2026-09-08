use std::cell::RefCell;
use std::fmt;
use std::iter::Peekable;
use std::rc::Rc;
use std::slice::Iter;

use crate::interpretor::lexical_analyzer::Lexeme;
use crate::types::core::SrsValue;

/// The specific reason a [`ReadError`] was raised.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadErrorKind {
    /// Input ended while a list, vector, or quoted datum was still open.
    UnexpectedEof,
    /// A `)` was encountered with no matching open list/vector.
    UnexpectedRParen,
    /// A `.` appeared outside of a list context (e.g. at the top level).
    DotOutsideList,
    /// A `.` was followed by zero, or more than one, trailing datum
    /// before the closing `)` (e.g. `(1 . )` or `(1 . 2 3)`).
    MisplacedDot,
    /// `'`, `` ` ``, `,` or `,@` appeared with no following datum.
    MissingDatumAfterQuote,
}

impl fmt::Display for ReadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadErrorKind::UnexpectedEof => write!(f, "unexpected end of input"),
            ReadErrorKind::UnexpectedRParen => write!(f, "unexpected ')'"),
            ReadErrorKind::DotOutsideList => write!(f, "'.' outside of a list"),
            ReadErrorKind::MisplacedDot => write!(f, "misplaced '.' in list"),
            ReadErrorKind::MissingDatumAfterQuote => {
                write!(f, "expected a datum after quote/quasiquote/unquote")
            }
        }
    }
}

/// A parse error produced while reading [`Lexeme`]s into [`SrsValue`]s.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadError {
    pub kind: ReadErrorKind,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for ReadError {}

fn err<T>(kind: ReadErrorKind) -> Result<T, ReadError> {
    Err(ReadError { kind })
}

/// Reads every top-level datum out of the given lexeme stream, turning them
/// into [`SrsValue`] trees (the closest thing this homoiconic language has
/// to an AST).
pub fn read_all(lexemes: Vec<Lexeme>) -> Result<Vec<SrsValue>, ReadError> {
    let mut values = Vec::new();
    let mut it = lexemes.iter().peekable();

    while it.peek().is_some() {
        values.push(read_datum(&mut it)?);
    }

    Ok(values)
}

/// Reads a single datum, consuming whatever lexemes it needs.
fn read_datum(it: &mut Peekable<Iter<'_, Lexeme>>) -> Result<SrsValue, ReadError> {
    match it.next() {
        None => err(ReadErrorKind::UnexpectedEof),
        Some(lexeme) => match lexeme {
            Lexeme::LParen => read_list(it),
            Lexeme::RParen => err(ReadErrorKind::UnexpectedRParen),
            Lexeme::VectorOpen => read_vector(it),
            Lexeme::Dot => err(ReadErrorKind::DotOutsideList),
            Lexeme::Quote => read_quoted(it, "quote"),
            Lexeme::Quasiquote => read_quoted(it, "quasiquote"),
            Lexeme::Unquote => read_quoted(it, "unquote"),
            Lexeme::UnquoteSplicing => read_quoted(it, "unquote-splicing"),
            Lexeme::Integer(n) => Ok(SrsValue::Integer(*n)),
            Lexeme::Float(f) => Ok(SrsValue::Float(*f)),
            Lexeme::Boolean(b) => Ok(SrsValue::Boolean(*b)),
            Lexeme::Character(c) => Ok(SrsValue::Character(*c)),
            Lexeme::String(s) => Ok(SrsValue::String(Rc::new(RefCell::new(s.clone())))),
            Lexeme::Id(name) => Ok(SrsValue::Symbol(name.clone())),
        },
    }
}

/// Wraps the next datum as `(symbol datum)`, e.g. `'x` -> `(quote x)`.
fn read_quoted(it: &mut Peekable<Iter<'_, Lexeme>>, symbol: &str) -> Result<SrsValue, ReadError> {
    if it.peek().is_none() {
        return err(ReadErrorKind::MissingDatumAfterQuote);
    }
    let datum = read_datum(it)?;
    Ok(cons(
        SrsValue::Symbol(symbol.to_string()),
        cons(datum, SrsValue::Nil),
    ))
}

/// Reads datums until the matching `)`, building a proper or dotted list.
/// The opening `(` has already been consumed by the caller.
fn read_list(it: &mut Peekable<Iter<'_, Lexeme>>) -> Result<SrsValue, ReadError> {
    let mut items: Vec<SrsValue> = Vec::new();
    let mut tail = SrsValue::Nil;

    loop {
        match it.peek() {
            None => return err(ReadErrorKind::UnexpectedEof),
            Some(Lexeme::RParen) => {
                it.next();
                break;
            }
            Some(Lexeme::Dot) => {
                it.next();
                tail = read_datum(it)?;
                match it.next() {
                    Some(Lexeme::RParen) => break,
                    _ => return err(ReadErrorKind::MisplacedDot),
                }
            }
            Some(_) => {
                items.push(read_datum(it)?);
            }
        }
    }

    Ok(items
        .into_iter()
        .rev()
        .fold(tail, |acc, item| cons(item, acc)))
}

/// Reads datums until the matching `)`, building a vector. The opening
/// `#(` (i.e. [`Lexeme::VectorOpen`]) has already been consumed by the caller.
fn read_vector(it: &mut Peekable<Iter<'_, Lexeme>>) -> Result<SrsValue, ReadError> {
    let mut items: Vec<SrsValue> = Vec::new();

    loop {
        match it.peek() {
            None => return err(ReadErrorKind::UnexpectedEof),
            Some(Lexeme::RParen) => {
                it.next();
                break;
            }
            Some(_) => items.push(read_datum(it)?),
        }
    }

    Ok(SrsValue::Vector(Rc::new(RefCell::new(items))))
}

fn cons(car: SrsValue, cdr: SrsValue) -> SrsValue {
    SrsValue::Pair(Rc::new(RefCell::new((car, cdr))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretor::lexical_analyzer::get_lexemes;

    fn read(src: &str) -> Vec<SrsValue> {
        read_all(get_lexemes(src).unwrap()).unwrap()
    }

    fn car(v: &SrsValue) -> SrsValue {
        match v {
            SrsValue::Pair(cell) => cell.borrow().0.clone(),
            _ => panic!("expected pair, got {:?}", v),
        }
    }

    fn cdr(v: &SrsValue) -> SrsValue {
        match v {
            SrsValue::Pair(cell) => cell.borrow().1.clone(),
            _ => panic!("expected pair, got {:?}", v),
        }
    }

    fn list_len(mut v: SrsValue) -> usize {
        let mut n = 0;
        loop {
            match v {
                SrsValue::Nil => return n,
                SrsValue::Pair(_) => {
                    n += 1;
                    v = cdr(&v);
                }
                _ => panic!("improper list"),
            }
        }
    }

    #[test]
    fn empty_list_is_nil() {
        let v = read("()");
        assert_eq!(v.len(), 1);
        assert!(matches!(v[0], SrsValue::Nil));
    }

    #[test]
    fn atoms() {
        let v = read("23 3.5 #t #f \"hi\" #\\a foo");
        assert!(matches!(v[0], SrsValue::Integer(23)));
        assert!(matches!(v[1], SrsValue::Float(f) if f == 3.5));
        assert!(matches!(v[2], SrsValue::Boolean(true)));
        assert!(matches!(v[3], SrsValue::Boolean(false)));
        match &v[4] {
            SrsValue::String(s) => assert_eq!(*s.borrow(), "hi"),
            other => panic!("expected string, got {:?}", other),
        }
        assert!(matches!(v[5], SrsValue::Character('a')));
        assert!(matches!(&v[6], SrsValue::Symbol(s) if s == "foo"));
    }

    #[test]
    fn simple_list() {
        let v = read("(+ 1 2)");
        assert_eq!(v.len(), 1);
        assert_eq!(list_len(v[0].clone()), 3);
        assert!(matches!(&car(&v[0]), SrsValue::Symbol(s) if s == "+"));
    }

    #[test]
    fn nested_list() {
        let v = read("(+ (* 2 3) 4)");
        assert_eq!(v.len(), 1);
        assert_eq!(list_len(v[0].clone()), 3);
        let second = car(&cdr(&v[0]));
        assert_eq!(list_len(second.clone()), 3);
        assert!(matches!(&car(&second), SrsValue::Symbol(s) if s == "*"));
    }

    #[test]
    fn dotted_pair() {
        let v = read("(1 . 2)");
        assert_eq!(v.len(), 1);
        assert!(matches!(car(&v[0]), SrsValue::Integer(1)));
        assert!(matches!(cdr(&v[0]), SrsValue::Integer(2)));
    }

    #[test]
    fn dotted_list_with_several_leading_items() {
        let v = read("(1 2 . 3)");
        assert!(matches!(car(&v[0]), SrsValue::Integer(1)));
        let rest = cdr(&v[0]);
        assert!(matches!(car(&rest), SrsValue::Integer(2)));
        assert!(matches!(cdr(&rest), SrsValue::Integer(3)));
    }

    #[test]
    fn quote_shorthand() {
        let v = read("'x");
        assert_eq!(v.len(), 1);
        assert!(matches!(&car(&v[0]), SrsValue::Symbol(s) if s == "quote"));
        assert!(matches!(&car(&cdr(&v[0])), SrsValue::Symbol(s) if s == "x"));
        assert!(matches!(cdr(&cdr(&v[0])), SrsValue::Nil));
    }

    #[test]
    fn quasiquote_and_unquote_shorthand() {
        let v = read("`(a ,b ,@c)");
        assert!(matches!(&car(&v[0]), SrsValue::Symbol(s) if s == "quasiquote"));
        let inner = car(&cdr(&v[0]));
        // inner == (a (unquote b) (unquote-splicing c))
        assert!(matches!(&car(&inner), SrsValue::Symbol(s) if s == "a"));
        let second = car(&cdr(&inner));
        assert!(matches!(&car(&second), SrsValue::Symbol(s) if s == "unquote"));
        let third = car(&cdr(&cdr(&inner)));
        assert!(matches!(&car(&third), SrsValue::Symbol(s) if s == "unquote-splicing"));
    }

    #[test]
    fn vector_literal() {
        let v = read("#(1 2 3)");
        assert_eq!(v.len(), 1);
        match &v[0] {
            SrsValue::Vector(items) => {
                assert_eq!(items.borrow().len(), 3);
                assert!(matches!(items.borrow()[0], SrsValue::Integer(1)));
                assert!(matches!(items.borrow()[2], SrsValue::Integer(3)));
            }
            other => panic!("expected vector, got {:?}", other),
        }
    }

    #[test]
    fn nested_vector_in_list() {
        let v = read("(a #(1 2) b)");
        assert_eq!(list_len(v[0].clone()), 3);
        let vec_val = car(&cdr(&v[0]));
        assert!(matches!(vec_val, SrsValue::Vector(_)));
    }

    #[test]
    fn unexpected_rparen_errors() {
        let err = read_all(get_lexemes(")").unwrap()).unwrap_err();
        assert_eq!(err.kind, ReadErrorKind::UnexpectedRParen);
    }

    #[test]
    fn unterminated_list_errors() {
        let err = read_all(get_lexemes("(1 2").unwrap()).unwrap_err();
        assert_eq!(err.kind, ReadErrorKind::UnexpectedEof);
    }

    #[test]
    fn dot_outside_list_errors() {
        let err = read_all(get_lexemes(".").unwrap()).unwrap_err();
        assert_eq!(err.kind, ReadErrorKind::DotOutsideList);
    }

    #[test]
    fn misplaced_dot_errors_on_extra_datum() {
        let err = read_all(get_lexemes("(1 . 2 3)").unwrap()).unwrap_err();
        assert_eq!(err.kind, ReadErrorKind::MisplacedDot);
    }

    #[test]
    fn misplaced_dot_errors_on_missing_datum() {
        let err = read_all(get_lexemes("(1 . )").unwrap()).unwrap_err();
        assert_eq!(err.kind, ReadErrorKind::UnexpectedRParen);
    }

    #[test]
    fn missing_datum_after_quote_errors() {
        let err = read_all(get_lexemes("'").unwrap()).unwrap_err();
        assert_eq!(err.kind, ReadErrorKind::MissingDatumAfterQuote);
    }
}
