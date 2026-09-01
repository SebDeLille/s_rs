use libsrs::interpretor::evaluator::Evaluator;
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::translator::translate_all;
use libsrs::types::core::SrsValue;

fn eval(scm: &str) -> SrsValue {
    let values = translate_all(get_lexemes(scm).unwrap()).unwrap();
    let evaluator = Evaluator::new();
    evaluator.eval(&values[0]).unwrap()
}

#[test]
fn add_two_integers() {
    assert_eq!(SrsValue::Integer(7), eval("(+ 3 4)"));
}

#[test]
fn add_single_integer() {
    assert_eq!(SrsValue::Integer(3), eval("(+ 3)"));
}

#[test]
fn add_no_args_returns_identity() {
    assert_eq!(SrsValue::Integer(0), eval("(+)"));
}
