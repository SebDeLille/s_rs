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
        "string->list".to_string(),
        SrsValue::Native(Native {
            name: "string->list",
            func: Rc::new(native_string_to_list),
        }),
    );
    env.define(
        "string-index".to_string(),
        SrsValue::Native(Native {
            name: "string-index",
            func: Rc::new(native_string_index),
        }),
    );
    env.define(
        "string-copy".to_string(),
        SrsValue::Native(Native {
            name: "string-copy",
            func: Rc::new(native_string_copy),
        }),
    );
    env.define(
        "number->string".to_string(),
        SrsValue::Native(Native {
            name: "number->string",
            func: Rc::new(native_number_to_string),
        }),
    );
    env.define(
        "string->number".to_string(),
        SrsValue::Native(Native {
            name: "string->number",
            func: Rc::new(native_string_to_number),
        }),
    );
    env.define(
        "char=?".to_string(),
        SrsValue::Native(Native {
            name: "char=?",
            func: Rc::new(native_char_eq),
        }),
    );
    env.define(
        "char?".to_string(),
        SrsValue::Native(Native {
            name: "char?",
            func: Rc::new(native_char_p),
        }),
    );
    env.define(
        "char->integer".to_string(),
        SrsValue::Native(Native {
            name: "char->integer",
            func: Rc::new(native_char_to_integer),
        }),
    );
    env.define(
        "integer->char".to_string(),
        SrsValue::Native(Native {
            name: "integer->char",
            func: Rc::new(native_integer_to_char),
        }),
    );
    env.define(
        "char-whitespace?".to_string(),
        SrsValue::Native(Native {
            name: "char-whitespace?",
            func: Rc::new(|args| {
                native_char_predicate(args, "char-whitespace?", char::is_whitespace)
            }),
        }),
    );
    env.define(
        "char-numeric?".to_string(),
        SrsValue::Native(Native {
            name: "char-numeric?",
            func: Rc::new(|args| native_char_predicate(args, "char-numeric?", char::is_numeric)),
        }),
    );
    env.define(
        "char-alphabetic?".to_string(),
        SrsValue::Native(Native {
            name: "char-alphabetic?",
            func: Rc::new(|args| {
                native_char_predicate(args, "char-alphabetic?", char::is_alphabetic)
            }),
        }),
    );
    env.define(
        "char-upcase".to_string(),
        SrsValue::Native(Native {
            name: "char-upcase",
            func: Rc::new(|args| native_char_case(args, "char-upcase", true)),
        }),
    );
    env.define(
        "char-downcase".to_string(),
        SrsValue::Native(Native {
            name: "char-downcase",
            func: Rc::new(|args| native_char_case(args, "char-downcase", false)),
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
        [SrsValue::String(cell)] => Ok(SrsValue::Integer(cell.borrow().chars().count() as i64)),
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

/// `(string->list string [start [end]])`: returns the characters in the
/// selected string range as a proper list.
fn native_string_to_list(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(cell), ..] => {
            let string = cell.borrow();
            let (start, end) = string_range(args, string.chars().count(), "string->list")?;
            let chars = string
                .chars()
                .skip(start)
                .take(end - start)
                .map(SrsValue::Character)
                .collect();
            Ok(super::super::util::vec_to_list(chars))
        }
        [] => Err("not enough arguments to string->list".to_string()),
        [_, ..] => Err("wrong type: expected string".to_string()),
    }
}

/// `(string-index string char)`: returns the character offset of the first
/// occurrence, or `#f` when the character is not present.
fn native_string_index(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(cell), SrsValue::Character(needle)] => {
            let result = cell.borrow().chars().position(|c| c == *needle);
            Ok(result
                .map(|index| SrsValue::Integer(index as i64))
                .unwrap_or(SrsValue::Boolean(false)))
        }
        [SrsValue::String(_), _] => {
            Err("wrong type: expected character to string-index".to_string())
        }
        [_, _] => Err("wrong type: expected string to string-index".to_string()),
        [] | [_] => Err("not enough arguments to string-index".to_string()),
        _ => Err("too many arguments to string-index".to_string()),
    }
}

/// `(string-copy string [start [end]])`: returns a fresh string containing
/// the selected character range.
fn native_string_copy(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(cell), ..] => {
            let string = cell.borrow();
            let (start, end) = string_range(args, string.chars().count(), "string-copy")?;
            let copy = string.chars().skip(start).take(end - start).collect();
            Ok(SrsValue::String(Rc::new(RefCell::new(copy))))
        }
        [] => Err("not enough arguments to string-copy".to_string()),
        [_, ..] => Err("wrong type: expected string".to_string()),
    }
}

fn string_range(
    args: &[SrsValue],
    length: usize,
    proc_name: &str,
) -> Result<(usize, usize), String> {
    let start = match args.get(1) {
        Some(value) => index_from(value, proc_name)?,
        None => 0,
    };
    let end = match args.get(2) {
        Some(value) => index_from(value, proc_name)?,
        None => length,
    };
    if args.len() > 3 {
        return Err(format!("too many arguments to {}", proc_name));
    }
    if end > length {
        return Err(format!("{}: index out of range", proc_name));
    }
    if start > end {
        return Err(format!("{}: start index greater than end", proc_name));
    }
    Ok((start, end))
}

/// `(number->string number [radix])`: formats an exact number in radix 2,
/// 8, 10, or 16. Inexact numbers are supported in radix 10.
fn native_number_to_string(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [number] => number_to_string(number, 10),
        [number, radix] => number_to_string(number, radix_from(radix, "number->string")?),
        [] => Err("not enough arguments to number->string".to_string()),
        _ => Err("too many arguments to number->string".to_string()),
    }
}

fn number_to_string(number: &SrsValue, radix: u32) -> Result<SrsValue, String> {
    let text = match number {
        SrsValue::Integer(n) => format_integer(*n, radix),
        SrsValue::Rational(n, d) => {
            if *d == 0 {
                return Err("number->string: invalid rational denominator".to_string());
            }
            format!(
                "{}/{}",
                format_integer(*n, radix),
                format_integer(*d, radix)
            )
        }
        SrsValue::Float(value) if radix == 10 => {
            let mut text = value.to_string();
            if value.is_finite() && !text.contains(['.', 'e', 'E']) {
                text.push_str(".0");
            }
            text
        }
        SrsValue::Float(_) => {
            return Err("number->string: inexact numbers require radix 10".to_string());
        }
        _ => return Err("wrong type: expected number".to_string()),
    };
    Ok(SrsValue::String(Rc::new(RefCell::new(text))))
}

fn format_integer(value: i64, radix: u32) -> String {
    let negative = value < 0;
    let magnitude = value.unsigned_abs();
    let digits = match radix {
        2 => format!("{:b}", magnitude),
        8 => format!("{:o}", magnitude),
        10 => return value.to_string(),
        16 => format!("{:x}", magnitude),
        _ => unreachable!("radix validated before formatting"),
    };
    if negative {
        format!("-{}", digits)
    } else {
        digits
    }
}

/// `(string->number string [radix])`: parses a number, returning `#f` if the
/// string is not a valid number in the requested radix.
fn native_string_to_number(args: &[SrsValue]) -> Result<SrsValue, String> {
    let (string, radix) = match args {
        [SrsValue::String(cell)] => (cell.borrow().clone(), 10),
        [SrsValue::String(cell), radix] => {
            (cell.borrow().clone(), radix_from(radix, "string->number")?)
        }
        [] => return Err("not enough arguments to string->number".to_string()),
        [_] => return Err("wrong type: expected string".to_string()),
        [_, _] => return Err("wrong type: expected string".to_string()),
        _ => return Err("too many arguments to string->number".to_string()),
    };
    Ok(parse_scheme_number(&string, radix).unwrap_or(SrsValue::Boolean(false)))
}

fn radix_from(value: &SrsValue, proc_name: &str) -> Result<u32, String> {
    match value {
        SrsValue::Integer(2) => Ok(2),
        SrsValue::Integer(8) => Ok(8),
        SrsValue::Integer(10) => Ok(10),
        SrsValue::Integer(16) => Ok(16),
        SrsValue::Integer(_) => Err(format!("{}: radix must be 2, 8, 10, or 16", proc_name)),
        _ => Err(format!(
            "wrong type: expected integer radix to {}",
            proc_name
        )),
    }
}

fn parse_scheme_number(text: &str, radix: u32) -> Option<SrsValue> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some((numerator, denominator)) = text.split_once('/') {
        if denominator.contains('/') {
            return None;
        }
        let numerator = i64::from_str_radix(numerator, radix).ok()?;
        let denominator = i64::from_str_radix(denominator, radix).ok()?;
        if denominator == 0 {
            return None;
        }
        if denominator == 1 {
            return Some(SrsValue::Integer(numerator));
        }
        return Some(SrsValue::Rational(numerator, denominator));
    }
    if radix == 10 {
        if let Ok(integer) = text.parse::<i64>() {
            return Some(SrsValue::Integer(integer));
        }
        return text.parse::<f64>().ok().map(SrsValue::Float);
    }
    i64::from_str_radix(text, radix).ok().map(SrsValue::Integer)
}

fn native_char_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(matches!(value, SrsValue::Character(_)))),
        [] => Err("not enough arguments to char?".to_string()),
        _ => Err("too many arguments to char?".to_string()),
    }
}

fn native_char_to_integer(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Character(c)] => Ok(SrsValue::Integer(*c as i64)),
        [_] => Err("wrong type: expected character".to_string()),
        [] => Err("not enough arguments to char->integer".to_string()),
        _ => Err("too many arguments to char->integer".to_string()),
    }
}

fn native_integer_to_char(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Integer(n)] => u32::try_from(*n)
            .ok()
            .and_then(char::from_u32)
            .map(SrsValue::Character)
            .ok_or_else(|| "integer->char: integer is not a Unicode scalar value".to_string()),
        [_] => Err("wrong type: expected integer".to_string()),
        [] => Err("not enough arguments to integer->char".to_string()),
        _ => Err("too many arguments to integer->char".to_string()),
    }
}

fn native_char_predicate(
    args: &[SrsValue],
    proc_name: &str,
    predicate: fn(char) -> bool,
) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Character(c)] => Ok(SrsValue::Boolean(predicate(*c))),
        [_] => Err(format!("wrong type: expected character to {}", proc_name)),
        [] => Err(format!("not enough arguments to {}", proc_name)),
        _ => Err(format!("too many arguments to {}", proc_name)),
    }
}

fn native_char_case(
    args: &[SrsValue],
    proc_name: &str,
    uppercase: bool,
) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Character(c)] => {
            let converted: String = if uppercase {
                c.to_uppercase().collect()
            } else {
                c.to_lowercase().collect()
            };
            let mut chars = converted.chars();
            let first = chars.next().unwrap_or(*c);
            Ok(SrsValue::Character(if chars.next().is_some() {
                *c
            } else {
                first
            }))
        }
        [_] => Err(format!("wrong type: expected character to {}", proc_name)),
        [] => Err(format!("not enough arguments to {}", proc_name)),
        _ => Err(format!("too many arguments to {}", proc_name)),
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
