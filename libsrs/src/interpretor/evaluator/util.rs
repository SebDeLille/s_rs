use std::cell::RefCell;
use std::rc::Rc;

use crate::types::core::SrsValue;

/// Converts a `Vec<SrsValue>` into a proper list (`Pair` chain ending in
/// `Nil`).
pub(crate) fn vec_to_list(values: Vec<SrsValue>) -> SrsValue {
    values.into_iter().rev().fold(SrsValue::Nil, |acc, v| {
        SrsValue::Pair(Rc::new(RefCell::new((v, acc))))
    })
}

/// Converts a proper list (`SrsValue::Pair` chain ending in `SrsValue::Nil`)
/// into a `Vec`. Fails on improper (dotted) lists.
pub(crate) fn list_to_vec(list: &SrsValue) -> Result<Vec<SrsValue>, super::EvalError> {
    let mut items = Vec::new();
    let mut current = list.clone();
    loop {
        match current {
            SrsValue::Nil => return Ok(items),
            SrsValue::Pair(cell) => {
                let (car, cdr) = cell.borrow().clone();
                items.push(car);
                current = cdr;
            }
            _ => return super::err(super::EvalErrorKind::WrongType),
        }
    }
}

/// R5RS truthiness: every value is true except `#f`.
pub(crate) fn is_truthy(value: &SrsValue) -> bool {
    !matches!(value, SrsValue::Boolean(false))
}
