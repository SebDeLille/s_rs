use std::cell::RefCell;
use std::rc::Rc;

use crate::types::core::{Env, Native, PortData, SrsValue};

pub(super) fn install(
    env: &Rc<Env>,
    stdin_port: Rc<RefCell<PortData>>,
    stdout_port: Rc<RefCell<PortData>>,
) {
    let stdin_for_current = stdin_port.clone();
    let stdin_for_read = stdin_port.clone();
    let stdin_for_peek = stdin_port;

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
    env.define(
        "current-input-port".to_string(),
        SrsValue::Native(Native {
            name: "current-input-port",
            func: Rc::new(move |args| native_current_input_port(args, &stdin_for_current)),
        }),
    );
    env.define(
        "current-output-port".to_string(),
        SrsValue::Native(Native {
            name: "current-output-port",
            func: Rc::new(move |args| native_current_output_port(args, &stdout_port)),
        }),
    );
    env.define(
        "read-char".to_string(),
        SrsValue::Native(Native {
            name: "read-char",
            func: Rc::new(move |args| native_read_char(args, &stdin_for_read)),
        }),
    );
    env.define(
        "peek-char".to_string(),
        SrsValue::Native(Native {
            name: "peek-char",
            func: Rc::new(move |args| native_peek_char(args, &stdin_for_peek)),
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

/// `(current-input-port)`: returns the singleton standard input port.
fn native_current_input_port(
    args: &[SrsValue],
    stdin_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    match args {
        [] => Ok(SrsValue::Port(stdin_port.clone())),
        _ => Err("too many arguments to current-input-port".to_string()),
    }
}

/// `(current-output-port)`: returns the singleton standard output port.
fn native_current_output_port(
    args: &[SrsValue],
    stdout_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    match args {
        [] => Ok(SrsValue::Port(stdout_port.clone())),
        _ => Err("too many arguments to current-output-port".to_string()),
    }
}

/// Resolves the port argument for character-input procedures.
/// Accepts either zero arguments (use the default stdin port) or one
/// argument that must be an input port.
fn input_port_from_args(
    proc_name: &str,
    args: &[SrsValue],
    default: &Rc<RefCell<PortData>>,
) -> Result<Rc<RefCell<PortData>>, String> {
    match args {
        [] => Ok(default.clone()),
        [SrsValue::Port(port)] => Ok(port.clone()),
        [_] => Err(format!("wrong type: expected input port to {}", proc_name)),
        _ => Err(format!("too many arguments to {}", proc_name)),
    }
}

/// `(read-char [port])`: reads and consumes one character from the input
/// port, returning `eof-object` at end of file.
fn native_read_char(
    args: &[SrsValue],
    stdin_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    let port = input_port_from_args("read-char", args, stdin_port)?;
    port.borrow_mut().read_char()
}

/// `(peek-char [port])`: reads one character from the input port without
/// consuming it, returning `eof-object` at end of file.
fn native_peek_char(
    args: &[SrsValue],
    stdin_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    let port = input_port_from_args("peek-char", args, stdin_port)?;
    port.borrow_mut().peek_char()
}
