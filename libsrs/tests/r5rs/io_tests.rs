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
