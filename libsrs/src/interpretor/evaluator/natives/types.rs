use std::cell::RefCell;
use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

pub(super) fn install(env: &Rc<Env>) {
    for (name, func) in [
        (
            "symbol?",
            native_symbol_p as fn(&[SrsValue]) -> Result<SrsValue, String>,
        ),
        ("symbol->string", native_symbol_to_string),
        ("string->symbol", native_string_to_symbol),
        ("boolean?", native_boolean_p),
        ("procedure?", native_procedure_p),
    ] {
        let name: &'static str = match name {
            "symbol?" => "symbol?",
            "symbol->string" => "symbol->string",
            "string->symbol" => "string->symbol",
            "boolean?" => "boolean?",
            _ => "procedure?",
        };
        env.define(
            name.to_string(),
            SrsValue::Native(Native {
                name,
                func: Rc::new(func),
            }),
        );
    }
}

fn native_symbol_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(matches!(value, SrsValue::Symbol(_)))),
        [] => Err("not enough arguments to symbol?".to_string()),
        _ => Err("too many arguments to symbol?".to_string()),
    }
}

fn native_symbol_to_string(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Symbol(symbol)] => Ok(SrsValue::String(Rc::new(RefCell::new(symbol.clone())))),
        [_] => Err("wrong type: expected symbol".to_string()),
        [] => Err("not enough arguments to symbol->string".to_string()),
        _ => Err("too many arguments to symbol->string".to_string()),
    }
}

fn native_string_to_symbol(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(string)] => Ok(SrsValue::Symbol(string.borrow().clone())),
        [_] => Err("wrong type: expected string".to_string()),
        [] => Err("not enough arguments to string->symbol".to_string()),
        _ => Err("too many arguments to string->symbol".to_string()),
    }
}

fn native_boolean_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(matches!(value, SrsValue::Boolean(_)))),
        [] => Err("not enough arguments to boolean?".to_string()),
        _ => Err("too many arguments to boolean?".to_string()),
    }
}

fn native_procedure_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(matches!(
            value,
            SrsValue::Procedure(_) | SrsValue::Native(_)
        ))),
        [] => Err("not enough arguments to procedure?".to_string()),
        _ => Err("too many arguments to procedure?".to_string()),
    }
}
