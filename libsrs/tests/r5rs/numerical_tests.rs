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

fn eval_src_result(scm: &str) -> Result<SrsValue, libsrs::interpretor::evaluator::EvalError> {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let env = global_env();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, &env)?;
    }
    Ok(result)
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

#[test]
fn abs_and_min_preserve_numeric_types() {
    assert!(matches!(eval_src("(abs -7)"), SrsValue::Integer(7)));
    assert!(matches!(eval_src("(abs -7.5)"), SrsValue::Float(n) if n == 7.5));
    assert!(matches!(eval_src("(min 3 4)"), SrsValue::Integer(3)));
    assert!(matches!(eval_src("(min 3.9 4)"), SrsValue::Float(n) if n == 3.9));
    assert!(matches!(eval_src("(min 4 3.9)"), SrsValue::Float(n) if n == 3.9));
}

#[test]
fn quotient_remainder_and_modulo_follow_scheme_sign_rules() {
    assert!(matches!(eval_src("(quotient 7 2)"), SrsValue::Integer(3)));
    assert!(matches!(eval_src("(remainder 13 4)"), SrsValue::Integer(1)));
    assert!(matches!(
        eval_src("(remainder -13 4)"),
        SrsValue::Integer(-1)
    ));
    assert!(matches!(
        eval_src("(remainder 13 -4)"),
        SrsValue::Integer(1)
    ));
    assert!(matches!(
        eval_src("(remainder -13 -4)"),
        SrsValue::Integer(-1)
    ));
    assert!(matches!(eval_src("(remainder -13 -4.0)"), SrsValue::Float(n) if n == -1.0));
    assert!(matches!(eval_src("(modulo 13 4)"), SrsValue::Integer(1)));
    assert!(matches!(eval_src("(modulo -13 4)"), SrsValue::Integer(3)));
    assert!(matches!(eval_src("(modulo 13 -4)"), SrsValue::Integer(-3)));
    assert!(matches!(eval_src("(modulo -13 -4)"), SrsValue::Integer(-1)));
}

#[test]
fn integer_division_by_zero_returns_scheme_errors() {
    assert!(eval_src_result("(quotient 1 0)").is_err());
    assert!(eval_src_result("(modulo 1 0)").is_err());
}

#[test]
fn numeric_type_and_sign_predicates() {
    assert!(matches!(eval_src("(zero? 0)"), SrsValue::Boolean(true)));
    assert!(matches!(eval_src("(zero? -0.0)"), SrsValue::Boolean(true)));
    assert!(matches!(eval_src("(positive? 2)"), SrsValue::Boolean(true)));
    assert!(matches!(
        eval_src("(negative? -2)"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(eval_src("(even? -4)"), SrsValue::Boolean(true)));
    assert!(matches!(eval_src("(odd? -3)"), SrsValue::Boolean(true)));
    assert!(matches!(eval_src("(number? 3)"), SrsValue::Boolean(true)));
    assert!(matches!(eval_src("(number? 'a)"), SrsValue::Boolean(false)));
    assert!(matches!(
        eval_src("(integer? 3.0)"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(integer? (string->number \"8/4\"))"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(integer? 3.5)"),
        SrsValue::Boolean(false)
    ));
}
