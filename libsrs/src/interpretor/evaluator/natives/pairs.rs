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
            func: Rc::new(native_cons),
        }),
    );
    env.define(
        "car".to_string(),
        SrsValue::Native(Native {
            name: "car",
            func: Rc::new(native_car),
        }),
    );
    env.define(
        "cdr".to_string(),
        SrsValue::Native(Native {
            name: "cdr",
            func: Rc::new(native_cdr),
        }),
    );
    install_cxr(env);
    env.define(
        "apply".to_string(),
        SrsValue::Native(Native {
            name: "apply",
            func: Rc::new(native_apply),
        }),
    );
    env.define(
        "map".to_string(),
        SrsValue::Native(Native {
            name: "map",
            func: Rc::new(native_map),
        }),
    );
    env.define(
        "length".to_string(),
        SrsValue::Native(Native {
            name: "length",
            func: Rc::new(native_length),
        }),
    );
    env.define(
        "null?".to_string(),
        SrsValue::Native(Native {
            name: "null?",
            func: Rc::new(native_is_null),
        }),
    );
    env.define(
        "eq?".to_string(),
        SrsValue::Native(Native {
            name: "eq?",
            func: Rc::new(native_eq_p),
        }),
    );
    env.define(
        "list".to_string(),
        SrsValue::Native(Native {
            name: "list",
            func: Rc::new(native_list),
        }),
    );
    env.define(
        "reverse".to_string(),
        SrsValue::Native(Native {
            name: "reverse",
            func: Rc::new(native_reverse),
        }),
    );
    env.define(
        "pair?".to_string(),
        SrsValue::Native(Native {
            name: "pair?",
            func: Rc::new(native_pair_p),
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

type CxrOp = fn(SrsValue) -> Result<SrsValue, String>;

/// Registers the R5RS composed pair accessors `caar`...`cddddr`.
fn install_cxr(env: &Rc<Env>) {
    // Right-to-left as in R5RS: each character from the right becomes car/cdr.
    let cxrs: &[(&str, &[CxrOp])] = &[
        ("caar", &[car, car]),
        ("cadr", &[car, cdr]),
        ("cdar", &[cdr, car]),
        ("cddr", &[cdr, cdr]),
        ("caaar", &[car, car, car]),
        ("caadr", &[car, car, cdr]),
        ("cadar", &[car, cdr, car]),
        ("caddr", &[car, cdr, cdr]),
        ("cdaar", &[cdr, car, car]),
        ("cdadr", &[cdr, car, cdr]),
        ("cddar", &[cdr, cdr, car]),
        ("cdddr", &[cdr, cdr, cdr]),
        ("caaaar", &[car, car, car, car]),
        ("caaadr", &[car, car, car, cdr]),
        ("caadar", &[car, car, cdr, car]),
        ("caaddr", &[car, car, cdr, cdr]),
        ("cadaar", &[car, cdr, car, car]),
        ("cadadr", &[car, cdr, car, cdr]),
        ("caddar", &[car, cdr, cdr, car]),
        ("cadddr", &[car, cdr, cdr, cdr]),
        ("cdaaar", &[cdr, car, car, car]),
        ("cdaadr", &[cdr, car, car, cdr]),
        ("cdadar", &[cdr, car, cdr, car]),
        ("cdaddr", &[cdr, car, cdr, cdr]),
        ("cddaar", &[cdr, cdr, car, car]),
        ("cddadr", &[cdr, cdr, car, cdr]),
        ("cdddar", &[cdr, cdr, cdr, car]),
        ("cddddr", &[cdr, cdr, cdr, cdr]),
    ];
    for (name, ops) in cxrs {
        let ops: Vec<CxrOp> = ops.to_vec();
        env.define(
            (*name).to_string(),
            SrsValue::Native(Native {
                name,
                func: Rc::new(move |args| cxr_apply(name, args, &ops)),
            }),
        );
    }
}

fn car(value: SrsValue) -> Result<SrsValue, String> {
    match value {
        SrsValue::Pair(cell) => Ok(cell.borrow().0.clone()),
        _ => Err("wrong type: expected pair".to_string()),
    }
}

fn cdr(value: SrsValue) -> Result<SrsValue, String> {
    match value {
        SrsValue::Pair(cell) => Ok(cell.borrow().1.clone()),
        _ => Err("wrong type: expected pair".to_string()),
    }
}

fn cxr_apply(name: &str, args: &[SrsValue], ops: &[CxrOp]) -> Result<SrsValue, String> {
    match args {
        [value] => ops.iter().rev().try_fold(value.clone(), |acc, op| op(acc)),
        [] => Err(format!("not enough arguments to {}", name)),
        _ => Err(format!("too many arguments to {}", name)),
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

/// `(length list)`: returns the number of elements in `list`, which must
/// be a proper list (R5RS section 6.3.2).
fn native_length(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [list] => {
            let items = list_to_vec(list).map_err(|e| e.to_string())?;
            Ok(SrsValue::Integer(items.len() as i64))
        }
        [] => Err("not enough arguments to length".to_string()),
        _ => Err("too many arguments to length".to_string()),
    }
}

/// `(null? obj)`: returns `#t` if `obj` is the empty list, `#f` otherwise
/// (R5RS section 6.3.2).
fn native_is_null(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [obj] => Ok(SrsValue::Boolean(matches!(obj, SrsValue::Nil))),
        [] => Err("not enough arguments to null?".to_string()),
        _ => Err("too many arguments to null?".to_string()),
    }
}

/// `(list obj ...)`: returns a newly allocated proper list of its arguments
/// (R5RS section 6.3.2).
fn native_list(args: &[SrsValue]) -> Result<SrsValue, String> {
    Ok(vec_to_list(args.to_vec()))
}

/// `(reverse list)`: returns a newly allocated list whose elements are the
/// elements of `list` in reverse order (R5RS section 6.3.2).
fn native_reverse(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [list] => {
            let mut items = list_to_vec(list).map_err(|e| e.to_string())?;
            items.reverse();
            Ok(vec_to_list(items))
        }
        [] => Err("not enough arguments to reverse".to_string()),
        _ => Err("too many arguments to reverse".to_string()),
    }
}

/// `(pair? obj)`: returns `#t` if `obj` is a pair, `#f` otherwise
/// (R5RS section 6.3.2).
fn native_pair_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [obj] => Ok(SrsValue::Boolean(matches!(obj, SrsValue::Pair(_)))),
        [] => Err("not enough arguments to pair?".to_string()),
        _ => Err("too many arguments to pair?".to_string()),
    }
}

/// `(eq? obj1 obj2)`: returns `#t` if `obj1` and `obj2` are the same
/// object (R5RS section 6.1). For mutable/Rc-wrapped values this uses
/// pointer identity; atomic values are compared by value.
fn native_eq_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [a, b] => Ok(SrsValue::Boolean(srs_value_eq(a, b))),
        [] | [_] => Err("not enough arguments to eq?".to_string()),
        _ => Err("too many arguments to eq?".to_string()),
    }
}

fn srs_value_eq(a: &SrsValue, b: &SrsValue) -> bool {
    match (a, b) {
        (SrsValue::Integer(x), SrsValue::Integer(y)) => x == y,
        (SrsValue::Float(x), SrsValue::Float(y)) => x == y,
        (SrsValue::Rational(n1, d1), SrsValue::Rational(n2, d2)) => n1 == n2 && d1 == d2,
        (SrsValue::Boolean(x), SrsValue::Boolean(y)) => x == y,
        (SrsValue::Character(x), SrsValue::Character(y)) => x == y,
        (SrsValue::Symbol(x), SrsValue::Symbol(y)) => x == y,
        (SrsValue::Nil, SrsValue::Nil) => true,
        (SrsValue::Unspecified, SrsValue::Unspecified) => true,
        (SrsValue::Eof, SrsValue::Eof) => true,
        (SrsValue::String(s1), SrsValue::String(s2)) => Rc::ptr_eq(s1, s2),
        (SrsValue::Pair(p1), SrsValue::Pair(p2)) => Rc::ptr_eq(p1, p2),
        (SrsValue::Vector(v1), SrsValue::Vector(v2)) => Rc::ptr_eq(v1, v2),
        (SrsValue::Procedure(l1), SrsValue::Procedure(l2)) => Rc::ptr_eq(l1, l2),
        (SrsValue::Promise(p1), SrsValue::Promise(p2)) => Rc::ptr_eq(p1, p2),
        (SrsValue::Port(p1), SrsValue::Port(p2)) => Rc::ptr_eq(p1, p2),
        (SrsValue::Native(n1), SrsValue::Native(n2)) => {
            n1.name == n2.name && Rc::ptr_eq(&n1.func, &n2.func)
        }
        _ => false,
    }
}
