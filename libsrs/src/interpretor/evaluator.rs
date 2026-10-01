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

pub use natives::{global_env, global_env_with_frontend};

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
    /// The Scheme `error` procedure was called.
    SchemeError(String),
    /// A request to terminate the program/REPL, e.g. from the R7RS
    /// extension `(exit [obj])`.
    Exit(i32),
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
            EvalErrorKind::SchemeError(msg) => write!(f, "{}", msg),
            EvalErrorKind::Exit(code) => write!(f, "exit requested with code {}", code),
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

/// An evaluation step either produces a value or schedules an expression in
/// tail position. `eval` consumes the latter in a loop instead of recursing.
pub(super) enum EvalControl {
    Value(SrsValue),
    Eval(SrsValue, Rc<Env>),
}

/// Evaluates a single [`SrsValue`] expression in the given environment.
pub fn eval(expr: &SrsValue, env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    let mut expr = expr.clone();
    let mut env = env.clone();
    loop {
        match &expr {
            SrsValue::Symbol(name) => {
                return env.get(name).ok_or_else(|| EvalError {
                    kind: EvalErrorKind::UnboundVariable(name.clone()),
                });
            }
            SrsValue::Pair(_) => match special_forms::eval_combination(&expr, &env)? {
                EvalControl::Value(value) => return Ok(value),
                EvalControl::Eval(next_expr, next_env) => {
                    expr = next_expr;
                    env = next_env;
                }
            },
            // Self-evaluating literals.
            other => return Ok(other.clone()),
        }
    }
}

/// Applies a callable [`SrsValue`] ([`SrsValue::Native`] or
/// [`SrsValue::Procedure`]) to already-evaluated arguments.
pub fn apply(proc: &SrsValue, args: &[SrsValue]) -> Result<SrsValue, EvalError> {
    match apply_tail(proc, args)? {
        EvalControl::Value(value) => Ok(value),
        EvalControl::Eval(expr, env) => eval(&expr, &env),
    }
}

/// Applies a procedure, scheduling a user procedure's body for evaluation in
/// tail position rather than recursing through the Rust stack.
pub(super) fn apply_tail(proc: &SrsValue, args: &[SrsValue]) -> Result<EvalControl, EvalError> {
    match proc {
        SrsValue::Native(native) if native.name == "apply" => {
            let (procedure, call_args) = expand_apply_arguments(args)?;
            apply_tail(procedure, &call_args)
        }
        SrsValue::Native(native) => {
            (native.func)(args)
                .map(EvalControl::Value)
                .map_err(|msg| EvalError {
                    kind: parse_native_error(&msg),
                })
        }
        SrsValue::Procedure(lambda) => apply_lambda(lambda, args),
        _ => err(EvalErrorKind::NotAProcedure),
    }
}

fn expand_apply_arguments(args: &[SrsValue]) -> Result<(&SrsValue, Vec<SrsValue>), EvalError> {
    let [procedure, rest @ ..] = args else {
        return err(EvalErrorKind::Native(
            "not enough arguments to apply".to_string(),
        ));
    };
    let Some((final_list, leading)) = rest.split_last() else {
        return err(EvalErrorKind::Native(
            "not enough arguments to apply".to_string(),
        ));
    };
    let mut call_args = leading.to_vec();
    call_args.extend(util::list_to_vec(final_list).map_err(|e| EvalError {
        kind: EvalErrorKind::Native(e.to_string()),
    })?);
    Ok((procedure, call_args))
}

/// Parses the error returned by a native procedure.
///
/// Some natives use a private marker encoding to transmit non-error
/// conditions such as a request to exit the program. This function turns
/// those markers into the appropriate [`EvalErrorKind`], everything else is
/// treated as a regular native error.
fn parse_native_error(msg: &str) -> EvalErrorKind {
    const PREFIX: &str = "\x1b__EXIT_MARKER__:";
    const SUFFIX: &str = "\x1b";
    const ERROR_PREFIX: &str = "\x1b__SCHEME_ERROR_MARKER__:";
    if let Some(body) = msg.strip_prefix(ERROR_PREFIX)
        && let Some(message) = body.strip_suffix(SUFFIX)
    {
        return EvalErrorKind::SchemeError(message.to_string());
    }
    if let Some(body) = msg.strip_prefix(PREFIX)
        && let Some(code_str) = body.strip_suffix(SUFFIX)
        && let Ok(code) = code_str.parse::<i32>()
    {
        return EvalErrorKind::Exit(code);
    }
    EvalErrorKind::Native(msg.to_string())
}

pub(crate) fn scheme_error_to_native(error: &EvalError) -> String {
    match &error.kind {
        EvalErrorKind::SchemeError(message) => {
            format!("\x1b__SCHEME_ERROR_MARKER__:{}\x1b", message)
        }
        _ => error.to_string(),
    }
}

/// Binds `args` to a lambda's parameters in a fresh child environment and
/// evaluates its body in sequence, returning the value of the last
/// expression (implicit `begin`).
fn apply_lambda(lambda: &Rc<Lambda>, args: &[SrsValue]) -> Result<EvalControl, EvalError> {
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

    let Some((last, body)) = lambda.body.split_last() else {
        return Ok(EvalControl::Value(SrsValue::Unspecified));
    };
    for expr in body {
        eval(expr, &call_env)?;
    }
    Ok(EvalControl::Eval(last.clone(), call_env))
}
