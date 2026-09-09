use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::types::core::{Env, Lambda, Native, SrsValue};

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
    env.define(
        "=".to_string(),
        SrsValue::Native(Native {
            name: "=",
            func: native_num_eq,
        }),
    );
    env.define(
        "<".to_string(),
        SrsValue::Native(Native {
            name: "<",
            func: native_lt,
        }),
    );
    env.define(
        ">".to_string(),
        SrsValue::Native(Native {
            name: ">",
            func: native_gt,
        }),
    );
    env.define(
        "<=".to_string(),
        SrsValue::Native(Native {
            name: "<=",
            func: native_le,
        }),
    );
    env.define(
        ">=".to_string(),
        SrsValue::Native(Native {
            name: ">=",
            func: native_ge,
        }),
    );
    env.define(
        "not".to_string(),
        SrsValue::Native(Native {
            name: "not",
            func: native_not,
        }),
    );
    env.define(
        "sin".to_string(),
        SrsValue::Native(Native {
            name: "sin",
            func: native_sin,
        }),
    );
    env.define(
        "cos".to_string(),
        SrsValue::Native(Native {
            name: "cos",
            func: native_cos,
        }),
    );
    env.define(
        "tan".to_string(),
        SrsValue::Native(Native {
            name: "tan",
            func: native_tan,
        }),
    );
    env.define(
        "atan".to_string(),
        SrsValue::Native(Native {
            name: "atan",
            func: native_atan,
        }),
    );
    env.define(
        "cons".to_string(),
        SrsValue::Native(Native {
            name: "cons",
            func: native_cons,
        }),
    );
    env.define(
        "car".to_string(),
        SrsValue::Native(Native {
            name: "car",
            func: native_car,
        }),
    );
    env.define(
        "cdr".to_string(),
        SrsValue::Native(Native {
            name: "cdr",
            func: native_cdr,
        }),
    );
    env.define(
        "apply".to_string(),
        SrsValue::Native(Native {
            name: "apply",
            func: native_apply,
        }),
    );
    env.define(
        "map".to_string(),
        SrsValue::Native(Native {
            name: "map",
            func: native_map,
        }),
    );
    env.define(
        "vector".to_string(),
        SrsValue::Native(Native {
            name: "vector",
            func: native_vector,
        }),
    );
    env.define(
        "make-vector".to_string(),
        SrsValue::Native(Native {
            name: "make-vector",
            func: native_make_vector,
        }),
    );
    env.define(
        "vector?".to_string(),
        SrsValue::Native(Native {
            name: "vector?",
            func: native_vector_p,
        }),
    );
    env.define(
        "vector-length".to_string(),
        SrsValue::Native(Native {
            name: "vector-length",
            func: native_vector_length,
        }),
    );
    env.define(
        "vector-ref".to_string(),
        SrsValue::Native(Native {
            name: "vector-ref",
            func: native_vector_ref,
        }),
    );
    env.define(
        "vector-set!".to_string(),
        SrsValue::Native(Native {
            name: "vector-set!",
            func: native_vector_set,
        }),
    );
    env.define(
        "vector->list".to_string(),
        SrsValue::Native(Native {
            name: "vector->list",
            func: native_vector_to_list,
        }),
    );
    env.define(
        "list->vector".to_string(),
        SrsValue::Native(Native {
            name: "list->vector",
            func: native_list_to_vector,
        }),
    );
    env.define(
        "vector-fill!".to_string(),
        SrsValue::Native(Native {
            name: "vector-fill!",
            func: native_vector_fill,
        }),
    );
    env.define(
        "display".to_string(),
        SrsValue::Native(Native {
            name: "display",
            func: native_display,
        }),
    );
    env.define(
        "newline".to_string(),
        SrsValue::Native(Native {
            name: "newline",
            func: native_newline,
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

    if let SrsValue::Symbol(op) = &items[0] {
        match op.as_str() {
            "define" => return eval_define(&items[1..], env),
            "lambda" => return eval_lambda(&items[1..], env),
            "let" => return eval_let(&items[1..], env),
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
                        None => do_env
                            .get(&var.name)
                            .expect("do variable should be bound"),
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

    let source = std::fs::read_to_string(&path).map_err(|e| EvalError {
        kind: EvalErrorKind::Native(format!("load: could not read {}: {}", path, e)),
    })?;

    let lexemes = crate::interpretor::lexical_analyzer::get_lexemes(&source).map_err(|e| {
        EvalError {
            kind: EvalErrorKind::Native(format!("load: {}: {}", path, e)),
        }
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

/// R5RS truthiness: every value is true except `#f`.
fn is_truthy(value: &SrsValue) -> bool {
    !matches!(value, SrsValue::Boolean(false))
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

/// Applies a callable [`SrsValue`] ([`SrsValue::Native`] or
/// [`SrsValue::Procedure`]) to already-evaluated arguments.
fn apply(proc: &SrsValue, args: &[SrsValue]) -> Result<SrsValue, EvalError> {
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
        call_env.define(rest_name.clone(), vec_to_list(rest_values));
    }

    let mut result = SrsValue::Unspecified;
    for expr in &lambda.body {
        result = eval(expr, &call_env)?;
    }
    Ok(result)
}

/// Converts a `Vec<SrsValue>` into a proper list (`Pair` chain ending in
/// `Nil`).
fn vec_to_list(values: Vec<SrsValue>) -> SrsValue {
    values.into_iter().rev().fold(SrsValue::Nil, |acc, v| {
        SrsValue::Pair(Rc::new(RefCell::new((v, acc))))
    })
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

/// `(not obj)`: `#t` if `obj` is `#f`, `#f` for any other value (per R5RS,
/// every value counts as true except `#f`).
fn native_not(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [only] => Ok(SrsValue::Boolean(matches!(only, SrsValue::Boolean(false)))),
        [] => Err("not enough arguments to not".to_string()),
        _ => Err("too many arguments to not".to_string()),
    }
}

fn native_sin(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to sin".to_string()),
        [only] => numeric_to_f64(only).map(|n| SrsValue::Float(n.sin())),
        _ => Err("too many arguments to sin".to_string()),
    }
}

fn native_cos(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to cos".to_string()),
        [only] => numeric_to_f64(only).map(|n| SrsValue::Float(n.cos())),
        _ => Err("too many arguments to cos".to_string()),
    }
}

fn native_tan(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to tan".to_string()),
        [only] => numeric_to_f64(only).map(|n| SrsValue::Float(n.tan())),
        _ => Err("too many arguments to tan".to_string()),
    }
}

fn native_atan(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to atan".to_string()),
        [only] => numeric_to_f64(only).map(|n| SrsValue::Float(n.atan())),
        _ => Err("too many arguments to atan".to_string()),
    }
}

/// `(cons car cdr)`: allocates a fresh mutable pair.
fn native_cons(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [car, cdr] => Ok(SrsValue::Pair(Rc::new(RefCell::new((
            car.clone(),
            cdr.clone(),
        ))))),
        [] | [_] => Err("not enough arguments to cons".to_string()),
        _ => Err("too many arguments to cons".to_string()),
    }
}

/// `(car pair)`: returns the first element of a pair.
/// `(display value)`: writes `value` to standard output using its
/// human-readable representation (no quotes around strings, characters
/// printed as themselves). Returns [`SrsValue::Unspecified`].
fn native_display(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => {
            use std::io::Write;
            print!("{}", value.display_repr());
            std::io::stdout()
                .flush()
                .map_err(|e| format!("display: {}", e))?;
            Ok(SrsValue::Unspecified)
        }
        [] => Err("not enough arguments to display".to_string()),
        _ => Err("too many arguments to display".to_string()),
    }
}

/// `(newline)`: writes a line break to standard output. Returns
/// [`SrsValue::Unspecified`].
fn native_newline(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => {
            use std::io::Write;
            println!();
            std::io::stdout()
                .flush()
                .map_err(|e| format!("newline: {}", e))?;
            Ok(SrsValue::Unspecified)
        }
        _ => Err("too many arguments to newline".to_string()),
    }
}

fn native_car(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Pair(cell)] => Ok(cell.borrow().0.clone()),
        [_] => Err("wrong type: expected pair".to_string()),
        [] => Err("not enough arguments to car".to_string()),
        _ => Err("too many arguments to car".to_string()),
    }
}

/// `(cdr pair)`: returns the second element of a pair.
fn native_cdr(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Pair(cell)] => Ok(cell.borrow().1.clone()),
        [_] => Err("wrong type: expected pair".to_string()),
        [] => Err("not enough arguments to cdr".to_string()),
        _ => Err("too many arguments to cdr".to_string()),
    }
}

/// `(apply proc arg1 ... args)`: applies `proc` to a list of arguments
/// formed by prepending `arg1 ...` (if any) onto the final `args` list,
/// which must itself be a proper list (R5RS section 6.4).
fn native_apply(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to apply".to_string()),
        [_] => Err("not enough arguments to apply".to_string()),
        [proc, rest @ ..] => {
            let (leading, last) = rest.split_at(rest.len() - 1);
            let mut call_args = leading.to_vec();
            call_args.extend(list_to_vec(&last[0]).map_err(|e| e.to_string())?);
            apply(proc, &call_args).map_err(|e| e.to_string())
        }
    }
}

/// `(map proc list1 list2 ...)`: applies `proc` element-wise to one or more
/// lists, returning a new list of the results. Traversal stops as soon as
/// the shortest list is exhausted (R5RS section 6.4).
fn native_map(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to map".to_string()),
        [_] => Err("not enough arguments to map".to_string()),
        [proc, lists @ ..] => {
            let mut vecs = Vec::with_capacity(lists.len());
            for list in lists {
                vecs.push(list_to_vec(list).map_err(|e| e.to_string())?);
            }
            let len = vecs.iter().map(|v| v.len()).min().unwrap_or(0);
            let mut results = Vec::with_capacity(len);
            for i in 0..len {
                let call_args: Vec<SrsValue> = vecs.iter().map(|v| v[i].clone()).collect();
                results.push(apply(proc, &call_args).map_err(|e| e.to_string())?);
            }
            Ok(vec_to_list(results))
        }
    }
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

/// `(vector obj ...)`: allocates a fresh mutable vector containing `obj ...`.
fn native_vector(args: &[SrsValue]) -> Result<SrsValue, String> {
    Ok(SrsValue::Vector(Rc::new(RefCell::new(args.to_vec()))))
}

/// `(make-vector k [fill])`: allocates a fresh vector of `k` elements,
/// initialized to `fill` if given, or `0` otherwise.
fn native_make_vector(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err("not enough arguments to make-vector".to_string()),
        [k] => {
            let len = index_from(k, "make-vector")?;
            Ok(SrsValue::Vector(Rc::new(RefCell::new(vec![
                SrsValue::Integer(0);
                len
            ]))))
        }
        [k, fill] => {
            let len = index_from(k, "make-vector")?;
            Ok(SrsValue::Vector(Rc::new(RefCell::new(vec![
                fill.clone();
                len
            ]))))
        }
        _ => Err("too many arguments to make-vector".to_string()),
    }
}

/// `(vector? obj)`: `#t` if `obj` is a vector, `#f` otherwise.
fn native_vector_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [only] => Ok(SrsValue::Boolean(matches!(only, SrsValue::Vector(_)))),
        [] => Err("not enough arguments to vector?".to_string()),
        _ => Err("too many arguments to vector?".to_string()),
    }
}

/// `(vector-length vector)`: returns the number of elements in `vector`.
fn native_vector_length(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell)] => Ok(SrsValue::Integer(cell.borrow().len() as i64)),
        [_] => Err("wrong type: expected vector".to_string()),
        [] => Err("not enough arguments to vector-length".to_string()),
        _ => Err("too many arguments to vector-length".to_string()),
    }
}

/// `(vector-ref vector k)`: returns the `k`-th element of `vector`.
fn native_vector_ref(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell), k] => {
            let index = index_from(k, "vector-ref")?;
            cell.borrow()
                .get(index)
                .cloned()
                .ok_or_else(|| "vector-ref: index out of range".to_string())
        }
        [_, _] => Err("wrong type: expected vector".to_string()),
        [] | [_] => Err("not enough arguments to vector-ref".to_string()),
        _ => Err("too many arguments to vector-ref".to_string()),
    }
}

/// `(vector-set! vector k obj)`: stores `obj` at index `k` of `vector`.
fn native_vector_set(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell), k, obj] => {
            let index = index_from(k, "vector-set!")?;
            let mut vec = cell.borrow_mut();
            if index >= vec.len() {
                return Err("vector-set!: index out of range".to_string());
            }
            vec[index] = obj.clone();
            Ok(SrsValue::Unspecified)
        }
        [_, _, _] => Err("wrong type: expected vector".to_string()),
        [] | [_] | [_, _] => Err("not enough arguments to vector-set!".to_string()),
        _ => Err("too many arguments to vector-set!".to_string()),
    }
}

/// `(vector->list vector)`: returns a new list with the same elements as
/// `vector`.
fn native_vector_to_list(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell)] => Ok(vec_to_list(cell.borrow().clone())),
        [_] => Err("wrong type: expected vector".to_string()),
        [] => Err("not enough arguments to vector->list".to_string()),
        _ => Err("too many arguments to vector->list".to_string()),
    }
}

/// `(list->vector list)`: returns a new vector with the same elements as
/// `list`, which must be a proper list.
fn native_list_to_vector(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [list] => {
            let items = list_to_vec(list).map_err(|e| e.to_string())?;
            Ok(SrsValue::Vector(Rc::new(RefCell::new(items))))
        }
        [] => Err("not enough arguments to list->vector".to_string()),
        _ => Err("too many arguments to list->vector".to_string()),
    }
}

/// `(vector-fill! vector fill)`: stores `fill` in every element of
/// `vector`.
fn native_vector_fill(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Vector(cell), fill] => {
            let mut vec = cell.borrow_mut();
            for item in vec.iter_mut() {
                *item = fill.clone();
            }
            Ok(SrsValue::Unspecified)
        }
        [_, _] => Err("wrong type: expected vector".to_string()),
        [] | [_] => Err("not enough arguments to vector-fill!".to_string()),
        _ => Err("too many arguments to vector-fill!".to_string()),
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
    fn sin_of_zero() {
        match ok("(sin 0)") {
            SrsValue::Float(f) => assert!((f - 0.0).abs() < f64::EPSILON),
            other => panic!("expected float, got {:?}", other),
        }
    }

    #[test]
    fn cos_of_zero() {
        match ok("(cos 0)") {
            SrsValue::Float(f) => assert!((f - 1.0).abs() < f64::EPSILON),
            other => panic!("expected float, got {:?}", other),
        }
    }

    #[test]
    fn tan_of_zero() {
        match ok("(tan 0)") {
            SrsValue::Float(f) => assert!((f - 0.0).abs() < f64::EPSILON),
            other => panic!("expected float, got {:?}", other),
        }
    }

    #[test]
    fn atan_of_zero() {
        match ok("(atan 0)") {
            SrsValue::Float(f) => assert!((f - 0.0).abs() < f64::EPSILON),
            other => panic!("expected float, got {:?}", other),
        }
    }

    #[test]
    fn sin_of_one_matches_std_computation() {
        match ok("(sin 1)") {
            SrsValue::Float(f) => assert!((f - 1f64.sin()).abs() < 1e-9),
            other => panic!("expected float, got {:?}", other),
        }
    }

    #[test]
    fn trig_accepts_float_argument() {
        match ok("(cos 0.0)") {
            SrsValue::Float(f) => assert!((f - 1.0).abs() < f64::EPSILON),
            other => panic!("expected float, got {:?}", other),
        }
    }

    #[test]
    fn sin_no_args_fails() {
        assert!(eval_src("(sin)").is_err());
    }

    #[test]
    fn sin_too_many_args_fails() {
        assert!(eval_src("(sin 1 2)").is_err());
    }

    #[test]
    fn sin_wrong_type_fails() {
        assert!(eval_src("(sin \"a\")").is_err());
    }

    #[test]
    fn num_eq_true() {
        assert!(matches!(ok("(= 1 1 1)"), SrsValue::Boolean(true)));
    }

    #[test]
    fn num_eq_false() {
        assert!(matches!(ok("(= 1 1 2)"), SrsValue::Boolean(false)));
    }

    #[test]
    fn num_eq_int_float_mix() {
        assert!(matches!(ok("(= 1 1.0)"), SrsValue::Boolean(true)));
    }

    #[test]
    fn lt_increasing() {
        assert!(matches!(ok("(< 1 2 3)"), SrsValue::Boolean(true)));
    }

    #[test]
    fn lt_not_increasing() {
        assert!(matches!(ok("(< 1 3 2)"), SrsValue::Boolean(false)));
    }

    #[test]
    fn gt_decreasing() {
        assert!(matches!(ok("(> 3 2 1)"), SrsValue::Boolean(true)));
    }

    #[test]
    fn gt_not_decreasing() {
        assert!(matches!(ok("(> 3 1 2)"), SrsValue::Boolean(false)));
    }

    #[test]
    fn le_allows_equal() {
        assert!(matches!(ok("(<= 1 1 2)"), SrsValue::Boolean(true)));
    }

    #[test]
    fn le_rejects_decrease() {
        assert!(matches!(ok("(<= 1 2 1)"), SrsValue::Boolean(false)));
    }

    #[test]
    fn ge_allows_equal() {
        assert!(matches!(ok("(>= 2 2 1)"), SrsValue::Boolean(true)));
    }

    #[test]
    fn ge_rejects_increase() {
        assert!(matches!(ok("(>= 2 1 2)"), SrsValue::Boolean(false)));
    }

    #[test]
    fn compare_single_arg_is_true() {
        assert!(matches!(ok("(< 1)"), SrsValue::Boolean(true)));
        assert!(matches!(ok("(= 1)"), SrsValue::Boolean(true)));
    }

    #[test]
    fn compare_no_args_fails() {
        assert!(eval_src("(=)").is_err());
        assert!(eval_src("(<)").is_err());
    }

    #[test]
    fn compare_wrong_type_fails() {
        assert!(eval_src("(< 1 \"a\")").is_err());
    }

    #[test]
    fn not_false_is_true() {
        assert!(matches!(ok("(not #f)"), SrsValue::Boolean(true)));
    }

    #[test]
    fn not_true_is_false() {
        assert!(matches!(ok("(not #t)"), SrsValue::Boolean(false)));
    }

    #[test]
    fn not_non_boolean_is_false() {
        assert!(matches!(ok("(not 0)"), SrsValue::Boolean(false)));
        assert!(matches!(ok("(not \"a\")"), SrsValue::Boolean(false)));
    }

    #[test]
    fn not_no_args_fails() {
        assert!(eval_src("(not)").is_err());
    }

    #[test]
    fn not_too_many_args_fails() {
        assert!(eval_src("(not #t #f)").is_err());
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

    #[test]
    fn lambda_produces_a_procedure() {
        assert!(matches!(ok("(lambda (x) x)"), SrsValue::Procedure(_)));
    }

    #[test]
    fn calling_a_lambda_applies_it() {
        assert!(matches!(
            ok("((lambda (x) (+ x 1)) 41)"),
            SrsValue::Integer(42)
        ));
    }

    #[test]
    fn define_then_call_lambda() {
        assert!(matches!(
            ok("(define square (lambda (x) (* x x))) (square 5)"),
            SrsValue::Integer(25)
        ));
    }

    #[test]
    fn lambda_multiple_params() {
        assert!(matches!(
            ok("((lambda (a b c) (+ a b c)) 1 2 3)"),
            SrsValue::Integer(6)
        ));
    }

    #[test]
    fn lambda_body_evaluates_in_sequence() {
        assert!(matches!(
            ok("(define x 0) ((lambda () (define x 1) (define x 2) x))"),
            SrsValue::Integer(2)
        ));
    }

    #[test]
    fn lambda_closes_over_environment() {
        assert!(matches!(
            ok("(define make-adder (lambda (n) (lambda (x) (+ x n)))) \
                (define add5 (make-adder 5)) (add5 10)"),
            SrsValue::Integer(15)
        ));
    }

    #[test]
    fn lambda_variadic_all_args() {
        assert!(matches!(
            ok("((lambda args args) 1 2 3)"),
            SrsValue::Pair(_)
        ));
    }

    #[test]
    fn lambda_dotted_rest_param() {
        assert!(matches!(
            ok("((lambda (a . rest) rest) 1 2 3)"),
            SrsValue::Pair(_)
        ));
    }

    #[test]
    fn lambda_dotted_rest_param_empty() {
        assert!(matches!(ok("((lambda (a . rest) rest) 1)"), SrsValue::Nil));
    }

    #[test]
    fn lambda_no_params() {
        assert!(matches!(ok("((lambda () 42))"), SrsValue::Integer(42)));
    }

    #[test]
    fn lambda_too_few_args_fails() {
        assert!(eval_src("((lambda (a b) a) 1)").is_err());
    }

    #[test]
    fn lambda_too_many_args_fails() {
        assert!(eval_src("((lambda (a) a) 1 2)").is_err());
    }

    #[test]
    fn lambda_missing_body_fails() {
        assert!(eval_src("(lambda (x))").is_err());
    }

    #[test]
    fn lambda_non_symbol_param_fails() {
        assert!(eval_src("(lambda (1) 1)").is_err());
    }

    #[test]
    fn if_true_branch() {
        assert!(matches!(ok("(if #t 1 2)"), SrsValue::Integer(1)));
    }

    #[test]
    fn if_false_branch() {
        assert!(matches!(ok("(if #f 1 2)"), SrsValue::Integer(2)));
    }

    #[test]
    fn if_without_alt_when_true() {
        assert!(matches!(ok("(if #t 42)"), SrsValue::Integer(42)));
    }

    #[test]
    fn if_without_alt_when_false_is_unspecified() {
        assert!(matches!(ok("(if #f 42)"), SrsValue::Unspecified));
    }

    #[test]
    fn if_treats_non_boolean_as_true() {
        assert!(matches!(ok("(if 0 1 2)"), SrsValue::Integer(1)));
    }

    #[test]
    fn if_only_evaluates_taken_branch() {
        assert!(matches!(ok("(if #t 1 (/ 1 0))"), SrsValue::Integer(1)));
        assert!(matches!(ok("(if #f (/ 1 0) 2)"), SrsValue::Integer(2)));
    }

    #[test]
    fn if_too_few_args_fails() {
        assert!(eval_src("(if #t)").is_err());
    }

    #[test]
    fn if_too_many_args_fails() {
        assert!(eval_src("(if #t 1 2 3)").is_err());
    }

    #[test]
    fn do_sums_from_zero_to_nine() {
        let src = "(do ((i 0 (+ i 1))
                         (sum 0 (+ sum i)))
                        ((= i 10) sum))";
        assert!(matches!(ok(src), SrsValue::Integer(45)));
    }

    #[test]
    fn do_without_step_keeps_value() {
        let src = "(do ((x 1) (i 0 (+ i 1)))
                        ((= i 3) x))";
        assert!(matches!(ok(src), SrsValue::Integer(1)));
    }

    #[test]
    fn do_steps_are_evaluated_simultaneously() {
        // `last`'s step reads the *previous* value of `i`, proving steps are
        // all evaluated before any variable is rebound.
        let src = "(do ((i 0 (+ i 1))
                         (last -1 i))
                        ((= i 3) last))";
        assert!(matches!(ok(src), SrsValue::Integer(2)));
    }

    #[test]
    fn do_with_no_result_exprs_is_unspecified() {
        assert!(matches!(
            ok("(do ((i 0 (+ i 1))) ((= i 3)))"),
            SrsValue::Unspecified
        ));
    }

    #[test]
    fn do_test_only_runs_once_when_immediately_true() {
        assert!(matches!(ok("(do ((i 0)) (#t i))"), SrsValue::Integer(0)));
    }

    #[test]
    fn do_too_few_args_fails() {
        assert!(eval_src("(do ((i 0)))").is_err());
    }

    #[test]
    fn display_returns_unspecified() {
        assert!(matches!(ok("(display 42)"), SrsValue::Unspecified));
    }

    #[test]
    fn display_of_a_sequence_evaluates_all() {
        // Chains a couple of displays followed by a normal expression to
        // make sure `display` doesn't disturb subsequent evaluation.
        assert!(matches!(
            ok("(display \"a\") (display 1) (+ 1 2)"),
            SrsValue::Integer(3)
        ));
    }

    #[test]
    fn display_no_args_fails() {
        assert!(eval_src("(display)").is_err());
    }

    #[test]
    fn display_too_many_args_fails() {
        assert!(eval_src("(display 1 2)").is_err());
    }

    #[test]
    fn newline_returns_unspecified() {
        assert!(matches!(ok("(newline)"), SrsValue::Unspecified));
    }

    #[test]
    fn newline_too_many_args_fails() {
        assert!(eval_src("(newline 1)").is_err());
    }

    #[test]
    fn cons_creates_a_pair() {
        assert!(matches!(ok("(cons 1 2)"), SrsValue::Pair(_)));
    }

    #[test]
    fn car_of_cons() {
        assert!(matches!(ok("(car (cons 1 2))"), SrsValue::Integer(1)));
    }

    #[test]
    fn cdr_of_cons() {
        assert!(matches!(ok("(cdr (cons 1 2))"), SrsValue::Integer(2)));
    }

    #[test]
    fn car_of_nested_cons() {
        assert!(matches!(ok("(car (cons (cons 1 2) 3))"), SrsValue::Pair(_)));
    }

    #[test]
    fn cdr_of_nested_cons_is_a_pair() {
        assert!(matches!(
            ok("(cdr (cons 1 (cons 2 (cons 3 (cons 4 4)))))"),
            SrsValue::Pair(_)
        ));
    }

    #[test]
    fn cons_no_args_fails() {
        assert!(eval_src("(cons)").is_err());
    }

    #[test]
    fn cons_one_arg_fails() {
        assert!(eval_src("(cons 1)").is_err());
    }

    #[test]
    fn cons_too_many_args_fails() {
        assert!(eval_src("(cons 1 2 3)").is_err());
    }

    #[test]
    fn car_no_args_fails() {
        assert!(eval_src("(car)").is_err());
    }

    #[test]
    fn car_too_many_args_fails() {
        assert!(eval_src("(car (cons 1 2) (cons 3 4))").is_err());
    }

    #[test]
    fn car_wrong_type_fails() {
        assert!(eval_src("(car 1)").is_err());
    }

    #[test]
    fn cdr_no_args_fails() {
        assert!(eval_src("(cdr)").is_err());
    }

    #[test]
    fn cdr_too_many_args_fails() {
        assert!(eval_src("(cdr (cons 1 2) (cons 3 4))").is_err());
    }

    #[test]
    fn cdr_wrong_type_fails() {
        assert!(eval_src("(cdr 1)").is_err());
    }

    fn pair_car(value: &SrsValue) -> SrsValue {
        match value {
            SrsValue::Pair(cell) => cell.borrow().0.clone(),
            other => panic!("expected pair, got {:?}", other),
        }
    }

    fn pair_cdr(value: &SrsValue) -> SrsValue {
        match value {
            SrsValue::Pair(cell) => cell.borrow().1.clone(),
            other => panic!("expected pair, got {:?}", other),
        }
    }

    #[test]
    fn quote_returns_symbol_unevaluated() {
        assert!(matches!(ok("(quote x)"), SrsValue::Symbol(s) if s == "x"));
    }

    #[test]
    fn quote_shorthand_returns_symbol_unevaluated() {
        assert!(matches!(ok("'x"), SrsValue::Symbol(s) if s == "x"));
    }

    #[test]
    fn quote_returns_list_unevaluated() {
        let v = ok("'(1 2 3)");
        assert!(matches!(pair_car(&v), SrsValue::Integer(1)));
        let rest = pair_cdr(&v);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(2)));
    }

    #[test]
    fn quote_does_not_evaluate_operator_looking_symbol() {
        // '(+ 1 2) must stay the list (+ 1 2), not evaluate to 3.
        let v = ok("'(+ 1 2)");
        assert!(matches!(pair_car(&v), SrsValue::Symbol(s) if s == "+"));
    }

    #[test]
    fn quote_no_args_fails() {
        assert!(eval_src("(quote)").is_err());
    }

    #[test]
    fn quote_too_many_args_fails() {
        assert!(eval_src("(quote 1 2)").is_err());
    }

    #[test]
    fn quasiquote_without_unquote_behaves_like_quote() {
        let v = ok("`(1 2 3)");
        assert!(matches!(pair_car(&v), SrsValue::Integer(1)));
    }

    #[test]
    fn quasiquote_evaluates_unquoted_expr() {
        let v = ok("(define x 5) `(a ,x c)");
        let rest = pair_cdr(&v);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(5)));
    }

    #[test]
    fn quasiquote_evaluates_unquoted_computation() {
        let v = ok("`(1 ,(+ 1 1) 3)");
        let rest = pair_cdr(&v);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(2)));
    }

    #[test]
    fn quasiquote_splices_unquote_splicing() {
        let v = ok("(define xs (cons 2 (cons 3 '()))) `(1 ,@xs 4)");
        assert!(matches!(pair_car(&v), SrsValue::Integer(1)));
        let rest = pair_cdr(&v);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(2)));
        let rest = pair_cdr(&rest);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(3)));
        let rest = pair_cdr(&rest);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(4)));
        assert!(matches!(pair_cdr(&rest), SrsValue::Nil));
    }

    #[test]
    fn quasiquote_shorthand_matches_form() {
        let v1 = ok("`(a ,(+ 1 2))");
        let v2 = ok("(quasiquote (a (unquote (+ 1 2))))");
        assert!(matches!(pair_car(&v1), SrsValue::Symbol(s) if s == "a"));
        assert!(matches!(pair_car(&v2), SrsValue::Symbol(s) if s == "a"));
        assert!(matches!(pair_car(&pair_cdr(&v1)), SrsValue::Integer(3)));
        assert!(matches!(pair_car(&pair_cdr(&v2)), SrsValue::Integer(3)));
    }

    #[test]
    fn nested_quasiquote_defers_inner_unquote() {
        // At depth 2, the inner `,x` is not evaluated; the whole thing stays
        // quoted data: (quasiquote (unquote x)).
        let v = ok("(define x 1) `(a `(b ,x))");
        let inner = pair_car(&pair_cdr(&v));
        assert!(matches!(pair_car(&inner), SrsValue::Symbol(s) if s == "quasiquote"));
    }

    #[test]
    fn quasiquote_no_args_fails() {
        assert!(eval_src("(quasiquote)").is_err());
    }

    #[test]
    fn quasiquote_too_many_args_fails() {
        assert!(eval_src("(quasiquote 1 2)").is_err());
    }

    #[test]
    fn apply_native_with_list_args() {
        assert!(matches!(ok("(apply + '(1 2 3))"), SrsValue::Integer(6)));
    }

    #[test]
    fn apply_with_leading_args_and_list() {
        assert!(matches!(ok("(apply + 1 2 '(3 4))"), SrsValue::Integer(10)));
    }

    #[test]
    fn apply_with_lambda() {
        assert!(matches!(
            ok("(apply (lambda (a b) (* a b)) '(6 7))"),
            SrsValue::Integer(42)
        ));
    }

    #[test]
    fn apply_last_arg_not_a_list_fails() {
        assert!(eval_src("(apply + 1)").is_err());
    }

    #[test]
    fn apply_no_args_fails() {
        assert!(eval_src("(apply)").is_err());
    }

    #[test]
    fn apply_one_arg_fails() {
        assert!(eval_src("(apply +)").is_err());
    }

    #[test]
    fn map_single_list() {
        let v = ok("(map (lambda (x) (* x x)) '(1 2 3))");
        assert!(matches!(pair_car(&v), SrsValue::Integer(1)));
        let rest = pair_cdr(&v);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(4)));
        let rest = pair_cdr(&rest);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(9)));
        assert!(matches!(pair_cdr(&rest), SrsValue::Nil));
    }

    #[test]
    fn map_multiple_lists() {
        let v = ok("(map + '(1 2 3) '(10 20 30))");
        assert!(matches!(pair_car(&v), SrsValue::Integer(11)));
        let rest = pair_cdr(&v);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(22)));
        let rest = pair_cdr(&rest);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(33)));
        assert!(matches!(pair_cdr(&rest), SrsValue::Nil));
    }

    #[test]
    fn map_stops_at_shortest_list() {
        let v = ok("(map + '(1 2 3) '(10 20))");
        assert!(matches!(pair_car(&v), SrsValue::Integer(11)));
        let rest = pair_cdr(&v);
        assert!(matches!(pair_car(&rest), SrsValue::Integer(22)));
        assert!(matches!(pair_cdr(&rest), SrsValue::Nil));
    }

    #[test]
    fn map_empty_list_returns_empty_list() {
        assert!(matches!(ok("(map + '())"), SrsValue::Nil));
    }

    #[test]
    fn map_no_args_fails() {
        assert!(eval_src("(map)").is_err());
    }

    #[test]
    fn map_one_arg_fails() {
        assert!(eval_src("(map +)").is_err());
    }

    fn write_temp_scm(contents: &str) -> String {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!(
            "srs_load_test_{}_{}.scm",
            std::process::id(),
            contents.len()
        ));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn load_defines_bindings_in_current_env() {
        let path = write_temp_scm("(define square (lambda (x) (* x x)))");
        let src = format!("(load \"{}\") (square 6)", path);
        assert!(matches!(ok(&src), SrsValue::Integer(36)));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn load_returns_value_of_last_expression() {
        let path = write_temp_scm("(+ 1 2) (+ 3 4)");
        let src = format!("(load \"{}\")", path);
        assert!(matches!(ok(&src), SrsValue::Integer(7)));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn load_missing_file_fails() {
        assert!(eval_src("(load \"/nonexistent/path/to/file.scm\")").is_err());
    }

    #[test]
    fn load_no_args_fails() {
        assert!(eval_src("(load)").is_err());
    }

    #[test]
    fn load_too_many_args_fails() {
        assert!(eval_src("(load \"a\" \"b\")").is_err());
    }

    #[test]
    fn load_non_string_path_fails() {
        assert!(eval_src("(load 42)").is_err());
    }
}
