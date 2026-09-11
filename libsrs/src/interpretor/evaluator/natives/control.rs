use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

use super::super::apply;

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "dynamic-wind".to_string(),
        SrsValue::Native(Native {
            name: "dynamic-wind",
            func: Rc::new(native_dynamic_wind),
        }),
    );
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
            0 | 1 | 2 => "not enough arguments to dynamic-wind".to_string(),
            _ => "too many arguments to dynamic-wind".to_string(),
        });
    };

    apply(before, &[]).map_err(|e| e.to_string())?;
    let result = apply(thunk, &[]);
    let _ = apply(after, &[]).map_err(|e| e.to_string())?;
    result.map_err(|e| e.to_string())
}
