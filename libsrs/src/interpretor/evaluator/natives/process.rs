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
}

/// `(process-installed?)`: native trivial permettant de valider le
/// branchement du module `process`. Retourne `#t`.
fn native_process_installed_p(args: &[SrsValue]) -> Result<SrsValue, String> {
    match args {
        [] => Ok(SrsValue::Boolean(true)),
        _ => Err("too many arguments to process-installed?".to_string()),
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
}
