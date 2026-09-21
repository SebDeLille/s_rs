use std::process::ExitStatus;
use std::rc::Rc;

use crate::types::core::{Env, Native, SrsValue};

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
#[allow(dead_code)]
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
}
