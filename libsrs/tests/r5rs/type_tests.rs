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

fn eval_error(scm: &str) -> String {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let env = global_env();
    eval(&values[0], &env).unwrap_err().to_string()
}

#[test]
fn symbol_predicate_and_conversions_round_trip_case_exactly() {
    assert!(matches!(
        eval_src("(symbol? 'foo)"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(symbol? '())"),
        SrsValue::Boolean(false)
    ));
    assert_eq!(
        eval_src("(symbol->string 'flying-fish)").to_string(),
        "\"flying-fish\""
    );
    assert_eq!(
        eval_src("(symbol->string (string->symbol \"Malvina\"))").to_string(),
        "\"Malvina\""
    );
    assert!(matches!(
        eval_src("(eq? 'JollyWog (string->symbol (symbol->string 'JollyWog)))"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src(
            "(string=? \"K. Harper, M.D.\" (symbol->string (string->symbol \"K. Harper, M.D.\")))"
        ),
        SrsValue::Boolean(true)
    ));
    assert!(eval_error("(symbol->string 1)").contains("expected symbol"));
    assert!(eval_error("(string->symbol 1)").contains("expected string"));
}

#[test]
fn boolean_and_procedure_predicates_recognize_only_their_types() {
    assert!(matches!(eval_src("(boolean? #f)"), SrsValue::Boolean(true)));
    assert!(matches!(eval_src("(boolean? 0)"), SrsValue::Boolean(false)));
    assert!(matches!(
        eval_src("(boolean? '())"),
        SrsValue::Boolean(false)
    ));
    assert!(matches!(
        eval_src("(procedure? car)"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(procedure? 'car)"),
        SrsValue::Boolean(false)
    ));
    assert!(matches!(
        eval_src("(procedure? (lambda (x) (* x x)))"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(procedure? '(lambda (x) (* x x)))"),
        SrsValue::Boolean(false)
    ));
}
