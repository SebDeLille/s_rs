use std::cell::RefCell;
use std::rc::Rc;

use super::index_from;
use crate::types::core::{Env, Native, PortData, SrsValue};

pub(super) fn install(
    env: &Rc<Env>,
    stdin_port: Rc<RefCell<PortData>>,
    stdout_port: Rc<RefCell<PortData>>,
) {
    let stdin_for_current = stdin_port.clone();
    let stdin_for_read = stdin_port.clone();
    let stdin_for_peek = stdin_port.clone();
    let stdin_for_read_line = stdin_port.clone();
    let stdin_for_char_ready = stdin_port;

    let stdout_for_display = stdout_port.clone();
    let stdout_for_newline = stdout_port.clone();
    let stdout_for_write_char = stdout_port.clone();
    let stdout_for_write_string = stdout_port.clone();

    env.define(
        "display".to_string(),
        SrsValue::Native(Native {
            name: "display",
            func: Rc::new(move |args| native_display(args, &stdout_for_display)),
        }),
    );
    env.define(
        "newline".to_string(),
        SrsValue::Native(Native {
            name: "newline",
            func: Rc::new(move |args| native_newline(args, &stdout_for_newline)),
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
    env.define(
        "read-line".to_string(),
        SrsValue::Native(Native {
            name: "read-line",
            func: Rc::new(move |args| native_read_line(args, &stdin_for_read_line)),
        }),
    );
    env.define(
        "char-ready?".to_string(),
        SrsValue::Native(Native {
            name: "char-ready?",
            func: Rc::new(move |args| native_char_ready_p(args, &stdin_for_char_ready)),
        }),
    );
    env.define(
        "open-input-string".to_string(),
        SrsValue::Native(Native {
            name: "open-input-string",
            func: Rc::new(native_open_input_string),
        }),
    );
    env.define(
        "open-input-file".to_string(),
        SrsValue::Native(Native {
            name: "open-input-file",
            func: Rc::new(native_open_input_file),
        }),
    );
    env.define(
        "open-output-string".to_string(),
        SrsValue::Native(Native {
            name: "open-output-string",
            func: Rc::new(native_open_output_string),
        }),
    );
    env.define(
        "get-output-string".to_string(),
        SrsValue::Native(Native {
            name: "get-output-string",
            func: Rc::new(native_get_output_string),
        }),
    );
    env.define(
        "write-char".to_string(),
        SrsValue::Native(Native {
            name: "write-char",
            func: Rc::new(move |args| native_write_char(args, &stdout_for_write_char)),
        }),
    );
    env.define(
        "write-string".to_string(),
        SrsValue::Native(Native {
            name: "write-string",
            func: Rc::new(move |args| native_write_string(args, &stdout_for_write_string)),
        }),
    );
}

/// `(display value [port])`: writes `value` to the output port using its
/// human-readable representation (no quotes around strings, characters
/// printed as themselves). Uses the current output port when `port` is
/// omitted. Returns [`SrsValue::Unspecified`].
fn native_display(
    args: &[SrsValue],
    stdout_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    let (value, port) = match args {
        [value] => (value, stdout_port.clone()),
        [value, SrsValue::Port(port)] => {
            if !port.borrow().is_output_port() {
                return Err("display: not an output port".to_string());
            }
            (value, port.clone())
        }
        [] => return Err("not enough arguments to display".to_string()),
        [_, _] => return Err("wrong type: expected output port".to_string()),
        _ => return Err("too many arguments to display".to_string()),
    };
    let repr = value.display_repr();
    let mut port = port.borrow_mut();
    match &mut *port {
        PortData::OutputString(buf) => {
            buf.push_str(&repr);
            Ok(SrsValue::Unspecified)
        }
        PortData::OutputStdout => {
            use std::io::Write;
            print!("{}", repr);
            std::io::stdout()
                .flush()
                .map_err(|e| format!("display: {}", e))?;
            Ok(SrsValue::Unspecified)
        }
        _ => Err("display: not an output port".to_string()),
    }
}

/// `(newline [port])`: writes a line break to the output port. Uses the
/// current output port when `port` is omitted. Returns
/// [`SrsValue::Unspecified`].
fn native_newline(
    args: &[SrsValue],
    stdout_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    match args {
        [] => {
            let mut port = stdout_port.borrow_mut();
            port.write_char('\n').map_err(|e| {
                if e.starts_with("write-char:") {
                    e.replace("write-char:", "newline:")
                } else {
                    format!("newline: {}", e)
                }
            })?;
            Ok(SrsValue::Unspecified)
        }
        [SrsValue::Port(port)] => {
            port.borrow_mut().write_char('\n').map_err(|e| {
                if e.starts_with("write-char:") {
                    e.replace("write-char:", "newline:")
                } else {
                    format!("newline: {}", e)
                }
            })?;
            Ok(SrsValue::Unspecified)
        }
        [_] => Err("wrong type: expected output port".to_string()),
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
        [SrsValue::Port(port)] if port.borrow().is_input_port() => Ok(port.clone()),
        [SrsValue::Port(_)] => Err(format!("{}: not an input port", proc_name)),
        [_] => Err(format!("wrong type: expected input port to {}", proc_name)),
        _ => Err(format!("too many arguments to {}", proc_name)),
    }
}

/// Resolves the port argument for output procedures.
/// Accepts either zero arguments (use the default stdout port) or one
/// argument that must be an output port.
#[allow(dead_code)]
fn output_port_from_args(
    proc_name: &str,
    args: &[SrsValue],
    default: &Rc<RefCell<PortData>>,
) -> Result<Rc<RefCell<PortData>>, String> {
    match args {
        [] => Ok(default.clone()),
        [SrsValue::Port(port)] if port.borrow().is_output_port() => Ok(port.clone()),
        [SrsValue::Port(_)] => Err(format!("{}: not an output port", proc_name)),
        [_] => Err(format!("wrong type: expected output port to {}", proc_name)),
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

/// `(read-line [port])`: reads characters from the input port until a
/// newline or end of file, returning the accumulated string. Returns
/// `eof-object` if no characters could be read before EOF.
fn native_read_line(
    args: &[SrsValue],
    stdin_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    let port = input_port_from_args("read-line", args, stdin_port)?;
    port.borrow_mut().read_line()
}

/// `(char-ready? [port])`: returns `#t` if a character is ready on the
/// input port, `#f` otherwise.
///
/// NOTE: Correct implementation for stdin is non-trivial because the
/// underlying `BufReader` may block on real terminal input. String ports
/// report availability based on remaining characters.
fn native_char_ready_p(
    args: &[SrsValue],
    stdin_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    let port = input_port_from_args("char-ready?", args, stdin_port)?;
    let ready = match &*port.borrow() {
        PortData::InputStdin { .. } | PortData::InputFile { .. } => true,
        PortData::InputString(chars, pos) => *pos < chars.len(),
        _ => false,
    };
    Ok(SrsValue::Boolean(ready))
}

/// `(open-input-string s)`: returns a new input port reading from string `s`.
fn native_open_input_string(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(s)] => Ok(SrsValue::Port(Rc::new(RefCell::new(
            PortData::input_string(s.borrow().clone()),
        )))),
        [SrsValue::Symbol(_)] => Err("open-input-string: expected string".to_string()),
        [] => Err("not enough arguments to open-input-string".to_string()),
        [_] => Err("wrong type: expected string".to_string()),
        _ => Err("too many arguments to open-input-string".to_string()),
    }
}

/// `(open-input-file path)`: returns a new input port reading from file
/// located at `path`.
fn native_open_input_file(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(s)] => Ok(SrsValue::Port(Rc::new(RefCell::new(PortData::input_file(
            &*s.borrow(),
        )?)))),
        [SrsValue::Symbol(_)] => Err("open-input-file: expected string".to_string()),
        [] => Err("not enough arguments to open-input-file".to_string()),
        [_] => Err("wrong type: expected string".to_string()),
        _ => Err("too many arguments to open-input-file".to_string()),
    }
}

/// `(open-output-string)`: returns a new output port accumulating into a
/// string buffer.
fn native_open_output_string(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Ok(SrsValue::Port(Rc::new(RefCell::new(
            PortData::output_string(),
        )))),
        _ => Err("too many arguments to open-output-string".to_string()),
    }
}

/// `(get-output-string port)`: returns the contents accumulated in an
/// output-string port as a new Scheme string.
fn native_get_output_string(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Port(port)] => {
            let contents = port.borrow().get_output_string()?;
            Ok(SrsValue::String(Rc::new(RefCell::new(contents))))
        }
        [] => Err("not enough arguments to get-output-string".to_string()),
        _ => Err("wrong type: expected output-string port".to_string()),
    }
}

/// `(write-char char [port])`: writes `char` to the output port. Uses the
/// current output port when no port is provided.
fn native_write_char(
    args: &[SrsValue],
    stdout_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    match args {
        [SrsValue::Character(c)] => stdout_port
            .borrow_mut()
            .write_char(*c)
            .map(|_| SrsValue::Unspecified),
        [SrsValue::Character(c), SrsValue::Port(port)] => port
            .borrow_mut()
            .write_char(*c)
            .map(|_| SrsValue::Unspecified),
        [] | [_] => Err("not enough arguments to write-char".to_string()),
        [_, _] => Err("wrong type: expected character and output port".to_string()),
        _ => Err("too many arguments to write-char".to_string()),
    }
}

/// `(write-string string [port [start [end]]])`: writes the characters of
/// `string` to the output port. Uses the current output port when `port` is
/// omitted. Returns [`SrsValue::Unspecified`].
fn native_write_string(
    args: &[SrsValue],
    stdout_port: &Rc<RefCell<PortData>>,
) -> Result<SrsValue, String> {
    if args.is_empty() {
        return Err("not enough arguments to write-string".to_string());
    }
    if args.len() > 4 {
        return Err("too many arguments to write-string".to_string());
    }
    let s = match &args[0] {
        SrsValue::String(s) => s.borrow().clone(),
        _ => return Err("wrong type: expected string".to_string()),
    };
    let port = match args.get(1) {
        None => stdout_port.clone(),
        Some(SrsValue::Port(port)) if port.borrow().is_output_port() => port.clone(),
        Some(SrsValue::Port(_)) => return Err("write-string: not an output port".to_string()),
        Some(_) => return Err("wrong type: expected output port".to_string()),
    };
    let start = match args.get(2) {
        Some(v) => Some(index_from(v, "write-string")?),
        None => None,
    };
    let end = match args.get(3) {
        Some(v) => Some(index_from(v, "write-string")?),
        None => None,
    };
    port.borrow_mut()
        .write_string(&s, start, end)
        .map(|_| SrsValue::Unspecified)
}
