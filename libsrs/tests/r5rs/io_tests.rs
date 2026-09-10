use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

fn eval_all(scm: &str) -> Result<SrsValue, String> {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let env = global_env();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, &env).map_err(|e| e.to_string())?;
    }
    Ok(result)
}

fn eval_src(scm: &str) -> SrsValue {
    eval_all(scm).unwrap()
}

#[test]
fn eof_object_returns_eof_value() {
    assert!(matches!(eval_src("(eof-object)"), SrsValue::Eof));
}

#[test]
fn eof_object_p_returns_true_for_eof_object() {
    assert!(matches!(
        eval_src("(eof-object? (eof-object))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn eof_object_p_returns_false_for_non_eof_values() {
    assert!(matches!(
        eval_src("(eof-object? 42)"),
        SrsValue::Boolean(false)
    ));
}

#[test]
fn current_input_port_returns_port_value() {
    assert!(matches!(
        eval_src("(current-input-port)"),
        SrsValue::Port(_)
    ));
}

#[test]
fn current_output_port_returns_port_value() {
    assert!(matches!(
        eval_src("(current-output-port)"),
        SrsValue::Port(_)
    ));
}

#[test]
fn current_input_port_is_singleton() {
    assert!(matches!(
        eval_src("(eq? (current-input-port) (current-input-port))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn current_output_port_is_singleton() {
    assert!(matches!(
        eval_src("(eq? (current-output-port) (current-output-port))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn current_input_port_binds_internal_variable() {
    assert!(matches!(
        eval_src("(eq? (current-input-port) *current-input-port*)"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn current_output_port_binds_internal_variable() {
    assert!(matches!(
        eval_src("(eq? (current-output-port) *current-output-port*)"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn read_char_rejects_non_port_argument() {
    assert!(eval_all("(read-char 42)").is_err());
}

#[test]
fn read_char_rejects_too_many_arguments() {
    assert!(eval_all("(read-char (current-input-port) 1)").is_err());
}

#[test]
fn peek_char_rejects_non_port_argument() {
    assert!(eval_all("(peek-char 42)").is_err());
}

#[test]
fn peek_char_rejects_too_many_arguments() {
    assert!(eval_all("(peek-char (current-input-port) 1)").is_err());
}

#[test]
fn read_char_rejects_output_port() {
    assert!(eval_all("(read-char (current-output-port))").is_err());
    assert!(eval_all("(peek-char (current-output-port))").is_err());
}

#[test]
#[ignore = "requires data on stdin; run with `echo -n 'x' | cargo test -- --ignored`"]
fn read_char_on_current_input_port_returns_character_or_eof() {
    // This test exercises the real standard input port. String ports
    // (step 6) will allow a fully automated test.
    let values = read_all(get_lexemes("(read-char)").unwrap()).unwrap();
    let env = global_env();
    let result = eval(&values[0], &env).unwrap();
    assert!(
        matches!(result, SrsValue::Character(_) | SrsValue::Eof),
        "expected #<char> or #<eof>, got {}",
        result
    );
}

#[test]
#[ignore = "requires data on stdin; run with `echo -n 'x' | cargo test -- --ignored`"]
fn peek_char_on_current_input_port_returns_character_or_eof() {
    // See read_char_on_current_input_port_returns_character_or_eof.
    let values = read_all(get_lexemes("(peek-char)").unwrap()).unwrap();
    let env = global_env();
    let result = eval(&values[0], &env).unwrap();
    assert!(
        matches!(result, SrsValue::Character(_) | SrsValue::Eof),
        "expected #<char> or #<eof>, got {}",
        result
    );
}

#[test]
#[ignore = "requires data on stdin; run twice with a single-char input: peek then read should match"]
fn peek_then_read_return_same_character() {
    let env = global_env();
    let first = eval(
        &read_all(get_lexemes("(peek-char)").unwrap()).unwrap()[0],
        &env,
    )
    .unwrap();
    let second = eval(
        &read_all(get_lexemes("(read-char)").unwrap()).unwrap()[0],
        &env,
    )
    .unwrap();
    match (&first, &second) {
        (SrsValue::Character(a), SrsValue::Character(b)) if a == b => {}
        (SrsValue::Eof, SrsValue::Eof) => {}
        _ => panic!(
            "expected peek and read to return the same character, got {} and {}",
            first, second
        ),
    }
}
