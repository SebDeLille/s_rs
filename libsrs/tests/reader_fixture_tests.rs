use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

const MATRIX_SCM: &str = include_str!("fixtures/scheme/matrix.scm");

fn car(v: &SrsValue) -> SrsValue {
    match v {
        SrsValue::Pair(cell) => cell.borrow().0.clone(),
        other => panic!("expected pair, got {:?}", other),
    }
}

fn cdr(v: &SrsValue) -> SrsValue {
    match v {
        SrsValue::Pair(cell) => cell.borrow().1.clone(),
        other => panic!("expected pair, got {:?}", other),
    }
}

#[test]
fn matrix_fixture_reads_without_error() {
    let lexemes = get_lexemes(MATRIX_SCM).unwrap();
    let result = read_all(lexemes);
    assert!(result.is_ok(), "matrix.scm failed to read: {:?}", result);
}

#[test]
fn matrix_fixture_has_expected_top_level_shape() {
    let lexemes = get_lexemes(MATRIX_SCM).unwrap();
    let datums = read_all(lexemes).unwrap();

    // Every top-level form in matrix.scm is a `(define name ...)`.
    assert!(!datums.is_empty());
    for datum in &datums {
        assert!(matches!(datum, SrsValue::Pair(_)));
        assert!(matches!(&car(datum), SrsValue::Symbol(s) if s == "define"));
    }

    // First form: (define make-matrix (lambda (rows columns) ...))
    let first = &datums[0];
    let name = car(&cdr(first));
    assert!(matches!(&name, SrsValue::Symbol(s) if s == "make-matrix"));

    let lambda_expr = car(&cdr(&cdr(first)));
    let lambda_head = car(&lambda_expr);
    assert!(matches!(&lambda_head, SrsValue::Symbol(s) if s == "lambda"));
}
