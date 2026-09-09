use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

use super::numeric_to_f64;

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "sin".to_string(),
        SrsValue::Native(Native {
            name: "sin",
            func: Rc::new(native_sin),
        }),
    );
    env.define(
        "cos".to_string(),
        SrsValue::Native(Native {
            name: "cos",
            func: Rc::new(native_cos),
        }),
    );
    env.define(
        "tan".to_string(),
        SrsValue::Native(Native {
            name: "tan",
            func: Rc::new(native_tan),
        }),
    );
    env.define(
        "atan".to_string(),
        SrsValue::Native(Native {
            name: "atan",
            func: Rc::new(native_atan),
        }),
    );
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
