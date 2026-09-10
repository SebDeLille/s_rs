use std::cell::RefCell;
use std::rc::Rc;

use crate::types::core::{Env, Lambda, SrsValue};

use super::util::{is_truthy, list_to_vec};
use super::{EvalError, EvalErrorKind, apply, err, eval};

/// Evaluates a combination `(operator arg...)`, handling special forms
/// (currently only `define`) before falling back to procedure application.
pub(super) fn eval_combination(expr: &SrsValue, env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    let items = list_to_vec(expr)?;

    if items.is_empty() {
        return Ok(SrsValue::Nil);
    }

    if let SrsValue::Symbol(op) = &items[0] {
        match op.as_str() {
            "define" => return eval_define(&items[1..], env),
            "lambda" => return eval_lambda(&items[1..], env),
            "let" => return eval_let(&items[1..], env),
            "let*" => return eval_let_star(&items[1..], env),
            "do" => return eval_do(&items[1..], env),
            "if" => return eval_if(&items[1..], env),
            "quote" => return eval_quote(&items[1..]),
            "quasiquote" => return eval_quasiquote(&items[1..], env),
            "load" => return eval_load(&items[1..], env),
            _ => {}
        }
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

/// Handles `(lambda <params> <body>...)`, capturing the current environment
/// and producing a [`SrsValue::Procedure`].
fn eval_lambda(args: &[SrsValue], env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match args {
        [] => err(EvalErrorKind::NotEnoughArguments),
        [_] => err(EvalErrorKind::NotEnoughArguments),
        [params_spec, body @ ..] => {
            let (params, rest) = parse_params(params_spec)?;
            Ok(SrsValue::Procedure(Rc::new(Lambda {
                params,
                rest,
                body: body.to_vec(),
                env: env.clone(),
            })))
        }
    }
}

/// Handles `(let ((<name> <init>)...) <body>...)`, evaluating each `<init>`
/// in the enclosing environment, binding the results in a fresh child
/// environment, and evaluating the body in sequence (implicit `begin`).
fn eval_let(args: &[SrsValue], env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match args {
        [] => err(EvalErrorKind::NotEnoughArguments),
        [_] => err(EvalErrorKind::NotEnoughArguments),
        [bindings_spec, body @ ..] => {
            let bindings = list_to_vec(bindings_spec)?;
            let let_env = Env::new(Some(env.clone()));
            for binding in &bindings {
                let parts = list_to_vec(binding)?;
                match parts.as_slice() {
                    [name, init_expr] => {
                        let name = match name {
                            SrsValue::Symbol(s) => s.clone(),
                            _ => return err(EvalErrorKind::WrongType),
                        };
                        let value = eval(init_expr, env)?;
                        let_env.define(name, value);
                    }
                    _ => return err(EvalErrorKind::WrongType),
                }
            }

            let mut result = SrsValue::Unspecified;
            for expr in body {
                result = eval(expr, &let_env)?;
            }
            Ok(result)
        }
    }
}

/// Handles `(let* ((<name> <init>)...) <body>...)`, evaluating each
/// `<init>` in an environment that already includes the preceding
/// bindings (each binding gets its own nested environment, matching
/// R5RS semantics where later inits may refer to earlier names, and
/// duplicate names are allowed), then evaluating the body in sequence
/// (implicit `begin`) in the innermost environment.
fn eval_let_star(args: &[SrsValue], env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match args {
        [] => err(EvalErrorKind::NotEnoughArguments),
        [_] => err(EvalErrorKind::NotEnoughArguments),
        [bindings_spec, body @ ..] => {
            let bindings = list_to_vec(bindings_spec)?;
            let mut current_env = env.clone();
            for binding in &bindings {
                let parts = list_to_vec(binding)?;
                match parts.as_slice() {
                    [name, init_expr] => {
                        let name = match name {
                            SrsValue::Symbol(s) => s.clone(),
                            _ => return err(EvalErrorKind::WrongType),
                        };
                        let value = eval(init_expr, &current_env)?;
                        let next_env = Env::new(Some(current_env.clone()));
                        next_env.define(name, value);
                        current_env = next_env;
                    }
                    _ => return err(EvalErrorKind::WrongType),
                }
            }

            let body_env = Env::new(Some(current_env));
            let mut result = SrsValue::Unspecified;
            for expr in body {
                result = eval(expr, &body_env)?;
            }
            Ok(result)
        }
    }
}

/// Handles `(do ((<var> <init> [<step>])...) (<test> <expr>...) <command>...)`.
///
/// Each `<init>` is evaluated once in the enclosing environment and bound to
/// `<var>` in a fresh environment. On every iteration `<test>` is evaluated
/// first: if true, the result expressions are evaluated in sequence and
/// their last value (or [`SrsValue::Unspecified`] if there are none) is
/// returned. Otherwise the `<command>`s are evaluated for their side
/// effects, then all `<step>` expressions are evaluated in the current
/// iteration's environment *before* any variable is rebound, and their
/// values are bound simultaneously to `<var>` in a fresh environment for the
/// next iteration. A `<var>` without a `<step>` keeps its value across
/// iterations.
fn eval_do(args: &[SrsValue], env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match args {
        [] => err(EvalErrorKind::NotEnoughArguments),
        [_] => err(EvalErrorKind::NotEnoughArguments),
        [bindings_spec, test_spec, commands @ ..] => {
            let bindings = list_to_vec(bindings_spec)?;

            struct DoVar {
                name: String,
                step: Option<SrsValue>,
            }

            let mut vars = Vec::with_capacity(bindings.len());
            let mut do_env = Env::new(Some(env.clone()));

            for binding in &bindings {
                let parts = list_to_vec(binding)?;
                match parts.as_slice() {
                    [name, init_expr] => {
                        let name = match name {
                            SrsValue::Symbol(s) => s.clone(),
                            _ => return err(EvalErrorKind::WrongType),
                        };
                        let value = eval(init_expr, env)?;
                        do_env.define(name.clone(), value);
                        vars.push(DoVar { name, step: None });
                    }
                    [name, init_expr, step_expr] => {
                        let name = match name {
                            SrsValue::Symbol(s) => s.clone(),
                            _ => return err(EvalErrorKind::WrongType),
                        };
                        let value = eval(init_expr, env)?;
                        do_env.define(name.clone(), value);
                        vars.push(DoVar {
                            name,
                            step: Some(step_expr.clone()),
                        });
                    }
                    _ => return err(EvalErrorKind::WrongType),
                }
            }

            let test_parts = list_to_vec(test_spec)?;
            let (test_expr, result_exprs) = match test_parts.as_slice() {
                [] => return err(EvalErrorKind::NotEnoughArguments),
                [test, results @ ..] => (test.clone(), results.to_vec()),
            };

            loop {
                if is_truthy(&eval(&test_expr, &do_env)?) {
                    let mut result = SrsValue::Unspecified;
                    for expr in &result_exprs {
                        result = eval(expr, &do_env)?;
                    }
                    return Ok(result);
                }

                for command in commands {
                    eval(command, &do_env)?;
                }

                let mut next_values = Vec::with_capacity(vars.len());
                for var in &vars {
                    let value = match &var.step {
                        Some(step_expr) => eval(step_expr, &do_env)?,
                        None => do_env.get(&var.name).expect("do variable should be bound"),
                    };
                    next_values.push(value);
                }

                let next_env = Env::new(Some(env.clone()));
                for (var, value) in vars.iter().zip(next_values) {
                    next_env.define(var.name.clone(), value);
                }
                do_env = next_env;
            }
        }
    }
}

/// Handles `(if <test> <conseq> [<alt>])`. Per R5RS, any value other than
/// `#f` counts as true. Evaluates and returns `<conseq>` when `<test>` is
/// true, otherwise evaluates and returns `<alt>` if present, or
/// [`SrsValue::Unspecified`] when it is absent.
fn eval_if(args: &[SrsValue], env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match args {
        [test, conseq] => {
            if is_truthy(&eval(test, env)?) {
                eval(conseq, env)
            } else {
                Ok(SrsValue::Unspecified)
            }
        }
        [test, conseq, alt] => {
            if is_truthy(&eval(test, env)?) {
                eval(conseq, env)
            } else {
                eval(alt, env)
            }
        }
        [] | [_] => err(EvalErrorKind::NotEnoughArguments),
        _ => err(EvalErrorKind::TooManyArguments),
    }
}

/// Handles `(quote datum)`, returning `datum` unevaluated.
fn eval_quote(args: &[SrsValue]) -> Result<SrsValue, EvalError> {
    match args {
        [datum] => Ok(datum.clone()),
        [] => err(EvalErrorKind::NotEnoughArguments),
        _ => err(EvalErrorKind::TooManyArguments),
    }
}

/// Handles `(quasiquote datum)`, returning `datum` with any nested
/// `(unquote expr)` replaced by the evaluated `expr`, and any
/// `(unquote-splicing expr)` appearing in a list position spliced in.
/// Nested `quasiquote`s increase the nesting level so that only `unquote`
/// forms at the matching level are evaluated (R5RS section 4.2.6).
fn eval_quasiquote(args: &[SrsValue], env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    match args {
        [datum] => quasiquote(datum, env, 1),
        [] => err(EvalErrorKind::NotEnoughArguments),
        _ => err(EvalErrorKind::TooManyArguments),
    }
}

/// Handles `(load <filename>)`: reads the Scheme source file at
/// `<filename>` (a string, evaluated normally), reads all top-level
/// expressions from it, and evaluates them in sequence in the current
/// environment `env`. Returns the value of the last expression, or
/// [`SrsValue::Unspecified`] if the file is empty.
///
/// If `<filename>` contains a `/` or already ends with `.scm`, it is used as
/// a literal path. Otherwise it is treated as a bare library name and
/// resolved to `~/.config/srs/libs/<name>.scm`.
fn eval_load(args: &[SrsValue], env: &Rc<Env>) -> Result<SrsValue, EvalError> {
    let path_expr = match args {
        [path_expr] => path_expr,
        [] => return err(EvalErrorKind::NotEnoughArguments),
        _ => return err(EvalErrorKind::TooManyArguments),
    };

    let path_value = eval(path_expr, env)?;
    let path = match &path_value {
        SrsValue::String(s) => s.borrow().clone(),
        _ => return err(EvalErrorKind::WrongType),
    };

    let resolved = resolve_load_path(&path);

    let source = std::fs::read_to_string(&resolved).map_err(|e| EvalError {
        kind: EvalErrorKind::Native(format!(
            "load: could not read {}: {}",
            resolved.display(),
            e
        )),
    })?;

    let lexemes =
        crate::interpretor::lexical_analyzer::get_lexemes(&source).map_err(|e| EvalError {
            kind: EvalErrorKind::Native(format!("load: {}: {}", path, e)),
        })?;
    let exprs = crate::interpretor::reader::read_all(lexemes).map_err(|e| EvalError {
        kind: EvalErrorKind::Native(format!("load: {}: {}", path, e)),
    })?;

    let mut result = SrsValue::Unspecified;
    for expr in &exprs {
        result = eval(expr, env)?;
    }
    Ok(result)
}

/// Resolves a `load` argument into an actual filesystem path.
///
/// Literal paths (`/path/file.scm`, `./file.scm`, `foo/bar.scm`) are returned
/// unchanged. Bare library names (`csv`) are mapped to
/// `~/.config/srs/libs/<name>.scm`.
fn resolve_load_path(path: &str) -> std::path::PathBuf {
    resolve_load_path_with_home(path, std::env::var("HOME").ok().as_deref())
}

pub(crate) fn resolve_load_path_with_home(path: &str, home: Option<&str>) -> std::path::PathBuf {
    if path.contains('/') || path.ends_with(".scm") {
        std::path::PathBuf::from(path)
    } else if let Some(home) = home {
        std::path::PathBuf::from(home)
            .join(".config")
            .join("srs")
            .join("libs")
            .join(format!("{}.scm", path))
    } else {
        std::path::PathBuf::from(path)
    }
}

/// Recursively expands a quasiquoted template at the given nesting `depth`.
fn quasiquote(expr: &SrsValue, env: &Rc<Env>, depth: u32) -> Result<SrsValue, EvalError> {
    match expr {
        SrsValue::Pair(cell) => {
            let (car, cdr) = cell.borrow().clone();

            if let SrsValue::Symbol(name) = &car {
                if name == "unquote" {
                    let arg = extract_single(&cdr)?;
                    return if depth == 1 {
                        eval(&arg, env)
                    } else {
                        Ok(unary_form("unquote", quasiquote(&arg, env, depth - 1)?))
                    };
                }
                if name == "quasiquote" {
                    let arg = extract_single(&cdr)?;
                    return Ok(unary_form("quasiquote", quasiquote(&arg, env, depth + 1)?));
                }
            }

            if let SrsValue::Pair(car_cell) = &car {
                let (car_car, car_cdr) = car_cell.borrow().clone();
                if matches!(&car_car, SrsValue::Symbol(name) if name == "unquote-splicing") {
                    let arg = extract_single(&car_cdr)?;
                    return if depth == 1 {
                        let spliced = eval(&arg, env)?;
                        let rest = quasiquote(&cdr, env, depth)?;
                        append_list(spliced, rest)
                    } else {
                        let new_car =
                            unary_form("unquote-splicing", quasiquote(&arg, env, depth - 1)?);
                        let rest = quasiquote(&cdr, env, depth)?;
                        Ok(SrsValue::Pair(Rc::new(RefCell::new((new_car, rest)))))
                    };
                }
            }

            let new_car = quasiquote(&car, env, depth)?;
            let new_cdr = quasiquote(&cdr, env, depth)?;
            Ok(SrsValue::Pair(Rc::new(RefCell::new((new_car, new_cdr)))))
        }
        other => Ok(other.clone()),
    }
}

/// Extracts the single argument of a one-argument special form like
/// `(unquote x)`, i.e. expects `spec` to be `(x . ())`.
fn extract_single(spec: &SrsValue) -> Result<SrsValue, EvalError> {
    match spec {
        SrsValue::Pair(cell) => {
            let (arg, rest) = cell.borrow().clone();
            match rest {
                SrsValue::Nil => Ok(arg),
                _ => err(EvalErrorKind::WrongType),
            }
        }
        _ => err(EvalErrorKind::WrongType),
    }
}

/// Builds `(symbol value)`, e.g. `unary_form("unquote", x)` -> `(unquote x)`.
fn unary_form(symbol: &str, value: SrsValue) -> SrsValue {
    SrsValue::Pair(Rc::new(RefCell::new((
        SrsValue::Symbol(symbol.to_string()),
        SrsValue::Pair(Rc::new(RefCell::new((value, SrsValue::Nil)))),
    ))))
}

/// Appends a proper list `list` in front of `tail`, e.g. splicing
/// `unquote-splicing` results into a surrounding quasiquoted list.
fn append_list(list: SrsValue, tail: SrsValue) -> Result<SrsValue, EvalError> {
    let items = list_to_vec(&list)?;
    let mut result = tail;
    for item in items.into_iter().rev() {
        result = SrsValue::Pair(Rc::new(RefCell::new((item, result))));
    }
    Ok(result)
}

/// Parses a lambda parameter spec into a fixed parameter list and an
/// optional rest parameter name. Accepts a bare symbol (fully variadic), a
/// proper list of symbols, or a dotted list ending in a rest symbol.
fn parse_params(spec: &SrsValue) -> Result<(Vec<String>, Option<String>), EvalError> {
    match spec {
        SrsValue::Symbol(name) => Ok((Vec::new(), Some(name.clone()))),
        SrsValue::Nil => Ok((Vec::new(), None)),
        SrsValue::Pair(_) => {
            let mut params = Vec::new();
            let mut current = spec.clone();
            loop {
                match current {
                    SrsValue::Nil => return Ok((params, None)),
                    SrsValue::Symbol(name) => return Ok((params, Some(name))),
                    SrsValue::Pair(cell) => {
                        let (car, cdr) = cell.borrow().clone();
                        match car {
                            SrsValue::Symbol(name) => params.push(name),
                            _ => return err(EvalErrorKind::WrongType),
                        }
                        current = cdr;
                    }
                    _ => return err(EvalErrorKind::WrongType),
                }
            }
        }
        _ => err(EvalErrorKind::WrongType),
    }
}
