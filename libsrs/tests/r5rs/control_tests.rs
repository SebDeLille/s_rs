use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

fn eval_all(scm: &str) -> Result<SrsValue, String> {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let env = global_env();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, &env).map_err(|e| e.to_string())?;
    }
    Ok(result)
}

fn eval_src(scm: &str) -> SrsValue {
    eval_all(scm).unwrap()
}

fn string_value(value: SrsValue) -> String {
    match value {
        SrsValue::String(s) => s.borrow().clone(),
        other => panic!("expected string, got {}", other),
    }
}

#[test]
fn dynamic_wind_runs_before_thunk_after_in_order() {
    let src = r#"
        (define out (open-output-string))
        (dynamic-wind
          (lambda () (display "b" out))
          (lambda () (display "t" out) 42)
          (lambda () (display "a" out)))
        (get-output-string out)
"#;
    assert_eq!(string_value(eval_src(src)), "bta");
}

#[test]
fn dynamic_wind_returns_thunk_value() {
    let src = "(dynamic-wind (lambda () 'ok) (lambda () (* 6 7)) (lambda () 'done))";
    match eval_src(src) {
        SrsValue::Integer(n) => assert_eq!(n, 42),
        other => panic!("expected integer 42, got {}", other),
    }
}

#[test]
fn dynamic_wind_runs_after_even_if_thunk_errors() {
    let src = r#"
        (define out (open-output-string))
        (dynamic-wind
          (lambda () 'ok)
          (lambda () (/ 1 0))
          (lambda () (display "clean" out)))
"#;
    let err = eval_all(src).expect_err("expected division by zero error");
    assert!(err.contains("division by zero"), "unexpected error: {err}");
}

#[test]
fn dynamic_wind_with_too_few_arguments_errors() {
    let err = eval_all("(dynamic-wind (lambda () 'a) (lambda () 'b))").unwrap_err();
    assert!(err.contains("not enough arguments to dynamic-wind"), "unexpected error: {err}");
}

#[test]
fn dynamic_wind_with_too_many_arguments_errors() {
    let err = eval_all("(dynamic-wind (lambda () 'a) (lambda () 'b) (lambda () 'c) 'd)")
        .unwrap_err();
    assert!(err.contains("too many arguments to dynamic-wind"), "unexpected error: {err}");
}
