use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "display".to_string(),
        SrsValue::Native(Native {
            name: "display",
            func: Rc::new(native_display),
        }),
    );
    env.define(
        "newline".to_string(),
        SrsValue::Native(Native {
            name: "newline",
            func: Rc::new(native_newline),
        }),
    );
    env.define(
        "eof-object".to_string(),
        SrsValue::Native(Native {
            name: "eof-object",
            func: Rc::new(native_eof_object),
        }),
    );
    env.define(
        "eof-object?".to_string(),
        SrsValue::Native(Native {
            name: "eof-object?",
            func: Rc::new(native_eof_object_p),
        }),
    );
}

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

/// `(eof-object)`: returns the canonical end-of-file object.
/// There is only one EOF value, represented by [`SrsValue::Eof`].
fn native_eof_object(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Ok(SrsValue::Eof),
        _ => Err("too many arguments to eof-object".to_string()),
    }
}

/// `(eof-object? value)`: returns `#t` if `value` is the end-of-file
/// object, `#f` otherwise.
fn native_eof_object_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [value] => Ok(SrsValue::Boolean(matches!(value, SrsValue::Eof))),
        [] => Err("not enough arguments to eof-object?".to_string()),
        _ => Err("too many arguments to eof-object?".to_string()),
    }
}
