//! Shared REPL evaluation pipeline (lex -> read -> eval), reusable by any
//! front-end (CLI, GTK widget, ...).
//!
//! The main feature on top of a naive `lex |> read |> eval` pipeline is the
//! ability to tell apart a genuine syntax error from an *incomplete* input
//! (e.g. an unclosed parenthesis or string): front-ends can use this to
//! implement multi-line input, waiting for more source instead of reporting
//! an error immediately.

use std::rc::Rc;

use crate::interpretor::evaluator::eval;
use crate::interpretor::lexical_analyzer::{LexErrorKind, get_lexemes};
use crate::interpretor::reader::{ReadErrorKind, read_all};
use crate::types::core::{Env, SrsValue};

/// Result of trying to evaluate a chunk of source text.
#[derive(Debug)]
pub enum EvalOutcome {
    /// The source contained zero or more complete top-level forms, each
    /// evaluated in order. The values are the results of each form, in
    /// source order.
    Done(Vec<SrsValue>),
    /// The source is a valid prefix of a longer form (unclosed list,
    /// unterminated string, ...): the caller should read more input and
    /// try again with the concatenated source.
    Incomplete,
}

/// Runs the `lex -> read -> eval` pipeline over `source`, evaluating every
/// top-level form found in the given `env`.
///
/// Returns [`EvalOutcome::Incomplete`] when `source` looks like a truncated
/// form rather than an invalid one, so that REPL front-ends can prompt for
/// more input instead of surfacing a spurious error.
pub fn eval_source(source: &str, env: &Rc<Env>) -> Result<EvalOutcome, String> {
    let lexemes = match get_lexemes(source) {
        Ok(lexemes) => lexemes,
        Err(e) if is_incomplete_lex_error(&e.kind) => return Ok(EvalOutcome::Incomplete),
        Err(e) => return Err(e.to_string()),
    };

    let values = match read_all(lexemes) {
        Ok(values) => values,
        Err(e) if e.kind == ReadErrorKind::UnexpectedEof => return Ok(EvalOutcome::Incomplete),
        Err(e) => return Err(e.to_string()),
    };

    let mut results = Vec::with_capacity(values.len());
    for value in &values {
        match eval(value, env) {
            Ok(result) => results.push(result),
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(EvalOutcome::Done(results))
}

/// Whether a lexical error stems from input ending prematurely (unclosed
/// string, character literal, block comment, ...) rather than from an
/// actually malformed token.
fn is_incomplete_lex_error(kind: &LexErrorKind) -> bool {
    matches!(
        kind,
        LexErrorKind::UnterminatedCharacter
            | LexErrorKind::UnterminatedSharpDispatch
            | LexErrorKind::UnterminatedBlockComment
            | LexErrorKind::UnterminatedString
            | LexErrorKind::UnterminatedExponent
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretor::evaluator::global_env;

    #[test]
    fn evaluates_complete_forms_in_order() {
        let env = global_env();
        let outcome = eval_source("(+ 1 2) (* 2 3)", &env).unwrap();
        match outcome {
            EvalOutcome::Done(values) => {
                assert_eq!(values.len(), 2);
                assert!(matches!(values[0], SrsValue::Integer(3)));
                assert!(matches!(values[1], SrsValue::Integer(6)));
            }
            EvalOutcome::Incomplete => panic!("expected complete evaluation"),
        }
    }

    #[test]
    fn reports_incomplete_unclosed_list() {
        let env = global_env();
        let outcome = eval_source("(+ 1 2", &env).unwrap();
        assert!(matches!(outcome, EvalOutcome::Incomplete));
    }

    #[test]
    fn reports_incomplete_unterminated_string() {
        let env = global_env();
        let outcome = eval_source("(display \"hello", &env).unwrap();
        assert!(matches!(outcome, EvalOutcome::Incomplete));
    }

    #[test]
    fn reports_genuine_errors_as_such() {
        let env = global_env();
        let err = eval_source("(unbound-symbol)", &env).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn continuation_can_complete_a_pending_form() {
        let env = global_env();
        assert!(matches!(
            eval_source("(+ 1", &env).unwrap(),
            EvalOutcome::Incomplete
        ));
        let outcome = eval_source("(+ 1\n2)", &env).unwrap();
        match outcome {
            EvalOutcome::Done(values) => {
                assert_eq!(values.len(), 1);
                assert!(matches!(values[0], SrsValue::Integer(3)));
            }
            EvalOutcome::Incomplete => panic!("expected complete evaluation"),
        }
    }
}
