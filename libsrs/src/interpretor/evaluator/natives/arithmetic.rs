use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

use super::{compare_chain, fold_numbers, negate, numeric_cmp, numeric_to_f64, reciprocal};

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "+".to_string(),
        SrsValue::Native(Native {
            name: "+",
            func: Rc::new(native_add),
        }),
    );
    env.define(
        "-".to_string(),
        SrsValue::Native(Native {
            name: "-",
            func: Rc::new(native_sub),
        }),
    );
    env.define(
        "*".to_string(),
        SrsValue::Native(Native {
            name: "*",
            func: Rc::new(native_mul),
        }),
    );
    env.define(
        "/".to_string(),
        SrsValue::Native(Native {
            name: "/",
            func: Rc::new(native_div),
        }),
    );
    env.define(
        "=".to_string(),
        SrsValue::Native(Native {
            name: "=",
            func: Rc::new(native_num_eq),
        }),
    );
    env.define(
        "<".to_string(),
        SrsValue::Native(Native {
            name: "<",
            func: Rc::new(native_lt),
        }),
    );
    env.define(
        ">".to_string(),
        SrsValue::Native(Native {
            name: ">",
            func: Rc::new(native_gt),
        }),
    );
    env.define(
        "<=".to_string(),
        SrsValue::Native(Native {
            name: "<=",
            func: Rc::new(native_le),
        }),
    );
    env.define(
        ">=".to_string(),
        SrsValue::Native(Native {
            name: ">=",
            func: Rc::new(native_ge),
        }),
    );
    env.define(
        "max".to_string(),
        SrsValue::Native(Native {
            name: "max",
            func: Rc::new(native_max),
        }),
    );
    env.define(
        "not".to_string(),
        SrsValue::Native(Native {
            name: "not",
            func: Rc::new(native_not),
        }),
    );
}

fn native_add(args: &[SrsValue]) -> Result<SrsValue, String> {
    fold_numbers(args, SrsValue::Integer(0), |a, b| a + b, |a, b| a + b)
}

fn native_sub(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to -".to_string()),
        [only] => negate(only),
        [first, rest @ ..] => fold_numbers(rest, first.clone(), |a, b| a - b, |a, b| a - b),
    }
}

fn native_mul(args: &[SrsValue]) -> Result<SrsValue, String> {
    fold_numbers(args, SrsValue::Integer(1), |a, b| a * b, |a, b| a * b)
}

fn native_div(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to /".to_string()),
        [only] => reciprocal(only),
        [first, rest @ ..] => {
            let mut result = first.clone();
            for value in rest {
                result = match (&result, value) {
                    (_, SrsValue::Integer(0)) => return Err("division by zero".to_string()),
                    (SrsValue::Integer(a), SrsValue::Integer(b)) => SrsValue::Integer(a / b),
                    (SrsValue::Integer(a), SrsValue::Float(b)) => SrsValue::Float(*a as f64 / b),
                    (SrsValue::Float(a), SrsValue::Integer(b)) => SrsValue::Float(a / *b as f64),
                    (SrsValue::Float(a), SrsValue::Float(b)) => SrsValue::Float(a / b),
                    _ => return Err("wrong type: expected number".to_string()),
                };
            }
            Ok(result)
        }
    }
}

fn native_num_eq(args: &[SrsValue]) -> Result<SrsValue, String> {
    compare_chain("=", args, |ord| ord == std::cmp::Ordering::Equal)
}

fn native_lt(args: &[SrsValue]) -> Result<SrsValue, String> {
    compare_chain("<", args, |ord| ord == std::cmp::Ordering::Less)
}

fn native_gt(args: &[SrsValue]) -> Result<SrsValue, String> {
    compare_chain(">", args, |ord| ord == std::cmp::Ordering::Greater)
}

fn native_le(args: &[SrsValue]) -> Result<SrsValue, String> {
    compare_chain("<=", args, |ord| ord != std::cmp::Ordering::Greater)
}

fn native_ge(args: &[SrsValue]) -> Result<SrsValue, String> {
    compare_chain(">=", args, |ord| ord != std::cmp::Ordering::Less)
}

/// `(max n1 n2 ...)`: returns the largest of its arguments. Per R5RS, if
/// any argument is inexact (`Float`), the result is coerced to inexact
/// even when the largest value came from an exact argument.
fn native_max(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to max".to_string()),
        [first, rest @ ..] => {
            let mut best = first.clone();
            let mut inexact = matches!(first, SrsValue::Float(_));
            for value in rest {
                inexact |= matches!(value, SrsValue::Float(_));
                if numeric_cmp(value, &best)? == std::cmp::Ordering::Greater {
                    best = value.clone();
                }
            }
            if inexact && !matches!(best, SrsValue::Float(_)) {
                Ok(SrsValue::Float(numeric_to_f64(&best)?))
            } else {
                Ok(best)
            }
        }
    }
}

/// `(not obj)`: `#t` if `obj` is `#f`, `#f` for any other value (per R5RS,
/// every value counts as true except `#f`).
fn native_not(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [only] => Ok(SrsValue::Boolean(matches!(only, SrsValue::Boolean(false)))),
        [] => Err("not enough arguments to not".to_string()),
        _ => Err("too many arguments to not".to_string()),
    }
}
