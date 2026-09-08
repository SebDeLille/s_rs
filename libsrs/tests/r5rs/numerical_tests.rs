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
fn add_two_integers() {
    assert!(matches!(eval_src("(+ 3 4)"), SrsValue::Integer(7)));
}

#[test]
fn add_single_integer() {
    assert!(matches!(eval_src("(+ 3)"), SrsValue::Integer(3)));
}

#[test]
fn add_no_args_returns_identity() {
    assert!(matches!(eval_src("(+)"), SrsValue::Integer(0)));
}

#[test]
fn multiply_single_integer() {
    assert!(matches!(eval_src("(* 4)"), SrsValue::Integer(4)));
}

#[test]
fn multiply_no_args_returns_identity() {
    assert!(matches!(eval_src("(*)"), SrsValue::Integer(1)));
}
