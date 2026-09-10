use std::cell::RefCell;
use std::rc::Rc;

use crate::types::core::{Env, PortData, SrsValue};

mod arithmetic;
mod io;
mod pairs;
mod trig;
mod vectors;

/// Builds a fresh global environment with the base arithmetic procedures
/// (`+`, `-`, `*`, `/`) already bound.
pub fn global_env() -> Rc<Env> {
    let env = Env::new(None);
    let stdin_port = Rc::new(RefCell::new(PortData::stdin()));
    let stdout_port = Rc::new(RefCell::new(PortData::stdout()));
    env.define(
        "*current-input-port*".to_string(),
        SrsValue::Port(stdin_port.clone()),
    );
    env.define(
        "*current-output-port*".to_string(),
        SrsValue::Port(stdout_port.clone()),
    );
    arithmetic::install(&env);
    trig::install(&env);
    pairs::install(&env);
    vectors::install(&env);
    io::install(&env, stdin_port, stdout_port);
    env
}

/// Converts a numeric [`SrsValue`] to `f64`, as needed by the transcendental
/// functions (`sin`, `cos`, `tan`, `atan`), which always return an inexact
/// (`Float`) result per R5RS section 6.2.6.
fn numeric_to_f64(value: &SrsValue) -> Result<f64, String> {
    match value {
        SrsValue::Integer(n) => Ok(*n as f64),
        SrsValue::Float(f) => Ok(*f),
        SrsValue::Rational(n, d) => Ok(*n as f64 / *d as f64),
        _ => Err("wrong type: expected number".to_string()),
    }
}

/// Compares two numeric [`SrsValue`]s, keeping exact `i64` comparison for
/// two integers and falling back to `f64` comparison for any mix involving
/// `Float` or `Rational`.
fn numeric_cmp(a: &SrsValue, b: &SrsValue) -> Result<std::cmp::Ordering, String> {
    match (a, b) {
        (SrsValue::Integer(x), SrsValue::Integer(y)) => Ok(x.cmp(y)),
        _ => {
            let fa = numeric_to_f64(a)?;
            let fb = numeric_to_f64(b)?;
            fa.partial_cmp(&fb)
                .ok_or_else(|| "wrong type: expected number".to_string())
        }
    }
}

/// Shared implementation for `=`, `<`, `>`, `<=`, `>=`: checks that `ok`
/// holds for every consecutive pair of `args`. A single argument is
/// trivially true; zero arguments is an error.
fn compare_chain(
    name: &str,
    args: &[SrsValue],
    ok: fn(std::cmp::Ordering) -> bool,
) -> Result<SrsValue, String> {
    if args.is_empty() {
        return Err(format!("not enough arguments to {}", name));
    }
    for pair in args.windows(2) {
        let ord = numeric_cmp(&pair[0], &pair[1])?;
        if !ok(ord) {
            return Ok(SrsValue::Boolean(false));
        }
    }
    Ok(SrsValue::Boolean(true))
}

/// Extracts a `usize` index from a `SrsValue::Integer`, rejecting negative
/// or non-integer indices.
fn index_from(value: &SrsValue, proc_name: &str) -> Result<usize, String> {
    match value {
        SrsValue::Integer(n) if *n >= 0 => Ok(*n as usize),
        SrsValue::Integer(_) => Err(format!("wrong type: negative index to {}", proc_name)),
        _ => Err(format!("wrong type: expected integer to {}", proc_name)),
    }
}

fn negate(value: &SrsValue) -> Result<SrsValue, String> {
    match value {
        SrsValue::Integer(n) => Ok(SrsValue::Integer(-n)),
        SrsValue::Float(f) => Ok(SrsValue::Float(-f)),
        _ => Err("wrong type: expected number".to_string()),
    }
}

fn reciprocal(value: &SrsValue) -> Result<SrsValue, String> {
    match value {
        SrsValue::Integer(0) => Err("division by zero".to_string()),
        SrsValue::Integer(n) => Ok(SrsValue::Rational(1, *n)),
        SrsValue::Float(f) if *f == 0.0 => Err("division by zero".to_string()),
        SrsValue::Float(f) => Ok(SrsValue::Float(1.0 / f)),
        _ => Err("wrong type: expected number".to_string()),
    }
}

fn fold_numbers(
    rest: &[SrsValue],
    init: SrsValue,
    iop: fn(i64, i64) -> i64,
    fop: fn(f64, f64) -> f64,
) -> Result<SrsValue, String> {
    let mut result = init;
    for value in rest {
        result = match (&result, value) {
            (SrsValue::Integer(a), SrsValue::Integer(b)) => SrsValue::Integer(iop(*a, *b)),
            (SrsValue::Integer(a), SrsValue::Float(b)) => SrsValue::Float(fop(*a as f64, *b)),
            (SrsValue::Float(a), SrsValue::Integer(b)) => SrsValue::Float(fop(*a, *b as f64)),
            (SrsValue::Float(a), SrsValue::Float(b)) => SrsValue::Float(fop(*a, *b)),
            _ => return Err("wrong type: expected number".to_string()),
        };
    }
    Ok(result)
}
