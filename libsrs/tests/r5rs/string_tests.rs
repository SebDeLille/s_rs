use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

fn eval_src(scm: &str) -> SrsValue {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let env = global_env();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, &env).unwrap();
    }
    result
}

fn as_str(value: &SrsValue) -> String {
    match value {
        SrsValue::String(cell) => cell.borrow().clone(),
        other => panic!("expected string, got {:?}", other),
    }
}

#[test]
fn string_p_recognizes_strings() {
    assert!(matches!(
        eval_src("(string? \"hello\")"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(string? 123)"),
        SrsValue::Boolean(false)
    ));
}

#[test]
fn string_length_returns_character_count() {
    assert!(matches!(
        eval_src("(string-length \"hello\")"),
        SrsValue::Integer(5)
    ));
}

#[test]
fn string_ref_returns_character_at_index() {
    assert!(matches!(
        eval_src("(string-ref \"hello\" 1)"),
        SrsValue::Character('e')
    ));
}

#[test]
fn string_ref_out_of_range_is_an_error() {
    let values = read_all(get_lexemes("(string-ref \"hi\" 5)").unwrap()).unwrap();
    let env = global_env();
    assert!(eval(&values[0], &env).is_err());
}

#[test]
fn string_eq_compares_character_sequences() {
    assert!(matches!(
        eval_src("(string=? \"abc\" \"abc\")"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(string=? \"abc\" \"def\")"),
        SrsValue::Boolean(false)
    ));
    assert!(matches!(
        eval_src("(string=? \"a\" \"a\" \"a\")"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn substring_extracts_slice() {
    assert_eq!(as_str(&eval_src("(substring \"hello\" 1 4)")), "ell");
}

#[test]
fn substring_start_equals_end_returns_empty() {
    assert_eq!(as_str(&eval_src("(substring \"hello\" 2 2)")), "");
}

#[test]
fn string_append_concatenates_arguments() {
    assert_eq!(
        as_str(&eval_src("(string-append \"foo\" \"bar\")")),
        "foobar"
    );
}

#[test]
fn string_append_with_no_arguments_returns_empty_string() {
    assert_eq!(as_str(&eval_src("(string-append)")), "");
}

#[test]
fn list_to_string_converts_characters() {
    assert_eq!(as_str(&eval_src("(list->string '(#\\f #\\o #\\o))")), "foo");
}
