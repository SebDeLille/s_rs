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
fn lambda_expression_evaluates_to_a_procedure() {
    assert!(matches!(
        eval_src("(lambda (x) (+ x x))"),
        SrsValue::Procedure(_)
    ));
}

#[test]
fn immediately_applied_lambda() {
    assert!(matches!(
        eval_src("((lambda (x) (+ x x)) 4)"),
        SrsValue::Integer(8)
    ));
}

#[test]
fn define_bound_lambda_reverse_subtract() {
    let src = "(define reverse-subtract (lambda (x y) (- y x))) (reverse-subtract 7 10)";
    assert!(matches!(eval_src(src), SrsValue::Integer(3)));
}

#[test]
fn let_captures_enclosing_binding_in_closure() {
    let src = "(define add4 (let ((x 4)) (lambda (y) (+ x y)))) (add4 6)";
    assert!(matches!(eval_src(src), SrsValue::Integer(10)));
}
