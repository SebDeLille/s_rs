use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

pub(super) fn install(env: &Rc<Env>) {
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
