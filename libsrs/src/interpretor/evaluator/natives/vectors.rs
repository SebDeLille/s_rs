use std::cell::RefCell;
use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

use super::super::util::{list_to_vec, vec_to_list};
use super::index_from;

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "vector".to_string(),
        SrsValue::Native(Native {
            name: "vector",
            func: native_vector,
        }),
    );
    env.define(
        "make-vector".to_string(),
        SrsValue::Native(Native {
            name: "make-vector",
            func: native_make_vector,
        }),
    );
    env.define(
        "vector?".to_string(),
        SrsValue::Native(Native {
            name: "vector?",
            func: native_vector_p,
        }),
    );
    env.define(
        "vector-length".to_string(),
        SrsValue::Native(Native {
            name: "vector-length",
            func: native_vector_length,
        }),
    );
    env.define(
        "vector-ref".to_string(),
        SrsValue::Native(Native {
            name: "vector-ref",
            func: native_vector_ref,
        }),
    );
    env.define(
        "vector-set!".to_string(),
        SrsValue::Native(Native {
            name: "vector-set!",
            func: native_vector_set,
        }),
    );
    env.define(
        "vector->list".to_string(),
        SrsValue::Native(Native {
            name: "vector->list",
            func: native_vector_to_list,
        }),
    );
    env.define(
        "list->vector".to_string(),
        SrsValue::Native(Native {
            name: "list->vector",
            func: native_list_to_vector,
        }),
    );
    env.define(
        "vector-fill!".to_string(),
        SrsValue::Native(Native {
            name: "vector-fill!",
            func: native_vector_fill,
        }),
    );
}

/// `(vector obj ...)`: allocates a fresh mutable vector containing `obj ...`.
fn native_vector(args: &[SrsValue]) -> Result<SrsValue, String> {
    Ok(SrsValue::Vector(Rc::new(RefCell::new(args.to_vec()))))
}

/// `(make-vector k [fill])`: allocates a fresh vector of `k` elements,
/// initialized to `fill` if given, or `0` otherwise.
fn native_make_vector(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to make-vector".to_string()),
        [k] => {
            let len = index_from(k, "make-vector")?;
            Ok(SrsValue::Vector(Rc::new(RefCell::new(vec![
                SrsValue::Integer(0);
                len
            ]))))
        }
        [k, fill] => {
            let len = index_from(k, "make-vector")?;
            Ok(SrsValue::Vector(Rc::new(RefCell::new(vec![
                fill.clone();
                len
            ]))))
        }
        _ => Err("too many arguments to make-vector".to_string()),
    }
}

/// `(vector? obj)`: `#t` if `obj` is a vector, `#f` otherwise.
fn native_vector_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [only] => Ok(SrsValue::Boolean(matches!(only, SrsValue::Vector(_)))),
        [] => Err("not enough arguments to vector?".to_string()),
        _ => Err("too many arguments to vector?".to_string()),
    }
}

/// `(vector-length vector)`: returns the number of elements in `vector`.
fn native_vector_length(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell)] => Ok(SrsValue::Integer(cell.borrow().len() as i64)),
        [_] => Err("wrong type: expected vector".to_string()),
        [] => Err("not enough arguments to vector-length".to_string()),
        _ => Err("too many arguments to vector-length".to_string()),
    }
}

/// `(vector-ref vector k)`: returns the `k`-th element of `vector`.
fn native_vector_ref(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell), k] => {
            let index = index_from(k, "vector-ref")?;
            cell.borrow()
                .get(index)
                .cloned()
                .ok_or_else(|| "vector-ref: index out of range".to_string())
        }
        [_, _] => Err("wrong type: expected vector".to_string()),
        [] | [_] => Err("not enough arguments to vector-ref".to_string()),
        _ => Err("too many arguments to vector-ref".to_string()),
    }
}

/// `(vector-set! vector k obj)`: stores `obj` at index `k` of `vector`.
fn native_vector_set(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell), k, obj] => {
            let index = index_from(k, "vector-set!")?;
            let mut vec = cell.borrow_mut();
            if index >= vec.len() {
                return Err("vector-set!: index out of range".to_string());
            }
            vec[index] = obj.clone();
            Ok(SrsValue::Unspecified)
        }
        [_, _, _] => Err("wrong type: expected vector".to_string()),
        [] | [_] | [_, _] => Err("not enough arguments to vector-set!".to_string()),
        _ => Err("too many arguments to vector-set!".to_string()),
    }
}

/// `(vector->list vector)`: returns a new list with the same elements as
/// `vector`.
fn native_vector_to_list(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell)] => Ok(vec_to_list(cell.borrow().clone())),
        [_] => Err("wrong type: expected vector".to_string()),
        [] => Err("not enough arguments to vector->list".to_string()),
        _ => Err("too many arguments to vector->list".to_string()),
    }
}

/// `(list->vector list)`: returns a new vector with the same elements as
/// `list`, which must be a proper list.
fn native_list_to_vector(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [list] => {
            let items = list_to_vec(list).map_err(|e| e.to_string())?;
            Ok(SrsValue::Vector(Rc::new(RefCell::new(items))))
        }
        [] => Err("not enough arguments to list->vector".to_string()),
        _ => Err("too many arguments to list->vector".to_string()),
    }
}

/// `(vector-fill! vector fill)`: stores `fill` in every element of
/// `vector`.
fn native_vector_fill(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell), fill] => {
            let mut vec = cell.borrow_mut();
            for item in vec.iter_mut() {
                *item = fill.clone();
            }
            Ok(SrsValue::Unspecified)
        }
        [_, _] => Err("wrong type: expected vector".to_string()),
        [] | [_] => Err("not enough arguments to vector-fill!".to_string()),
        _ => Err("too many arguments to vector-fill!".to_string()),
    }
}
