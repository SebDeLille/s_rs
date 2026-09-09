use std::rc::Rc;

use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::{Env, SrsValue};

const FACTORIEL_SCM: &str = include_str!("fixtures/scheme/factoriel.scm");

fn eval_src(src: &str, env: &Rc<Env>) -> SrsValue {
    let values = read_all(get_lexemes(src).unwrap()).unwrap();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, env).unwrap_or_else(|e| panic!("eval error: {}", e));
    }
    result
}

#[test]
fn factoriel_fixture_reads_without_error() {
    let lexemes = get_lexemes(FACTORIEL_SCM).unwrap();
    let result = read_all(lexemes);
    assert!(result.is_ok(), "factoriel.scm failed to read: {:?}", result);
}

#[test]
fn factoriel_fixture_defines_fact() {
    let lexemes = get_lexemes(FACTORIEL_SCM).unwrap();
    let datums = read_all(lexemes).unwrap();

    assert_eq!(datums.len(), 1);
    let define = &datums[0];
    match define {
        SrsValue::Pair(cell) => {
            let (car, _) = cell.borrow().clone();
            assert!(matches!(&car, SrsValue::Symbol(s) if s == "define"));
        }
        other => panic!("expected pair, got {:?}", other),
    }
}

#[test]
fn factoriel_fixture_evaluates_correctly() {
    let env = global_env();
    eval_src(FACTORIEL_SCM, &env);

    assert!(matches!(eval_src("(fact 0)", &env), SrsValue::Integer(1)));
    assert!(matches!(eval_src("(fact 1)", &env), SrsValue::Integer(1)));
    assert!(matches!(eval_src("(fact 5)", &env), SrsValue::Integer(120)));
    assert!(matches!(
        eval_src("(fact 10)", &env),
        SrsValue::Integer(3_628_800)
    ));
}

#[test]
fn factoriel_fixture_loadable_via_load_special_form() {
    let env = global_env();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/scheme/factoriel.scm"
    );
    let src = format!("(load \"{}\") (fact 6)", path);
    assert!(matches!(eval_src(&src, &env), SrsValue::Integer(720)));
}
