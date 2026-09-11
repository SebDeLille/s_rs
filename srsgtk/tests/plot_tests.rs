use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

const FUNCTION_SAMPLES_SCM: &str = include_str!("../../libs/function-samples.scm");
const PLOT_SCM: &str = include_str!("../../libs/plot.scm");

fn eval_all(scm: &str, env: &std::rc::Rc<libsrs::types::core::Env>) -> SrsValue {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, env).unwrap();
    }
    result
}

fn env_with_plot() -> std::rc::Rc<libsrs::types::core::Env> {
    let env = global_env();
    eval_all(FUNCTION_SAMPLES_SCM, &env);
    eval_all(PLOT_SCM, &env);
    env
}

fn as_f64(v: &SrsValue) -> f64 {
    match v {
        SrsValue::Float(f) => *f,
        SrsValue::Integer(n) => *n as f64,
        other => panic!("expected number, got {:?}", other),
    }
}

#[test]
fn scale_x_maps_bounds_to_canvas_with_margins() {
    let env = env_with_plot();
    let result = eval_all("(plot-scale-x 0.5 0.0 1.0 10 200)", &env);
    let x = as_f64(&result);
    assert!((x - 100.0).abs() < 1e-9);
}

#[test]
fn scale_y_inverts_axis_for_cairo_origin_top_left() {
    let env = env_with_plot();
    // y at ymin must map to the lower border (canvas-h - margin).
    let ymin_px = eval_all("(plot-scale-y 0.0 0.0 10.0 10 200)", &env);
    assert!((as_f64(&ymin_px) - 190.0).abs() < 1e-9);
    // y at ymax must map to the upper border (margin).
    let ymax_px = eval_all("(plot-scale-y 10.0 0.0 10.0 10 200)", &env);
    assert!((as_f64(&ymax_px) - 10.0).abs() < 1e-9);
}

#[test]
fn bounds_over_all_segments() {
    let env = env_with_plot();
    eval_all("(define f (lambda (x) (* x x)))", &env);
    let bounds = eval_all("(plot-bounds (function-samples f -2.0 2.0 5))", &env);
    let values: Vec<f64> = match bounds {
        SrsValue::Pair(_) => {
            let mut items = Vec::new();
            let mut cur = bounds.clone();
            loop {
                match cur {
                    SrsValue::Nil => break,
                    SrsValue::Pair(cell) => {
                        let (car, cdr) = cell.borrow().clone();
                        items.push(as_f64(&car));
                        cur = cdr;
                    }
                    _ => panic!("improper list"),
                }
            }
            items
        }
        _ => panic!("expected list, got {:?}", bounds),
    };
    assert_eq!(values.len(), 4);
    assert!((values[0] - (-2.0)).abs() < 1e-9);
    assert!((values[1] - 2.0).abs() < 1e-9);
    assert!((values[2] - 0.0).abs() < 1e-9);
    assert!((values[3] - 4.0).abs() < 1e-9);
}

#[test]
fn empty_bounds_fallback() {
    let env = env_with_plot();
    let result = eval_all("(plot-bounds ())", &env);
    let values: Vec<f64> = match &result {
        SrsValue::Pair(_) => {
            let mut items = Vec::new();
            let mut cur = result.clone();
            loop {
                match cur {
                    SrsValue::Nil => break,
                    SrsValue::Pair(cell) => {
                        let (car, cdr) = cell.borrow().clone();
                        items.push(as_f64(&car));
                        cur = cdr;
                    }
                    _ => panic!("improper list"),
                }
            }
            items
        }
        _ => panic!("expected list, got {:?}", result),
    };
    assert_eq!(values, vec![-1.0, 1.0, -1.0, 1.0]);
}
