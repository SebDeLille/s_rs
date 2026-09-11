use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

const FUNCTION_SAMPLES_SCM: &str = include_str!("../../libs/function-samples.scm");

fn eval_all(scm: &str, env: &std::rc::Rc<libsrs::types::core::Env>) -> SrsValue {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, env).unwrap();
    }
    result
}

fn segments_shape(value: &SrsValue) -> Vec<Vec<(f64, f64)>> {
    fn to_vec(list: &SrsValue) -> Vec<SrsValue> {
        let mut items = Vec::new();
        let mut cur = list.clone();
        loop {
            match cur {
                SrsValue::Nil => break,
                SrsValue::Pair(cell) => {
                    let (car, cdr) = cell.borrow().clone();
                    items.push(car);
                    cur = cdr;
                }
                _ => panic!("improper list"),
            }
        }
        items
    }

    fn as_f64(v: &SrsValue) -> f64 {
        match v {
            SrsValue::Float(f) => *f,
            SrsValue::Integer(n) => *n as f64,
            other => panic!("expected number, got {:?}", other),
        }
    }

    to_vec(value)
        .iter()
        .map(|segment| {
            to_vec(segment)
                .iter()
                .map(|pair| match pair {
                    SrsValue::Pair(cell) => {
                        let (car, cdr) = cell.borrow().clone();
                        (as_f64(&car), as_f64(&cdr))
                    }
                    other => panic!("expected pair, got {:?}", other),
                })
                .collect()
        })
        .collect()
}

fn env_with_prelude() -> std::rc::Rc<libsrs::types::core::Env> {
    let env = global_env();
    eval_all(FUNCTION_SAMPLES_SCM, &env);
    env
}

#[test]
fn nominal_case_without_discontinuity_yields_a_single_segment() {
    let env = env_with_prelude();
    eval_all("(define g (lambda (x) (* x x)))", &env);
    let result = eval_all("(function-samples g 0.0 4.0 5)", &env);
    let segments = segments_shape(&result);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].len(), 5);
    let expected_xs = [0.0, 1.0, 2.0, 3.0, 4.0];
    for (point, expected_x) in segments[0].iter().zip(expected_xs) {
        assert!((point.0 - expected_x).abs() < 1e-9);
        assert!((point.1 - expected_x * expected_x).abs() < 1e-9);
    }
}

#[test]
fn discontinuity_splits_into_two_segments() {
    // Simulates 1/x sampled across x = 0: the sample at x = 0 is
    // +/-infinity (Float 0.0 divisor, not an Integer division-by-zero
    // error) and must be dropped, splitting the segment in two.
    let env = env_with_prelude();
    eval_all("(define recip (lambda (x) (/ 1.0 x)))", &env);
    let result = eval_all("(function-samples recip -2.0 2.0 5)", &env);
    let segments = segments_shape(&result);
    // x values sampled: -2, -1, 0, 1, 2 -> x = 0 is dropped.
    assert_eq!(segments.len(), 2, "segments: {:?}", segments);
    assert_eq!(segments[0].len(), 2);
    assert_eq!(segments[1].len(), 2);
    assert!((segments[0][0].0 - (-2.0)).abs() < 1e-9);
    assert!((segments[0][1].0 - (-1.0)).abs() < 1e-9);
    assert!((segments[1][0].0 - 1.0).abs() < 1e-9);
    assert!((segments[1][1].0 - 2.0).abs() < 1e-9);
}

#[test]
fn nan_result_is_filtered_out() {
    // Simulates a function returning NaN for a single sample (e.g. 0/0),
    // which must be dropped like any other non-finite result.
    let env = env_with_prelude();
    eval_all(
        "(define maybe-nan (lambda (x) (if (= x 0.0) (/ 0.0 0.0) x)))",
        &env,
    );
    let result = eval_all("(function-samples maybe-nan -1.0 1.0 3)", &env);
    let segments = segments_shape(&result);
    assert_eq!(segments.len(), 2, "segments: {:?}", segments);
    assert_eq!(segments[0].len(), 1);
    assert_eq!(segments[1].len(), 1);
}

#[test]
fn single_sample_point() {
    let env = env_with_prelude();
    eval_all("(define g (lambda (x) (* x 2)))", &env);
    let result = eval_all("(function-samples g 1.0 1.0 1)", &env);
    let segments = segments_shape(&result);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].len(), 1);
    assert!((segments[0][0].0 - 1.0).abs() < 1e-9);
    assert!((segments[0][0].1 - 2.0).abs() < 1e-9);
}

#[test]
fn zero_samples_yields_no_segments() {
    let env = env_with_prelude();
    eval_all("(define g (lambda (x) x))", &env);
    let result = eval_all("(function-samples g 0.0 1.0 0)", &env);
    let segments = segments_shape(&result);
    assert!(segments.is_empty());
}
