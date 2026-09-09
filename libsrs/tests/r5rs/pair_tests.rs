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
fn cons_builds_a_pair() {
    assert!(matches!(eval_src("(cons 1 2)"), SrsValue::Pair(_)));
}

#[test]
fn car_returns_the_first_element() {
    assert!(matches!(eval_src("(car (cons 1 2))"), SrsValue::Integer(1)));
}

#[test]
fn cdr_returns_the_second_element() {
    assert!(matches!(eval_src("(cdr (cons 1 2))"), SrsValue::Integer(2)));
}

#[test]
fn cons_car_cdr_compose_over_nested_pairs() {
    let src = "(car (cdr (cons 1 (cons 2 3))))";
    assert!(matches!(eval_src(src), SrsValue::Integer(2)));
}

#[test]
fn define_bound_pair_accessors() {
    let src = "(define p (cons 10 20)) (+ (car p) (cdr p))";
    assert!(matches!(eval_src(src), SrsValue::Integer(30)));
}

#[test]
fn length_returns_zero_for_the_empty_list() {
    assert!(matches!(eval_src("(length '())"), SrsValue::Integer(0)));
}

#[test]
fn length_returns_the_element_count() {
    assert!(matches!(
        eval_src("(length '(1 2 3))"),
        SrsValue::Integer(3)
    ));
}

#[test]
fn length_counts_a_hand_built_list() {
    let src = "(length (cons 1 (cons 2 (cons 3 '()))))";
    assert!(matches!(eval_src(src), SrsValue::Integer(3)));
}
