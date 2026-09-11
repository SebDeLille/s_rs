use std::cell::RefCell;
use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

use super::super::util::list_to_vec;
use super::index_from;

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "string?".to_string(),
        SrsValue::Native(Native {
            name: "string?",
            func: Rc::new(native_string_p),
        }),
    );
    env.define(
        "string-length".to_string(),
        SrsValue::Native(Native {
            name: "string-length",
            func: Rc::new(native_string_length),
        }),
    );
    env.define(
        "string-ref".to_string(),
        SrsValue::Native(Native {
            name: "string-ref",
            func: Rc::new(native_string_ref),
        }),
    );
    env.define(
        "string=?".to_string(),
        SrsValue::Native(Native {
            name: "string=?",
            func: Rc::new(native_string_eq),
        }),
    );
    env.define(
        "substring".to_string(),
        SrsValue::Native(Native {
            name: "substring",
            func: Rc::new(native_substring),
        }),
    );
    env.define(
        "string-append".to_string(),
        SrsValue::Native(Native {
            name: "string-append",
            func: Rc::new(native_string_append),
        }),
    );
    env.define(
        "list->string".to_string(),
        SrsValue::Native(Native {
            name: "list->string",
            func: Rc::new(native_list_to_string),
        }),
    );
    env.define(
        "char=?".to_string(),
        SrsValue::Native(Native {
            name: "char=?",
            func: Rc::new(native_char_eq),
        }),
    );
}

/// `(string? obj)`: returns `#t` if `obj` is a string, `#f` otherwise.
fn native_string_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [only] => Ok(SrsValue::Boolean(matches!(only, SrsValue::String(_)))),
        [] => Err("not enough arguments to string?".to_string()),
        _ => Err("too many arguments to string?".to_string()),
    }
}

/// `(string-length string)`: returns the number of characters in `string`.
fn native_string_length(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(cell)] => Ok(SrsValue::Integer(cell.borrow().len() as i64)),
        [_] => Err("wrong type: expected string".to_string()),
        [] => Err("not enough arguments to string-length".to_string()),
        _ => Err("too many arguments to string-length".to_string()),
    }
}

/// `(string-ref string k)`: returns the `k`-th character of `string`.
fn native_string_ref(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(cell), k] => {
            let index = index_from(k, "string-ref")?;
            cell.borrow()
                .chars()
                .nth(index)
                .map(SrsValue::Character)
                .ok_or_else(|| "string-ref: index out of range".to_string())
        }
        [_, _] => Err("wrong type: expected string and integer".to_string()),
        [] | [_] => Err("not enough arguments to string-ref".to_string()),
        _ => Err("too many arguments to string-ref".to_string()),
    }
}

/// `(string=? string1 string2 ...)`: returns `#t` iff all arguments are
/// strings with the same sequence of characters.
fn native_string_eq(args: &[SrsValue]) -> Result<SrsValue, String> {
    if args.is_empty() {
        return Err("not enough arguments to string=?".to_string());
    }
    let first = match &args[0] {
        SrsValue::String(cell) => cell.borrow().clone(),
        _ => return Err("wrong type: expected string".to_string()),
    };
    for arg in &args[1..] {
        match arg {
            SrsValue::String(cell) => {
                if cell.borrow().as_str() != first {
                    return Ok(SrsValue::Boolean(false));
                }
            }
            _ => return Err("wrong type: expected string".to_string()),
        }
    }
    Ok(SrsValue::Boolean(true))
}

/// `(substring string start end)`: returns a freshly allocated string
/// containing the characters from index `start` (inclusive) to `end`
/// (exclusive).
fn native_substring(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(cell), start, end] => {
            let s = cell.borrow();
            let start = index_from(start, "substring")?;
            let end = index_from(end, "substring")?;
            if end > s.len() {
                return Err("substring: index out of range".to_string());
            }
            if start > end {
                return Err("substring: start index greater than end".to_string());
            }
            let sub: String = s.chars().skip(start).take(end - start).collect();
            Ok(SrsValue::String(Rc::new(RefCell::new(sub))))
        }
        [_, _, _] => Err("wrong type: expected string".to_string()),
        [] | [_] | [_, _] => Err("not enough arguments to substring".to_string()),
        _ => Err("too many arguments to substring".to_string()),
    }
}

/// `(string-append string ...)`: returns a freshly allocated string whose
/// characters form the concatenation of the given strings.
fn native_string_append(args: &[SrsValue]) -> Result<SrsValue, String> {
    let mut result = String::new();
    for arg in args {
        match arg {
            SrsValue::String(cell) => result.push_str(&cell.borrow()),
            _ => return Err("wrong type: expected string".to_string()),
        }
    }
    Ok(SrsValue::String(Rc::new(RefCell::new(result))))
}

/// `(list->string list)`: returns a freshly allocated string formed from
/// the characters in `list`, which must be a proper list of characters.
fn native_list_to_string(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [list] => {
            let items = list_to_vec(list).map_err(|e| e.to_string())?;
            let mut result = String::with_capacity(items.len());
            for item in items {
                match item {
                    SrsValue::Character(c) => result.push(c),
                    _ => return Err("wrong type: expected list of characters".to_string()),
                }
            }
            Ok(SrsValue::String(Rc::new(RefCell::new(result))))
        }
        [] => Err("not enough arguments to list->string".to_string()),
        _ => Err("too many arguments to list->string".to_string()),
    }
}

/// `(char=? char1 char2 ...)`: returns `#t` iff all arguments are the same
/// character.
fn native_char_eq(args: &[SrsValue]) -> Result<SrsValue, String> {
    if args.is_empty() {
        return Err("not enough arguments to char=?".to_string());
    }
    let first = match &args[0] {
        SrsValue::Character(c) => *c,
        _ => return Err("wrong type: expected character".to_string()),
    };
    for arg in &args[1..] {
        match arg {
            SrsValue::Character(c) => {
                if *c != first {
                    return Ok(SrsValue::Boolean(false));
                }
            }
            _ => return Err("wrong type: expected character".to_string()),
        }
    }
    Ok(SrsValue::Boolean(true))
}
