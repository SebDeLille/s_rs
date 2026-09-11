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
    env.define(
        "exact->inexact".to_string(),
        SrsValue::Native(Native {
            name: "exact->inexact",
            func: Rc::new(native_exact_to_inexact),
        }),
    );
    env.define(
        "inexact->exact".to_string(),
        SrsValue::Native(Native {
            name: "inexact->exact",
            func: Rc::new(native_inexact_to_exact),
        }),
    );
    env.define(
        "exact?".to_string(),
        SrsValue::Native(Native {
            name: "exact?",
            func: Rc::new(native_is_exact),
        }),
    );
    env.define(
        "inexact?".to_string(),
        SrsValue::Native(Native {
            name: "inexact?",
            func: Rc::new(native_is_inexact),
        }),
    );
    env.define(
        "nan?".to_string(),
        SrsValue::Native(Native {
            name: "nan?",
            func: Rc::new(native_is_nan),
        }),
    );
    env.define(
        "finite?".to_string(),
        SrsValue::Native(Native {
            name: "finite?",
            func: Rc::new(native_is_finite),
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

/// `(exact->inexact z)`: coerces an exact number (`Integer` or `Rational`)
/// to its `Float` equivalent. A `Float` argument is returned unchanged.
fn native_exact_to_inexact(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [only] => Ok(SrsValue::Float(numeric_to_f64(only)?)),
        [] => Err("not enough arguments to exact->inexact".to_string()),
        _ => Err("too many arguments to exact->inexact".to_string()),
    }
}

/// `(inexact->exact z)`: coerces a `Float` to an exact `Integer` or
/// `Rational` representing the same value. Exact arguments (`Integer`,
/// `Rational`) are returned unchanged.
fn native_inexact_to_exact(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Integer(_) | SrsValue::Rational(_, _)] => Ok(args[0].clone()),
        [SrsValue::Float(f)] => Ok(float_to_exact(*f)),
        [_] => Err("wrong type: expected number to inexact->exact".to_string()),
        [] => Err("not enough arguments to inexact->exact".to_string()),
        _ => Err("too many arguments to inexact->exact".to_string()),
    }
}

/// `(exact? z)`: `#t` if `z` is an `Integer` or `Rational`.
fn native_is_exact(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(matches!(
            value,
            SrsValue::Integer(_) | SrsValue::Rational(_, _)
        ))),
        [] => Err("not enough arguments to exact?".to_string()),
        _ => Err("too many arguments to exact?".to_string()),
    }
}

/// `(inexact? z)`: `#t` if `z` is a `Float`.
fn native_is_inexact(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(matches!(value, SrsValue::Float(_)))),
        [] => Err("not enough arguments to inexact?".to_string()),
        _ => Err("too many arguments to inexact?".to_string()),
    }
}

/// `(nan? z)`: `#t` if `z` is a number and its `f64` value is NaN
/// (`Integer`/`Rational` are never NaN, only `Float` can be). Not R5RS,
/// added as a small extension (in the spirit of R7RS's `nan?`) so that
/// pure-Scheme code can detect and filter non-numeric floating point
/// results without any native support specific to a single use case.
fn native_is_nan(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(numeric_to_f64(value)?.is_nan())),
        [] => Err("not enough arguments to nan?".to_string()),
        _ => Err("too many arguments to nan?".to_string()),
    }
}

/// `(finite? z)`: `#t` if `z` is a number whose `f64` value is neither
/// infinite nor NaN. `Integer`/`Rational` are always finite. Not R5RS,
/// added as a small extension (in the spirit of R7RS's `finite?`).
fn native_is_finite(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(numeric_to_f64(value)?.is_finite())),
        [] => Err("not enough arguments to finite?".to_string()),
        _ => Err("too many arguments to finite?".to_string()),
    }
}

/// Converts a `Float` to an exact `Integer` or `Rational` representing
/// the same value, by repeatedly doubling until the value is integral
/// (bounded to avoid overflow), then reducing to lowest terms.
fn float_to_exact(f: f64) -> SrsValue {
    if !f.is_finite() {
        return SrsValue::Integer(0);
    }
    if f.fract() == 0.0 && f.abs() < 1.0e18 {
        return SrsValue::Integer(f as i64);
    }
    let mut num = f;
    let mut den: i64 = 1;
    while num.fract() != 0.0 && den < (1i64 << 52) {
        num *= 2.0;
        den *= 2;
    }
    let mut n = num as i64;
    let mut d = den;
    let g = gcd(n.abs(), d);
    if g > 1 {
        n /= g;
        d /= g;
    }
    if d == 1 {
        SrsValue::Integer(n)
    } else {
        SrsValue::Rational(n, d)
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a } else { gcd(b, a % b) }
}
