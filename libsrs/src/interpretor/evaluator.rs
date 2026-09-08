use std::fmt;
use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

/// The specific reason an [`EvalError`] was raised.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalErrorKind {
    /// A symbol was referenced but has no binding in scope.
    UnboundVariable(String),
    /// The operator position of a combination did not evaluate to
    /// something callable.
    NotAProcedure,
    /// A special form (e.g. `define`) was given the wrong shape of
    /// arguments (missing/extra parts, wrong types).
    NotEnoughArguments,
    TooManyArguments,
    WrongType,
    /// A native procedure reported an error (e.g. division by zero).
    Native(String),
}

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalErrorKind::UnboundVariable(name) => write!(f, "unbound variable: {}", name),
            EvalErrorKind::NotAProcedure => write!(f, "not a procedure"),
            EvalErrorKind::NotEnoughArguments => write!(f, "not enough arguments"),
            EvalErrorKind::TooManyArguments => write!(f, "too many arguments"),
            EvalErrorKind::WrongType => write!(f, "wrong type"),
            EvalErrorKind::Native(msg) => write!(f, "{}", msg),
        }
    }
}

/// An error raised while evaluating an [`SrsValue`] expression.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalError {
    pub kind: EvalErrorKind,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for EvalError {}

fn err<T>(kind: EvalErrorKind) -> Result<T, EvalError> {
    Err(EvalError { kind })
}

/// Builds a fresh global environment with the base arithmetic procedures
/// (`+`, `-`, `*`, `/`) already bound.
pub fn global_env() -> Rc<Env> {
    let env = Env::new(None);
    env.define(
        "+".to_string(),
        SrsValue::Native(Native {
            name: "+",
            func: native_add,
        }),
    );
    env.define(
        "-".to_string(),
        SrsValue::Native(Native {
            name: "-",
            func: native_sub,
        }),
    );
    env.define(
        "*".to_string(),
        SrsValue::Native(Native {
            name: "*",
            func: native_mul,
        }),
    );
    env.define(
        "/".to_string(),
        SrsValue::Native(Native {
            name: "/",
            func: native_div,
        }),
    );
    env
}

/// Evaluates a single [`SrsValue`] expression in the given environment.
pub fn eval(expr: &SrsValue, env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match expr {
        SrsValue::Symbol(name) => env.get(name).ok_or_else(|| EvalError {
            kind: EvalErrorKind::UnboundVariable(name.clone()),
        }),
        SrsValue::Pair(_) => eval_combination(expr, env),
        // Self-evaluating literals.
        other => Ok(other.clone()),
    }
}

/// Evaluates a combination `(operator arg...)`, handling special forms
/// (currently only `define`) before falling back to procedure application.
fn eval_combination(expr: &SrsValue, env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    let items = list_to_vec(expr)?;

    if items.is_empty() {
        return Ok(SrsValue::Nil);
    }

    if let SrsValue::Symbol(op) = &items[0]
        && op == "define"
    {
        return eval_define(&items[1..], env);
    }

    let proc = eval(&items[0], env)?;
    let mut args = Vec::with_capacity(items.len() - 1);
    for arg in &items[1..] {
        args.push(eval(arg, env)?);
    }
    apply(&proc, &args)
}

/// Handles `(define <name> <expr>)`, evaluating the initializer once and
/// binding it in the current environment. Returns [`SrsValue::Unspecified`].
fn eval_define(args: &[SrsValue], env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match args {
        [name, value_expr] => {
            let name = match name {
                SrsValue::Symbol(s) => s.clone(),
                _ => return err(EvalErrorKind::WrongType),
            };
            let value = eval(value_expr, env)?;
            env.define(name, value);
            Ok(SrsValue::Unspecified)
        }
        [] | [_] => err(EvalErrorKind::NotEnoughArguments),
        _ => err(EvalErrorKind::TooManyArguments),
    }
}

/// Applies a callable [`SrsValue`] (currently only [`SrsValue::Native`]) to
/// already-evaluated arguments.
fn apply(proc: &SrsValue, args: &[SrsValue]) -> Result<SrsValue, EvalError> {
    match proc {
        SrsValue::Native(native) => (native.func)(args).map_err(|msg| EvalError {
            kind: EvalErrorKind::Native(msg),
        }),
        _ => err(EvalErrorKind::NotAProcedure),
    }
}

/// Converts a proper list (`SrsValue::Pair` chain ending in `SrsValue::Nil`)
/// into a `Vec`. Fails on improper (dotted) lists.
fn list_to_vec(list: &SrsValue) -> Result<Vec<SrsValue>, EvalError> {
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
            _ => return err(EvalErrorKind::WrongType),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretor::lexical_analyzer::get_lexemes;
    use crate::interpretor::reader::read_all;

    fn eval_src(src: &str) -> Result<SrsValue, EvalError> {
        let values = read_all(get_lexemes(src).unwrap()).unwrap();
        let env = global_env();
        let mut result = Ok(SrsValue::Unspecified);
        for value in &values {
            result = eval(value, &env);
            if result.is_err() {
                return result;
            }
        }
        result
    }

    fn ok(src: &str) -> SrsValue {
        eval_src(src).unwrap()
    }

    #[test]
    fn integer_is_self_evaluating() {
        assert!(matches!(ok("42"), SrsValue::Integer(42)));
    }

    #[test]
    fn add_two_integers() {
        assert!(matches!(ok("(+ 3 4)"), SrsValue::Integer(7)));
    }

    #[test]
    fn add_single_integer() {
        assert!(matches!(ok("(+ 3)"), SrsValue::Integer(3)));
    }

    #[test]
    fn add_no_args_returns_identity() {
        assert!(matches!(ok("(+)"), SrsValue::Integer(0)));
    }

    #[test]
    fn sub_two_integers() {
        assert!(matches!(ok("(- 5 2)"), SrsValue::Integer(3)));
    }

    #[test]
    fn sub_single_integer_negates() {
        assert!(matches!(ok("(- 5)"), SrsValue::Integer(-5)));
    }

    #[test]
    fn sub_no_args_fails() {
        assert!(eval_src("(-)").is_err());
    }

    #[test]
    fn multiply_several_integers() {
        assert!(matches!(ok("(* 2 3 4)"), SrsValue::Integer(24)));
    }

    #[test]
    fn multiply_single_integer() {
        assert!(matches!(ok("(* 4)"), SrsValue::Integer(4)));
    }

    #[test]
    fn multiply_no_args_returns_identity() {
        assert!(matches!(ok("(*)"), SrsValue::Integer(1)));
    }

    #[test]
    fn divide_two_integers() {
        assert!(matches!(ok("(/ 10 2)"), SrsValue::Integer(5)));
    }

    #[test]
    fn divide_by_zero_fails() {
        assert!(eval_src("(/ 1 0)").is_err());
    }

    #[test]
    fn nested_expression() {
        assert!(matches!(ok("(+ (* 2 3) 4)"), SrsValue::Integer(10)));
    }

    #[test]
    fn float_coercion() {
        match ok("(+ 1 2.5)") {
            SrsValue::Float(f) => assert!((f - 3.5).abs() < f64::EPSILON),
            other => panic!("expected float, got {:?}", other),
        }
    }

    #[test]
    fn define_returns_unspecified() {
        assert!(matches!(ok("(define x 42)"), SrsValue::Unspecified));
    }

    #[test]
    fn define_then_reread() {
        assert!(matches!(ok("(define x 42) x"), SrsValue::Integer(42)));
    }

    #[test]
    fn define_with_expression_initializer() {
        assert!(matches!(ok("(define x (+ 2 3)) x"), SrsValue::Integer(5)));
    }

    #[test]
    fn redefine_replaces_value() {
        assert!(matches!(
            ok("(define x 1) (define x 2) x"),
            SrsValue::Integer(2)
        ));
    }

    #[test]
    fn define_undefined_initializer_fails() {
        assert!(eval_src("(define x unknown)").is_err());
    }

    #[test]
    fn define_non_symbol_name_fails() {
        assert!(eval_src("(define 1 2)").is_err());
    }

    #[test]
    fn define_missing_initializer_fails() {
        assert!(eval_src("(define x)").is_err());
    }

    #[test]
    fn define_too_many_args_fails() {
        assert!(eval_src("(define x 1 2)").is_err());
    }

    #[test]
    fn unbound_variable_fails() {
        assert!(eval_src("unknown").is_err());
    }

    #[test]
    fn calling_a_non_procedure_fails() {
        assert!(eval_src("(1 2 3)").is_err());
    }
}
