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

#[test]
fn sin_of_zero() {
    match eval_src("(sin 0)") {
        SrsValue::Float(f) => assert!((f - 0.0).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn cos_of_zero() {
    match eval_src("(cos 0)") {
        SrsValue::Float(f) => assert!((f - 1.0).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn tan_of_zero() {
    match eval_src("(tan 0)") {
        SrsValue::Float(f) => assert!((f - 0.0).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn atan_of_zero() {
    match eval_src("(atan 0)") {
        SrsValue::Float(f) => assert!((f - 0.0).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn sin_of_one() {
    match eval_src("(sin 1)") {
        SrsValue::Float(f) => assert!((f - 1f64.sin()).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn cos_of_one() {
    match eval_src("(cos 1)") {
        SrsValue::Float(f) => assert!((f - 1f64.cos()).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn tan_of_one() {
    match eval_src("(tan 1)") {
        SrsValue::Float(f) => assert!((f - 1f64.tan()).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn atan_of_one() {
    match eval_src("(atan 1)") {
        SrsValue::Float(f) => assert!((f - 1f64.atan()).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn nan_true_for_nan_float() {
    assert!(matches!(
        eval_src("(nan? (/ 0.0 0.0))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn nan_false_for_ordinary_float() {
    assert!(matches!(eval_src("(nan? 1.0)"), SrsValue::Boolean(false)));
}

#[test]
fn nan_false_for_infinity() {
    assert!(matches!(
        eval_src("(nan? (/ 1.0 0.0))"),
        SrsValue::Boolean(false)
    ));
}

#[test]
fn nan_false_for_integer() {
    assert!(matches!(eval_src("(nan? 3)"), SrsValue::Boolean(false)));
}

#[test]
fn finite_true_for_ordinary_float() {
    assert!(matches!(eval_src("(finite? 1.0)"), SrsValue::Boolean(true)));
}

#[test]
fn finite_true_for_integer() {
    assert!(matches!(eval_src("(finite? 3)"), SrsValue::Boolean(true)));
}

#[test]
fn finite_false_for_infinity() {
    assert!(matches!(
        eval_src("(finite? (/ 1.0 0.0))"),
        SrsValue::Boolean(false)
    ));
}

#[test]
fn finite_false_for_nan() {
    assert!(matches!(
        eval_src("(finite? (/ 0.0 0.0))"),
        SrsValue::Boolean(false)
    ));
}
