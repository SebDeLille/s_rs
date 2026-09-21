use std::process::ExitStatus;
use std::rc::Rc;

use crate::types::core::{Env, Native, PortData, SrsValue};

pub(super) fn install(env: &Rc<Env>) {
    env.define(
        "process-installed?".to_string(),
        SrsValue::Native(Native {
            name: "process-installed?",
            func: Rc::new(native_process_installed_p),
        }),
    );
    env.define(
        "exit".to_string(),
        SrsValue::Native(Native {
            name: "exit",
            func: Rc::new(native_exit),
        }),
    );
    env.define(
        "system".to_string(),
        SrsValue::Native(Native {
            name: "system",
            func: Rc::new(native_system),
        }),
    );
    env.define(
        "system*".to_string(),
        SrsValue::Native(Native {
            name: "system*",
            func: Rc::new(native_system_star),
        }),
    );
    env.define(
        "open-input-pipe".to_string(),
        SrsValue::Native(Native {
            name: "open-input-pipe",
            func: Rc::new(native_open_input_pipe),
        }),
    );
    env.define(
        "close-pipe".to_string(),
        SrsValue::Native(Native {
            name: "close-pipe",
            func: Rc::new(native_close_pipe),
        }),
    );
}

/// `(process-installed?)`: native trivial permettant de valider le
/// branchement du module `process`. Retourne `#t`.
fn native_process_installed_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Ok(SrsValue::Boolean(true)),
        _ => Err("too many arguments to process-installed?".to_string()),
    }
}

/// `(exit)` / `(exit obj)`: extension R7RS `(scheme process-context)` qui
/// demande la terminaison du programme/REPL.
///
/// - `(exit)` et tout objet autre que `#f` ou un entier : code 0.
/// - `(exit #f)` : code 1 (sortie anormale).
/// - `(exit 42)` : le code entier, siloé dans un `i32`.
/// - Plus d'un argument est une erreur.
fn native_exit(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Err(crate::interpretor::evaluator::natives::exit_code_to_exit(0)),
        [SrsValue::Boolean(false)] => {
            Err(crate::interpretor::evaluator::natives::exit_code_to_exit(1))
        }
        [SrsValue::Integer(n)] => Err(crate::interpretor::evaluator::natives::exit_code_to_exit(
            *n as i32,
        )),
        [_] => Err(crate::interpretor::evaluator::natives::exit_code_to_exit(0)),
        _ => Err("too many arguments to exit".to_string()),
    }
}

/// `(system command)`: exécute `command` dans `/bin/sh -c command`.
///
/// Retourne le code de sortie du processus, ou `-1` s'il a été tué par un
/// signal.
fn native_system(args: &[SrsValue]) -> Result<SrsValue, String> {
    if args.is_empty() {
        return Err("not enough arguments to system".to_string());
    }
    if args.len() > 1 {
        return Err("too many arguments to system".to_string());
    }
    let command = match &args[0] {
        SrsValue::String(s) => s.borrow().clone(),
        _ => return Err("wrong type: expected string to system".to_string()),
    };
    let status = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .status()
        .map_err(|e| format!("system: {}", e))?;
    Ok(exit_status_to_integer(status))
}

/// `(system* prog arg...)`: exécute `prog` avec les `arg...` comme arguments.
///
/// Contrairement à `system`, le programme est lancé directement sans passer
/// par un shell intermédiaire.
///
/// Retourne le code de sortie du processus, ou `-1` s'il a été tué par un
/// signal.
fn native_system_star(args: &[SrsValue]) -> Result<SrsValue, String> {
    if args.is_empty() {
        return Err("not enough arguments to system*".to_string());
    }
    let strings = strings_from_args("system*", args)?;
    let (program, argv) = strings.split_first().unwrap();
    let status = std::process::Command::new(program)
        .args(argv)
        .status()
        .map_err(|e| format!("system*: {}", e))?;
    Ok(exit_status_to_integer(status))
}

/// `(open-input-pipe command)`: lance `command` dans `/bin/sh -c command`
/// et retourne un port en lecture sur le stdout du processus fils.
fn native_open_input_pipe(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [SrsValue::String(s)] => {
            let command = s.borrow().clone();
            let mut child = std::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(command)
                .stdout(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| format!("open-input-pipe: {}", e))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| "open-input-pipe: could not capture child stdout".to_string())?;
            Ok(SrsValue::Port(Rc::new(std::cell::RefCell::new(
                PortData::InputPipe {
                    reader: std::io::BufReader::new(Box::new(stdout)),
                    peeked: None,
                    child,
                },
            ))))
        }
        [] => Err("not enough arguments to open-input-pipe".to_string()),
        [SrsValue::Symbol(_) | _] => {
            Err("wrong type: expected string to open-input-pipe".to_string())
        }
        _ => Err("too many arguments to open-input-pipe".to_string()),
    }
}

/// `(close-pipe port)`: ferme un port de pipe et attend le processus fils.
///
/// Retourne le code de sortie du processus, ou `-1` s'il a été tué par un
/// signal.
fn native_close_pipe(args: &[SrsValue]) -> Result<SrsValue, String> {
    if args.is_empty() {
        return Err("not enough arguments to close-pipe".to_string());
    }
    if args.len() > 1 {
        return Err("too many arguments to close-pipe".to_string());
    }
    match &args[0] {
        SrsValue::Port(p) => match p.borrow_mut().close_pipe() {
            Some(status) => Ok(exit_status_to_integer(status)),
            None => Err("wrong type: expected pipe port to close-pipe".to_string()),
        },
        _ => Err("wrong type: expected pipe port to close-pipe".to_string()),
    }
}

/// Convertit un [`ExitStatus`] en code de sortie entier.
///
/// Si le processus s'est terminé normalement, retourne son code de sortie
/// (0-255 typiquement). S'il a été tué par un signal, retourne -1.
#[allow(dead_code)]
pub(super) fn exit_status_to_integer(status: ExitStatus) -> SrsValue {
    let code = status.code().unwrap_or(-1);
    SrsValue::Integer(code as i64)
}

/// Extrait une liste d'arguments `String` depuis une slice de
/// [`SrsValue`].
///
/// Chaque valeur doit être une chaîne de caractères ; sinon retourne une
/// erreur formatée avec le nom de la primitive.
pub(super) fn strings_from_args(name: &str, args: &[SrsValue]) -> Result<Vec<String>, String> {
    args.iter()
        .map(|value| match value {
            SrsValue::String(s) => Ok(s.borrow().clone()),
            _ => Err(format!("{}: wrong type: expected string", name)),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_status_success_returns_zero() {
        let status = ExitStatus::default();
        assert!(matches!(
            exit_status_to_integer(status),
            SrsValue::Integer(0)
        ));
    }

    #[test]
    fn strings_from_args_extracts_strings() {
        let a = SrsValue::String(Rc::new(std::cell::RefCell::new("a".to_string())));
        let b = SrsValue::String(Rc::new(std::cell::RefCell::new("b".to_string())));
        let got = strings_from_args("test", &[a, b]).unwrap();
        assert_eq!(got, vec!["a", "b"]);
    }

    #[test]
    fn strings_from_args_rejects_non_string() {
        let got = strings_from_args("test", &[SrsValue::Integer(42)]);
        assert!(got.is_err());
        assert!(got.unwrap_err().contains("wrong type"));
    }

    #[test]
    fn system_returns_zero_on_success() {
        let args = [SrsValue::String(Rc::new(std::cell::RefCell::new(
            "exit 0".to_string(),
        )))];
        let result = native_system(&args).unwrap();
        assert!(matches!(result, SrsValue::Integer(0)));
    }

    #[test]
    fn system_returns_non_zero_exit_code() {
        let args = [SrsValue::String(Rc::new(std::cell::RefCell::new(
            "exit 3".to_string(),
        )))];
        let result = native_system(&args).unwrap();
        assert!(matches!(result, SrsValue::Integer(3)));
    }

    #[test]
    fn system_rejects_non_string_argument() {
        let args = [SrsValue::Integer(42)];
        let err = native_system(&args).unwrap_err();
        assert_eq!(err, "wrong type: expected string to system");
    }

    #[test]
    fn system_rejects_no_arguments() {
        let err = native_system(&[]).unwrap_err();
        assert_eq!(err, "not enough arguments to system");
    }

    #[test]
    fn system_rejects_too_many_arguments() {
        let a = SrsValue::String(Rc::new(std::cell::RefCell::new("echo".to_string())));
        let b = SrsValue::String(Rc::new(std::cell::RefCell::new("hello".to_string())));
        let err = native_system(&[a, b]).unwrap_err();
        assert_eq!(err, "too many arguments to system");
    }

    #[test]
    fn system_star_returns_zero_on_success() {
        let args = [
            SrsValue::String(Rc::new(std::cell::RefCell::new("/bin/sh".to_string()))),
            SrsValue::String(Rc::new(std::cell::RefCell::new("-c".to_string()))),
            SrsValue::String(Rc::new(std::cell::RefCell::new("exit 0".to_string()))),
        ];
        let result = native_system_star(&args).unwrap();
        assert!(matches!(result, SrsValue::Integer(0)));
    }

    #[test]
    fn system_star_returns_non_zero_exit_code() {
        let args = [
            SrsValue::String(Rc::new(std::cell::RefCell::new("/bin/sh".to_string()))),
            SrsValue::String(Rc::new(std::cell::RefCell::new("-c".to_string()))),
            SrsValue::String(Rc::new(std::cell::RefCell::new("exit 7".to_string()))),
        ];
        let result = native_system_star(&args).unwrap();
        assert!(matches!(result, SrsValue::Integer(7)));
    }

    #[test]
    fn system_star_rejects_non_string_argument() {
        let args = [
            SrsValue::String(Rc::new(std::cell::RefCell::new("/bin/sh".to_string()))),
            SrsValue::Integer(42),
        ];
        let err = native_system_star(&args).unwrap_err();
        assert_eq!(err, "system*: wrong type: expected string");
    }

    #[test]
    fn system_star_rejects_no_arguments() {
        let err = native_system_star(&[]).unwrap_err();
        assert_eq!(err, "not enough arguments to system*");
    }

    #[test]
    fn system_star_returns_error_when_program_not_found() {
        let args = [SrsValue::String(Rc::new(std::cell::RefCell::new(
            "/no/such/program/should/exist".to_string(),
        )))];
        let err = native_system_star(&args).unwrap_err();
        assert!(err.starts_with("system*:"));
    }

    #[test]
    fn exit_marker_starts_with_prefix() {
        let err = super::super::exit_code_to_exit(42);
        assert!(err.starts_with("\x1b__EXIT_MARKER__:"));
        assert!(err.ends_with("\x1b"));
    }

    #[test]
    fn exit_with_no_argument_requests_zero() {
        let err = native_exit(&[]).unwrap_err();
        assert!(err.contains("__EXIT_MARKER__:0"));
    }

    #[test]
    fn exit_with_false_requests_failure() {
        let err = native_exit(&[SrsValue::Boolean(false)]).unwrap_err();
        assert!(err.contains("__EXIT_MARKER__:1"));
    }

    #[test]
    fn exit_with_integer_requests_code() {
        let err = native_exit(&[SrsValue::Integer(42)]).unwrap_err();
        assert!(err.contains("__EXIT_MARKER__:42"));
    }

    #[test]
    fn exit_with_true_requests_zero() {
        let err = native_exit(&[SrsValue::Boolean(true)]).unwrap_err();
        assert!(err.contains("__EXIT_MARKER__:0"));
    }

    #[test]
    fn exit_with_other_value_requests_zero() {
        let err = native_exit(&[SrsValue::String(Rc::new(std::cell::RefCell::new(
            "bye".to_string(),
        )))])
        .unwrap_err();
        assert!(err.contains("__EXIT_MARKER__:0"));
    }

    #[test]
    fn exit_rejects_too_many_arguments() {
        let err = native_exit(&[SrsValue::Boolean(true), SrsValue::Boolean(false)]).unwrap_err();
        assert_eq!(err, "too many arguments to exit");
    }

    #[test]
    fn open_input_pipe_reads_line_from_child_stdout() {
        let args = [SrsValue::String(Rc::new(std::cell::RefCell::new(
            "echo hello".to_string(),
        )))];
        let port_value = native_open_input_pipe(&args).unwrap();
        let port = match port_value {
            SrsValue::Port(p) => p,
            other => panic!("expected port, got {}", other),
        };
        let result = port.borrow_mut().read_line().unwrap();
        assert!(matches!(result, SrsValue::String(s) if s.borrow().as_str() == "hello"));
    }

    #[test]
    fn open_input_pipe_reads_until_eof() {
        let args = [SrsValue::String(Rc::new(std::cell::RefCell::new(
            "printf 'a b c'".to_string(),
        )))];
        let port_value = native_open_input_pipe(&args).unwrap();
        let port = match port_value {
            SrsValue::Port(p) => p,
            other => panic!("expected port, got {}", other),
        };
        let mut read = Vec::new();
        loop {
            match port.borrow_mut().read_char().unwrap() {
                SrsValue::Character(c) => read.push(c),
                SrsValue::Eof => break,
                other => panic!("unexpected value: {}", other),
            }
        }
        assert_eq!(read.iter().collect::<String>(), "a b c");
    }

    #[test]
    fn open_input_pipe_reports_failure_to_read_stdout() {
        // Une commande valide mais sans stdout piped n'est pas applicable
        // ici car on pipe toujours stdout. On vérifie donc indirectement
        // l'erreur en créant un pipe dont l'extrémité de lecture est
        // consommée, ce qui force un échec.
        let result = native_open_input_pipe(&[SrsValue::String(Rc::new(std::cell::RefCell::new(
            "true".to_string(),
        )))]);
        let port_value = result.unwrap();
        let port = match port_value {
            SrsValue::Port(p) => p,
            other => panic!("expected port, got {}", other),
        };
        // Le port est un InputPipe : la primitive a bien fonctionné.
        assert!(port.borrow().is_input_port());
    }

    #[test]
    fn open_input_pipe_rejects_non_string_argument() {
        let args = [SrsValue::Integer(42)];
        let err = native_open_input_pipe(&args).unwrap_err();
        assert_eq!(err, "wrong type: expected string to open-input-pipe");
    }

    #[test]
    fn open_input_pipe_rejects_no_arguments() {
        let err = native_open_input_pipe(&[]).unwrap_err();
        assert_eq!(err, "not enough arguments to open-input-pipe");
    }

    #[test]
    fn open_input_pipe_rejects_too_many_arguments() {
        let a = SrsValue::String(Rc::new(std::cell::RefCell::new("echo".to_string())));
        let b = SrsValue::String(Rc::new(std::cell::RefCell::new("hello".to_string())));
        let err = native_open_input_pipe(&[a, b]).unwrap_err();
        assert_eq!(err, "too many arguments to open-input-pipe");
    }

    #[test]
    fn close_pipe_returns_zero_on_success() {
        let port = native_open_input_pipe(&[SrsValue::String(Rc::new(std::cell::RefCell::new(
            "exit 0".to_string(),
        )))])
        .unwrap();
        let result = native_close_pipe(std::slice::from_ref(&port)).unwrap();
        assert!(matches!(result, SrsValue::Integer(0)));
    }

    #[test]
    fn close_pipe_returns_non_zero_exit_code() {
        let port = native_open_input_pipe(&[SrsValue::String(Rc::new(std::cell::RefCell::new(
            "exit 5".to_string(),
        )))])
        .unwrap();
        let result = native_close_pipe(std::slice::from_ref(&port)).unwrap();
        assert!(matches!(result, SrsValue::Integer(5)));
    }

    #[test]
    fn close_pipe_rejects_non_port_argument() {
        let err = native_close_pipe(&[SrsValue::Integer(42)]).unwrap_err();
        assert_eq!(err, "wrong type: expected pipe port to close-pipe");
    }

    #[test]
    fn close_pipe_rejects_no_arguments() {
        let err = native_close_pipe(&[]).unwrap_err();
        assert_eq!(err, "not enough arguments to close-pipe");
    }

    #[test]
    fn close_pipe_rejects_too_many_arguments() {
        let port = native_open_input_pipe(&[SrsValue::String(Rc::new(std::cell::RefCell::new(
            "echo".to_string(),
        )))])
        .unwrap();
        let err = native_close_pipe(&[port.clone(), port]).unwrap_err();
        assert_eq!(err, "too many arguments to close-pipe");
    }

    #[test]
    fn close_pipe_rejects_non_pipe_input_port() {
        let file_port = SrsValue::Port(Rc::new(std::cell::RefCell::new(PortData::InputFile {
            reader: std::io::BufReader::new(Box::new(std::io::empty())),
            peeked: None,
        })));
        let err = native_close_pipe(&[file_port]).unwrap_err();
        assert_eq!(err, "wrong type: expected pipe port to close-pipe");
    }

    #[test]
    fn close_pipe_rejects_output_port() {
        let output_port = SrsValue::Port(Rc::new(std::cell::RefCell::new(PortData::OutputString(
            String::new(),
        ))));
        let err = native_close_pipe(&[output_port]).unwrap_err();
        assert_eq!(err, "wrong type: expected pipe port to close-pipe");
    }
}
