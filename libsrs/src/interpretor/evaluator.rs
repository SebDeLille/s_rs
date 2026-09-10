use std::fmt;
use std::rc::Rc;

use crate::types::core::{Env, Lambda, SrsValue};

mod natives;
mod special_forms;
#[cfg(test)]
mod tests;
mod util;

#[cfg(test)]
pub(crate) use special_forms::resolve_load_path_with_home;

pub use natives::global_env;

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

pub(crate) fn err<T>(kind: EvalErrorKind) -> Result<T, EvalError> {
    Err(EvalError { kind })
}

/// Evaluates a single [`SrsValue`] expression in the given environment.
pub fn eval(expr: &SrsValue, env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match expr {
        SrsValue::Symbol(name) => env.get(name).ok_or_else(|| EvalError {
            kind: EvalErrorKind::UnboundVariable(name.clone()),
        }),
        SrsValue::Pair(_) => special_forms::eval_combination(expr, env),
        // Self-evaluating literals.
        other => Ok(other.clone()),
    }
}

/// Applies a callable [`SrsValue`] ([`SrsValue::Native`] or
/// [`SrsValue::Procedure`]) to already-evaluated arguments.
pub fn apply(proc: &SrsValue, args: &[SrsValue]) -> Result<SrsValue, EvalError> {
    match proc {
        SrsValue::Native(native) => (native.func)(args).map_err(|msg| EvalError {
            kind: EvalErrorKind::Native(msg),
        }),
        SrsValue::Procedure(lambda) => apply_lambda(lambda, args),
        _ => err(EvalErrorKind::NotAProcedure),
    }
}

/// Binds `args` to a lambda's parameters in a fresh child environment and
/// evaluates its body in sequence, returning the value of the last
/// expression (implicit `begin`).
fn apply_lambda(lambda: &Rc<Lambda>, args: &[SrsValue]) -> Result<SrsValue, EvalError> {
    if args.len() < lambda.params.len()
        || (lambda.rest.is_none() && args.len() > lambda.params.len())
    {
        return if args.len() < lambda.params.len() {
            err(EvalErrorKind::NotEnoughArguments)
        } else {
            err(EvalErrorKind::TooManyArguments)
        };
    }

    let call_env = Env::new(Some(lambda.env.clone()));
    for (name, value) in lambda.params.iter().zip(args.iter()) {
        call_env.define(name.clone(), value.clone());
    }
    if let Some(rest_name) = &lambda.rest {
        let rest_values = args[lambda.params.len()..].to_vec();
        call_env.define(rest_name.clone(), util::vec_to_list(rest_values));
    }

    let mut result = SrsValue::Unspecified;
    for expr in &lambda.body {
        result = eval(expr, &call_env)?;
    }
    Ok(result)
}
