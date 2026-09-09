use std::cell::RefCell;
use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

use super::super::apply;
use super::super::util::{list_to_vec, vec_to_list};

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "cons".to_string(),
        SrsValue::Native(Native {
            name: "cons",
            func: native_cons,
        }),
    );
    env.define(
        "car".to_string(),
        SrsValue::Native(Native {
            name: "car",
            func: native_car,
        }),
    );
    env.define(
        "cdr".to_string(),
        SrsValue::Native(Native {
            name: "cdr",
            func: native_cdr,
        }),
    );
    env.define(
        "apply".to_string(),
        SrsValue::Native(Native {
            name: "apply",
            func: native_apply,
        }),
    );
    env.define(
        "map".to_string(),
        SrsValue::Native(Native {
            name: "map",
            func: native_map,
        }),
    );
}

/// `(cons car cdr)`: allocates a fresh mutable pair.
fn native_cons(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [car, cdr] => Ok(SrsValue::Pair(Rc::new(RefCell::new((
            car.clone(),
            cdr.clone(),
        ))))),
        [] | [_] => Err("not enough arguments to cons".to_string()),
        _ => Err("too many arguments to cons".to_string()),
    }
}

/// `(car pair)`: returns the first element of a pair.
fn native_car(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Pair(cell)] => Ok(cell.borrow().0.clone()),
        [_] => Err("wrong type: expected pair".to_string()),
        [] => Err("not enough arguments to car".to_string()),
        _ => Err("too many arguments to car".to_string()),
    }
}

/// `(cdr pair)`: returns the second element of a pair.
fn native_cdr(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Pair(cell)] => Ok(cell.borrow().1.clone()),
        [_] => Err("wrong type: expected pair".to_string()),
        [] => Err("not enough arguments to cdr".to_string()),
        _ => Err("too many arguments to cdr".to_string()),
    }
}

/// `(apply proc arg1 ... args)`: applies `proc` to a list of arguments
/// formed by prepending `arg1 ...` (if any) onto the final `args` list,
/// which must itself be a proper list (R5RS section 6.4).
fn native_apply(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to apply".to_string()),
        [_] => Err("not enough arguments to apply".to_string()),
        [proc, rest @ ..] => {
            let (leading, last) = rest.split_at(rest.len() - 1);
            let mut call_args = leading.to_vec();
            call_args.extend(list_to_vec(&last[0]).map_err(|e| e.to_string())?);
            apply(proc, &call_args).map_err(|e| e.to_string())
        }
    }
}

/// `(map proc list1 list2 ...)`: applies `proc` element-wise to one or more
/// lists, returning a new list of the results. Traversal stops as soon as
/// the shortest list is exhausted (R5RS section 6.4).
fn native_map(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to map".to_string()),
        [_] => Err("not enough arguments to map".to_string()),
        [proc, lists @ ..] => {
            let mut vecs = Vec::with_capacity(lists.len());
            for list in lists {
                vecs.push(list_to_vec(list).map_err(|e| e.to_string())?);
            }
            let len = vecs.iter().map(|v| v.len()).min().unwrap_or(0);
            let mut results = Vec::with_capacity(len);
            for i in 0..len {
                let call_args: Vec<SrsValue> = vecs.iter().map(|v| v[i].clone()).collect();
                results.push(apply(proc, &call_args).map_err(|e| e.to_string())?);
            }
            Ok(vec_to_list(results))
        }
    }
}
