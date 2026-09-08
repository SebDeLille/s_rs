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

fn as_vec(value: &SrsValue) -> Vec<SrsValue> {
    match value {
        SrsValue::Vector(cell) => cell.borrow().clone(),
        other => panic!("expected vector, got {:?}", other),
    }
}

#[test]
fn vector_literal_self_evaluates() {
    let items = as_vec(&eval_src("#(1 2 3)"));
    assert!(matches!(items[0], SrsValue::Integer(1)));
    assert!(matches!(items[1], SrsValue::Integer(2)));
    assert!(matches!(items[2], SrsValue::Integer(3)));
}

#[test]
fn vector_builds_a_vector_from_its_arguments() {
    let items = as_vec(&eval_src("(vector 1 2 3)"));
    assert_eq!(items.len(), 3);
    assert!(matches!(items[0], SrsValue::Integer(1)));
}

#[test]
fn vector_p_recognizes_vectors() {
    assert!(matches!(
        eval_src("(vector? (vector 1 2))"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(vector? (cons 1 2))"),
        SrsValue::Boolean(false)
    ));
}

#[test]
fn make_vector_defaults_to_zero_fill() {
    let items = as_vec(&eval_src("(make-vector 3)"));
    assert_eq!(items.len(), 3);
    assert!(items.iter().all(|v| matches!(v, SrsValue::Integer(0))));
}

#[test]
fn make_vector_uses_given_fill() {
    let items = as_vec(&eval_src("(make-vector 3 'a)"));
    assert_eq!(items.len(), 3);
    assert!(
        items
            .iter()
            .all(|v| matches!(v, SrsValue::Symbol(s) if s == "a"))
    );
}

#[test]
fn vector_length_returns_the_element_count() {
    assert!(matches!(
        eval_src("(vector-length (vector 1 2 3))"),
        SrsValue::Integer(3)
    ));
}

#[test]
fn vector_ref_returns_the_element_at_index() {
    assert!(matches!(
        eval_src("(vector-ref (vector 10 20 30) 1)"),
        SrsValue::Integer(20)
    ));
}

#[test]
fn vector_ref_out_of_range_is_an_error() {
    let values = read_all(get_lexemes("(vector-ref (vector 1 2) 5)").unwrap()).unwrap();
    let env = global_env();
    assert!(eval(&values[0], &env).is_err());
}

#[test]
fn vector_set_mutates_in_place() {
    let src = "(define v (vector 1 2 3)) (vector-set! v 1 99) (vector-ref v 1)";
    assert!(matches!(eval_src(src), SrsValue::Integer(99)));
}

#[test]
fn vector_to_list_converts_elements_in_order() {
    let src = "(vector->list (vector 1 2 3))";
    let result = eval_src(src);
    match result {
        SrsValue::Pair(_) | SrsValue::Nil => {
            let mut items = Vec::new();
            let mut current = result;
            loop {
                match current {
                    SrsValue::Nil => break,
                    SrsValue::Pair(cell) => {
                        let (car, cdr) = cell.borrow().clone();
                        items.push(car);
                        current = cdr;
                    }
                    _ => panic!("expected proper list"),
                }
            }
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], SrsValue::Integer(1)));
        }
        other => panic!("expected list, got {:?}", other),
    }
}

#[test]
fn list_to_vector_converts_elements_in_order() {
    let items = as_vec(&eval_src("(list->vector '(1 2 3))"));
    assert_eq!(items.len(), 3);
    assert!(matches!(items[2], SrsValue::Integer(3)));
}

#[test]
fn vector_fill_sets_every_element() {
    let src = "(define v (vector 1 2 3)) (vector-fill! v 7) v";
    let items = as_vec(&eval_src(src));
    assert!(items.iter().all(|v| matches!(v, SrsValue::Integer(7))));
}
