use std::rc::Rc;

use std::cell::RefCell;

use crate::types::core::{Env, Native, PromiseState, SrsValue};

use super::super::{apply, eval, scheme_error_to_native};

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "dynamic-wind".to_string(),
        SrsValue::Native(Native {
            name: "dynamic-wind",
            func: Rc::new(native_dynamic_wind),
        }),
    );
    env.define(
        "force".to_string(),
        SrsValue::Native(Native {
            name: "force",
            func: Rc::new(native_force),
        }),
    );
    env.define(
        "make-promise".to_string(),
        SrsValue::Native(Native {
            name: "make-promise",
            func: Rc::new(native_make_promise),
        }),
    );
    env.define(
        "promise?".to_string(),
        SrsValue::Native(Native {
            name: "promise?",
            func: Rc::new(native_promise_p),
        }),
    );
}

/// `(force promise)` evaluates a delayed promise once and memoizes the
/// result. Non-promise arguments are returned unchanged.
fn native_force(args: &[SrsValue]) -> Result<SrsValue, String> {
    let [value] = args else {
        return Err(match args.len() {
            0 => "not enough arguments to force".to_string(),
            _ => "too many arguments to force".to_string(),
        });
    };
    let SrsValue::Promise(promise) = value else {
        return Ok(value.clone());
    };
    let state = promise.borrow().clone();
    match state {
        PromiseState::Forced(v) => Ok(v),
        PromiseState::Delayed(body, env) => {
            let mut result = SrsValue::Unspecified;
            for expr in &body {
                result = eval(expr, &env).map_err(|e| scheme_error_to_native(&e))?;
            }
            // The promise may have been forced re-entrantly; keep the first value.
            let mut slot = promise.borrow_mut();
            if let PromiseState::Forced(v) = &*slot {
                return Ok(v.clone());
            }
            *slot = PromiseState::Forced(result.clone());
            Ok(result)
        }
    }
}

/// `(make-promise obj)` returns an already forced promise (or `obj` itself
/// when it is already a promise).
fn native_make_promise(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [p @ SrsValue::Promise(_)] => Ok(p.clone()),
        [v] => Ok(SrsValue::Promise(Rc::new(RefCell::new(
            PromiseState::Forced(v.clone()),
        )))),
        [] => Err("not enough arguments to make-promise".to_string()),
        _ => Err("too many arguments to make-promise".to_string()),
    }
}

fn native_promise_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [v] => Ok(SrsValue::Boolean(matches!(v, SrsValue::Promise(_)))),
        [] => Err("not enough arguments to promise?".to_string()),
        _ => Err("too many arguments to promise?".to_string()),
    }
}

/// `(dynamic-wind before thunk after)` evaluates `before` with no arguments,
/// then `thunk` with no arguments, then `after` with no arguments. The result
/// returned is the value produced by `thunk`.
///
/// The `after` thunk is always invoked, even if `thunk` raises an error. In
/// that case the `after` thunk runs first (for cleanup), then the original
/// error is propagated to the caller.
///
/// This is a "downward-only" implementation: since there is no
/// `call-with-current-continuation` in this interpreter, continuations cannot
/// re-enter `thunk` after `after` has already run.
fn native_dynamic_wind(args: &[SrsValue]) -> Result<SrsValue, String> {
    let [before, thunk, after] = args else {
        return Err(match args.len() {
            0..=2 => "not enough arguments to dynamic-wind".to_string(),
            _ => "too many arguments to dynamic-wind".to_string(),
        });
    };

    apply(before, &[]).map_err(|e| e.to_string())?;
    let result = apply(thunk, &[]);
    let _ = apply(after, &[]).map_err(|e| e.to_string())?;
    result.map_err(|e| e.to_string())
}
