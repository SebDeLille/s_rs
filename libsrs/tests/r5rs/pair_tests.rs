use libsrs::interpretor::evaluator::{eval, global_env, EvalError};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

fn eval_src(scm: &str) -> SrsValue {
    eval_src_result(scm).unwrap()
}

fn eval_src_result(scm: &str) -> Result<SrsValue, EvalError> {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let env = global_env();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, &env)?;
    }
    Ok(result)
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

#[test]
fn null_returns_true_for_the_empty_list() {
    assert!(matches!(eval_src("(null? '())"), SrsValue::Boolean(true)));
}

#[test]
fn null_returns_false_for_a_non_empty_list() {
    assert!(matches!(eval_src("(null? '(1))"), SrsValue::Boolean(false)));
}

#[test]
fn null_returns_false_for_a_pair() {
    assert!(matches!(
        eval_src("(null? (cons 1 2))"),
        SrsValue::Boolean(false)
    ));
}

#[test]
fn pair_returns_true_for_a_pair() {
    assert!(matches!(
        eval_src("(pair? (cons 1 2))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn pair_returns_false_for_the_empty_list() {
    assert!(matches!(eval_src("(pair? '())"), SrsValue::Boolean(false)));
}

#[test]
fn pair_returns_false_for_an_atom() {
    assert!(matches!(eval_src("(pair? 42)"), SrsValue::Boolean(false)));
}

#[test]
fn list_returns_the_empty_list_when_given_no_arguments() {
    assert!(matches!(eval_src("(list)"), SrsValue::Nil));
}

#[test]
fn list_builds_a_proper_list_from_arguments() {
    assert!(matches!(
        eval_src("(car (list 1 2 3))"),
        SrsValue::Integer(1)
    ));
    assert!(matches!(
        eval_src("(car (cdr (list 1 2 3)))"),
        SrsValue::Integer(2)
    ));
    assert!(matches!(
        eval_src("(length (list 1 2 3))"),
        SrsValue::Integer(3)
    ));
}

#[test]
fn reverse_reverses_a_proper_list() {
    let src = "(reverse '(1 2 3))";
    assert!(matches!(eval_src(src), SrsValue::Pair(_)));
    assert!(matches!(
        eval_src("(car (reverse '(1 2 3)))"),
        SrsValue::Integer(3)
    ));
    assert!(matches!(
        eval_src("(car (cdr (reverse '(1 2 3))))"),
        SrsValue::Integer(2)
    ));
    assert!(matches!(
        eval_src("(car (cdr (cdr (reverse '(1 2 3)))))"),
        SrsValue::Integer(1)
    ));
}

#[test]
fn reverse_returns_the_empty_list_for_the_empty_list() {
    assert!(matches!(eval_src("(reverse '())"), SrsValue::Nil));
}

#[test]
fn composed_cxr_level_2() {
    // (cadr '((1 2) (3 4) (5 6))) => car(cdr '((1 2) (3 4) (5 6))) => (3 4)
    assert!(matches!(
        eval_src("(cadr '((1 2) (3 4) (5 6)))"),
        SrsValue::Pair(_)
    ));
    // (cdar '((1 2) 3)) => cdr(car) => (2)
    assert!(matches!(eval_src("(cdar '((1 2) 3))"), SrsValue::Pair(_)));
}

#[test]
fn composed_cxr_level_3() {
    // (caddr '(1 2 3 4)) => 3
    assert!(matches!(
        eval_src("(caddr '(1 2 3 4))"),
        SrsValue::Integer(3)
    ));
    // (caaar '(((42)))) => 42
    assert!(matches!(
        eval_src("(caaar '(((42))))"),
        SrsValue::Integer(42)
    ));
}

#[test]
fn composed_cxr_level_4() {
    // (cadddr '(1 2 3 4 5)) => 4
    assert!(matches!(
        eval_src("(cadddr '(1 2 3 4 5))"),
        SrsValue::Integer(4)
    ));
    // (cddddr '(1 2 3 4 5)) => (5)
    assert!(matches!(
        eval_src("(cddddr '(1 2 3 4 5))"),
        SrsValue::Pair(_)
    ));
}

#[test]
fn composed_cxr_errors_on_non_pair() {
    assert!(eval_src_result("(cadr 1)").is_err());
}

#[test]
fn list_ref_returns_the_kth_element() {
    assert!(matches!(
        eval_src("(list-ref '(a b c d) 0)"),
        SrsValue::Symbol(s) if s == "a"
    ));
    assert!(matches!(
        eval_src("(list-ref '(a b c d) 2)"),
        SrsValue::Symbol(s) if s == "c"
    ));
}

#[test]
fn list_ref_errors_when_index_out_of_bounds() {
    assert!(eval_src_result("(list-ref '(a b c d) 4)").is_err());
}

#[test]
fn list_ref_errors_on_negative_index() {
    assert!(eval_src_result("(list-ref '(a b c) -1)").is_err());
}

#[test]
fn list_ref_errors_on_improper_list() {
    assert!(eval_src_result("(list-ref (cons 1 2) 0)").is_err());
}

#[test]
fn append_returns_the_empty_list_with_no_arguments() {
    assert!(matches!(eval_src("(append)"), SrsValue::Nil));
}

#[test]
fn append_returns_the_argument_unchanged_with_one_argument() {
    assert!(matches!(
        eval_src("(length (append '(1 2 3)))"),
        SrsValue::Integer(3)
    ));
}

#[test]
fn append_concatenates_several_lists() {
    let src = "(append '(1 2) '(3 4) '(5 6))";
    assert!(matches!(
        eval_src("(length (append '(1 2) '(3 4) '(5 6)))"),
        SrsValue::Integer(6)
    ));
    assert!(matches!(
        eval_src(&format!("(car {src})")),
        SrsValue::Integer(1)
    ));
    assert!(matches!(
        eval_src(&format!("(list-ref {src} 5)")),
        SrsValue::Integer(6)
    ));
}

#[test]
fn append_treats_empty_lists_as_identity() {
    assert!(matches!(
        eval_src("(length (append '() '(1)))"),
        SrsValue::Integer(1)
    ));
    assert!(matches!(
        eval_src("(length (append '(1) '()))"),
        SrsValue::Integer(1)
    ));
}

#[test]
fn append_uses_the_last_argument_as_is_even_if_improper() {
    assert!(matches!(
        eval_src("(cdr (cdr (append '(1 2) 3)))"),
        SrsValue::Integer(3)
    ));
}
