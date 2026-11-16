use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

use super::{compare_chain, fold_numbers, negate, numeric_cmp, numeric_to_f64, reciprocal};

type Predicate = fn(&[SrsValue]) -> Result<SrsValue, String>;
type IntegerArgument = (i64, bool);

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
        "min".to_string(),
        SrsValue::Native(Native {
            name: "min",
            func: Rc::new(native_min),
        }),
    );
    env.define(
        "abs".to_string(),
        SrsValue::Native(Native {
            name: "abs",
            func: Rc::new(native_abs),
        }),
    );
    env.define(
        "quotient".to_string(),
        SrsValue::Native(Native {
            name: "quotient",
            func: Rc::new(native_quotient),
        }),
    );
    env.define(
        "remainder".to_string(),
        SrsValue::Native(Native {
            name: "remainder",
            func: Rc::new(native_remainder),
        }),
    );
    env.define(
        "modulo".to_string(),
        SrsValue::Native(Native {
            name: "modulo",
            func: Rc::new(native_modulo),
        }),
    );
    let predicates: [(&str, Predicate); 7] = [
        ("zero?", native_zero),
        ("positive?", native_positive),
        ("negative?", native_negative),
        ("even?", native_even),
        ("odd?", native_odd),
        ("number?", native_number),
        ("integer?", native_integer),
    ];
    for (name, func) in predicates {
        env.define(
            name.to_string(),
            SrsValue::Native(Native {
                name,
                func: Rc::new(func),
            }),
        );
    }
    let more: [(&'static str, Predicate); 8] = [
        ("floor", native_floor),
        ("round", native_round),
        ("sqrt", native_sqrt),
        ("expt", native_expt),
        ("gcd", native_gcd),
        ("lcm", native_lcm),
        ("exp", native_exp),
        ("log", native_log),
    ];
    for (name, func) in more {
        env.define(
            name.to_string(),
            SrsValue::Native(Native {
                name,
                func: Rc::new(func),
            }),
        );
    }
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

/// `(min n1 n2 ...)`: returns the smallest argument, producing an inexact
/// result whenever any argument is inexact (as required by R5RS).
fn native_min(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to min".to_string()),
        [first, rest @ ..] => {
            let mut best = first.clone();
            let mut inexact = matches!(first, SrsValue::Float(_));
            for value in rest {
                inexact |= matches!(value, SrsValue::Float(_));
                if numeric_cmp(value, &best)? == std::cmp::Ordering::Less {
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

fn native_abs(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Integer(n)] => n
            .checked_abs()
            .map(SrsValue::Integer)
            .ok_or_else(|| "integer overflow in abs".to_string()),
        [SrsValue::Float(n)] => Ok(SrsValue::Float(n.abs())),
        [SrsValue::Rational(n, d)] => {
            let numerator = if (*n < 0) != (*d < 0) {
                n.checked_neg()
                    .ok_or_else(|| "integer overflow in abs".to_string())?
            } else {
                *n
            };
            Ok(SrsValue::Rational(numerator, *d))
        }
        [] => Err("not enough arguments to abs".to_string()),
        [_] => Err("wrong type: expected number to abs".to_string()),
        _ => Err("too many arguments to abs".to_string()),
    }
}

/// Converts an integer-valued numeric argument to its integer value and
/// reports whether it was inexact.
fn integer_argument(value: &SrsValue, name: &str) -> Result<(i64, bool), String> {
    match value {
        SrsValue::Integer(n) => Ok((*n, false)),
        SrsValue::Rational(n, d) if *d != 0 && n.checked_rem(*d) == Some(0) => n
            .checked_div(*d)
            .map(|integer| (integer, false))
            .ok_or_else(|| format!("integer overflow in {}", name)),
        SrsValue::Float(n)
            if n.is_finite()
                && n.fract() == 0.0
                && *n >= i64::MIN as f64
                && *n < -(i64::MIN as f64) =>
        {
            Ok((*n as i64, true))
        }
        SrsValue::Rational(_, _) | SrsValue::Float(_) => {
            Err(format!("wrong type: expected integer to {}", name))
        }
        _ => Err("wrong type: expected number".to_string()),
    }
}

fn integer_pair(
    args: &[SrsValue],
    name: &str,
) -> Result<(IntegerArgument, IntegerArgument), String> {
    match args {
        [a, b] => Ok((integer_argument(a, name)?, integer_argument(b, name)?)),
        [] => Err(format!("not enough arguments to {}", name)),
        [_] => Err(format!("not enough arguments to {}", name)),
        _ => Err(format!("too many arguments to {}", name)),
    }
}

fn integer_division(
    args: &[SrsValue],
    name: &str,
    mode: IntegerDivision,
) -> Result<SrsValue, String> {
    let ((a, inexact_a), (b, inexact_b)) = integer_pair(args, name)?;
    if b == 0 {
        return Err("division by zero".to_string());
    }
    let value = match mode {
        IntegerDivision::Quotient => a.checked_div(b),
        IntegerDivision::Remainder => a
            .checked_rem(b)
            .or_else(|| (a == i64::MIN && b == -1).then_some(0)),
        IntegerDivision::Modulo => a
            .checked_rem(b)
            .or_else(|| (a == i64::MIN && b == -1).then_some(0))
            .and_then(|r| {
                if r != 0 && (r < 0) != (b < 0) {
                    r.checked_add(b)
                } else {
                    Some(r)
                }
            }),
    }
    .ok_or_else(|| format!("integer overflow in {}", name))?;
    if inexact_a || inexact_b {
        Ok(SrsValue::Float(value as f64))
    } else {
        Ok(SrsValue::Integer(value))
    }
}

enum IntegerDivision {
    Quotient,
    Remainder,
    Modulo,
}

fn native_quotient(args: &[SrsValue]) -> Result<SrsValue, String> {
    integer_division(args, "quotient", IntegerDivision::Quotient)
}

fn native_remainder(args: &[SrsValue]) -> Result<SrsValue, String> {
    integer_division(args, "remainder", IntegerDivision::Remainder)
}

fn native_modulo(args: &[SrsValue]) -> Result<SrsValue, String> {
    integer_division(args, "modulo", IntegerDivision::Modulo)
}

fn one_number<'a>(args: &'a [SrsValue], name: &str) -> Result<&'a SrsValue, String> {
    match args {
        [value @ (SrsValue::Integer(_) | SrsValue::Float(_) | SrsValue::Rational(_, _))] => {
            Ok(value)
        }
        [] => Err(format!("not enough arguments to {}", name)),
        [_] => Err("wrong type: expected number".to_string()),
        _ => Err(format!("too many arguments to {}", name)),
    }
}

fn native_zero(args: &[SrsValue]) -> Result<SrsValue, String> {
    let value = one_number(args, "zero?")?;
    Ok(SrsValue::Boolean(numeric_to_f64(value)? == 0.0))
}

fn native_positive(args: &[SrsValue]) -> Result<SrsValue, String> {
    let value = one_number(args, "positive?")?;
    Ok(SrsValue::Boolean(numeric_to_f64(value)? > 0.0))
}

fn native_negative(args: &[SrsValue]) -> Result<SrsValue, String> {
    let value = one_number(args, "negative?")?;
    Ok(SrsValue::Boolean(numeric_to_f64(value)? < 0.0))
}

fn parity(args: &[SrsValue], name: &str, even: bool) -> Result<SrsValue, String> {
    let value = one_number(args, name)?;
    let (n, _) = integer_argument(value, name)?;
    Ok(SrsValue::Boolean((n % 2 == 0) == even))
}

fn native_even(args: &[SrsValue]) -> Result<SrsValue, String> {
    parity(args, "even?", true)
}

fn native_odd(args: &[SrsValue]) -> Result<SrsValue, String> {
    parity(args, "odd?", false)
}

fn native_number(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(matches!(
            value,
            SrsValue::Integer(_) | SrsValue::Float(_) | SrsValue::Rational(_, _)
        ))),
        [] => Err("not enough arguments to number?".to_string()),
        _ => Err("too many arguments to number?".to_string()),
    }
}

fn native_integer(args: &[SrsValue]) -> Result<SrsValue, String> {
    let value = match args {
        [value] => value,
        [] => return Err("not enough arguments to integer?".to_string()),
        _ => return Err("too many arguments to integer?".to_string()),
    };
    let integer = match value {
        SrsValue::Integer(_) => true,
        SrsValue::Rational(n, d) => *d != 0 && n.checked_rem(*d) == Some(0),
        SrsValue::Float(n) => n.is_finite() && n.fract() == 0.0,
        _ => false,
    };
    Ok(SrsValue::Boolean(integer))
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

fn make_rational(n: i64, d: i64) -> SrsValue {
    let (mut n, mut d) = if d < 0 { (-n, -d) } else { (n, d) };
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

fn rounding(
    args: &[SrsValue],
    name: &str,
    on_float: fn(f64) -> f64,
    on_rational: fn(i64, i64) -> i64,
) -> Result<SrsValue, String> {
    match one_number(args, name)? {
        SrsValue::Integer(n) => Ok(SrsValue::Integer(*n)),
        SrsValue::Float(f) => Ok(SrsValue::Float(on_float(*f))),
        SrsValue::Rational(n, d) if *d != 0 => {
            let (n, d) = if *d < 0 { (-*n, -*d) } else { (*n, *d) };
            Ok(SrsValue::Integer(on_rational(n, d)))
        }
        _ => Err(format!("wrong type: expected number to {}", name)),
    }
}

fn native_floor(args: &[SrsValue]) -> Result<SrsValue, String> {
    rounding(args, "floor", f64::floor, |n, d| n.div_euclid(d))
}

/// Rounds to the nearest integer, ties to even (as required by R5RS).
fn native_round(args: &[SrsValue]) -> Result<SrsValue, String> {
    rounding(args, "round", f64::round_ties_even, |n, d| {
        let fl = n.div_euclid(d);
        let twice_rem = 2 * (n - fl * d) as i128;
        let d = d as i128;
        if twice_rem > d || (twice_rem == d && fl % 2 != 0) {
            fl + 1
        } else {
            fl
        }
    })
}

fn exact_sqrt(n: i64) -> Option<i64> {
    if n < 0 {
        return None;
    }
    let r = (n as f64).sqrt().round() as i64;
    (r.checked_mul(r) == Some(n)).then_some(r)
}

fn native_sqrt(args: &[SrsValue]) -> Result<SrsValue, String> {
    let value = one_number(args, "sqrt")?;
    match value {
        SrsValue::Integer(n) => {
            if let Some(r) = exact_sqrt(*n) {
                return Ok(SrsValue::Integer(r));
            }
        }
        SrsValue::Rational(n, d) if *d > 0 => {
            if let (Some(a), Some(b)) = (exact_sqrt(*n), exact_sqrt(*d)) {
                return Ok(make_rational(a, b));
            }
        }
        _ => {}
    }
    Ok(SrsValue::Float(numeric_to_f64(value)?.sqrt()))
}

fn native_expt(args: &[SrsValue]) -> Result<SrsValue, String> {
    let (base, exponent) = match args {
        [a, b] => (a, b),
        [] | [_] => return Err("not enough arguments to expt".to_string()),
        _ => return Err("too many arguments to expt".to_string()),
    };
    let base_exact = match base {
        SrsValue::Integer(n) => Some((*n, 1)),
        SrsValue::Rational(n, d) if *d != 0 => Some((*n, *d)),
        _ => None,
    };
    if let (Some((n, d)), SrsValue::Integer(e)) = (base_exact, exponent) {
        let magnitude =
            u32::try_from(e.unsigned_abs()).map_err(|_| "integer overflow in expt".to_string())?;
        let overflow = || "integer overflow in expt".to_string();
        let pn = n.checked_pow(magnitude).ok_or_else(overflow)?;
        let pd = d.checked_pow(magnitude).ok_or_else(overflow)?;
        return if *e >= 0 {
            Ok(make_rational(pn, pd))
        } else if pn == 0 {
            Err("division by zero".to_string())
        } else {
            Ok(make_rational(pd, pn))
        };
    }
    one_number(std::slice::from_ref(base), "expt")?;
    one_number(std::slice::from_ref(exponent), "expt")?;
    Ok(SrsValue::Float(
        numeric_to_f64(base)?.powf(numeric_to_f64(exponent)?),
    ))
}

fn fold_integers(
    args: &[SrsValue],
    name: &str,
    identity: i64,
    combine: fn(i64, i64) -> Option<i64>,
) -> Result<SrsValue, String> {
    let mut acc = identity;
    let mut inexact = false;
    for value in args {
        let (n, inex) = integer_argument(value, name)?;
        inexact |= inex;
        acc = combine(acc, n).ok_or_else(|| format!("integer overflow in {}", name))?;
    }
    if inexact {
        Ok(SrsValue::Float(acc as f64))
    } else {
        Ok(SrsValue::Integer(acc))
    }
}

fn native_gcd(args: &[SrsValue]) -> Result<SrsValue, String> {
    fold_integers(args, "gcd", 0, |a, b| {
        a.checked_abs()
            .and_then(|a| b.checked_abs().map(|b| gcd(a, b)))
    })
}

fn native_lcm(args: &[SrsValue]) -> Result<SrsValue, String> {
    fold_integers(args, "lcm", 1, |a, b| {
        if a == 0 || b == 0 {
            return Some(0);
        }
        let (a, b) = (a.checked_abs()?, b.checked_abs()?);
        (a / gcd(a, b)).checked_mul(b)
    })
}

fn native_exp(args: &[SrsValue]) -> Result<SrsValue, String> {
    let value = one_number(args, "exp")?;
    Ok(SrsValue::Float(numeric_to_f64(value)?.exp()))
}

fn native_log(args: &[SrsValue]) -> Result<SrsValue, String> {
    let value = one_number(args, "log")?;
    Ok(SrsValue::Float(numeric_to_f64(value)?.ln()))
}
