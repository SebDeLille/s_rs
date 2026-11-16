use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use libsrs::interpretor::evaluator::{apply, eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::{Env, Native, SrsValue};

const FUNCTION_SAMPLES_SCM: &str = include_str!("../../libs/function-samples.scm");
const PLOT_SCM: &str = include_str!("../../libs/plot.scm");

static HOME_LOCK: Mutex<()> = Mutex::new(());

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rgb(f64, f64, f64);

#[derive(Debug, Clone)]
enum RecordedCommand {
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Rgb,
    },
    Point {
        x: f64,
        y: f64,
        color: Rgb,
    },
}

struct MockCanvas {
    width: i64,
    height: i64,
    current_color: Rgb,
    commands: Vec<RecordedCommand>,
    redraw_hook: Option<SrsValue>,
}

impl MockCanvas {
    fn new(width: i64, height: i64) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            width,
            height,
            current_color: Rgb(1.0, 1.0, 1.0),
            commands: Vec::new(),
            redraw_hook: None,
        }))
    }

    fn resize(state: &Rc<RefCell<Self>>, width: i64, height: i64) {
        let hook = {
            let mut s = state.borrow_mut();
            s.width = width;
            s.height = height;
            s.redraw_hook.clone()
        };
        if let Some(hook) = hook {
            apply(
                &hook,
                &[SrsValue::Integer(width), SrsValue::Integer(height)],
            )
            .unwrap();
        }
    }
}

fn define_native(
    env: &Rc<Env>,
    name: &'static str,
    func: impl Fn(&[SrsValue]) -> Result<SrsValue, String> + 'static,
) {
    env.define(
        name.to_string(),
        SrsValue::Native(Native {
            name,
            func: Rc::new(func),
        }),
    );
}

fn install_mock_canvas(env: &Rc<Env>, canvas: &Rc<RefCell<MockCanvas>>) {
    define_native(env, "clear-canvas", {
        let canvas = canvas.clone();
        move |_| {
            canvas.borrow_mut().commands.clear();
            Ok(SrsValue::Unspecified)
        }
    });
    define_native(env, "set-color", {
        let canvas = canvas.clone();
        move |args| {
            canvas.borrow_mut().current_color =
                Rgb(as_f64(&args[0]), as_f64(&args[1]), as_f64(&args[2]));
            Ok(SrsValue::Unspecified)
        }
    });
    define_native(env, "draw-line", {
        let canvas = canvas.clone();
        move |args| {
            let color = canvas.borrow().current_color;
            canvas.borrow_mut().commands.push(RecordedCommand::Line {
                x1: as_f64(&args[0]),
                y1: as_f64(&args[1]),
                x2: as_f64(&args[2]),
                y2: as_f64(&args[3]),
                color,
            });
            Ok(SrsValue::Unspecified)
        }
    });
    define_native(env, "draw-point", {
        let canvas = canvas.clone();
        move |args| {
            let color = canvas.borrow().current_color;
            canvas.borrow_mut().commands.push(RecordedCommand::Point {
                x: as_f64(&args[0]),
                y: as_f64(&args[1]),
                color,
            });
            Ok(SrsValue::Unspecified)
        }
    });
    define_native(env, "canvas-width", {
        let canvas = canvas.clone();
        move |_| Ok(SrsValue::Integer(canvas.borrow().width))
    });
    define_native(env, "canvas-height", {
        let canvas = canvas.clone();
        move |_| Ok(SrsValue::Integer(canvas.borrow().height))
    });
    define_native(env, "set-redraw-hook!", {
        let canvas = canvas.clone();
        move |args| {
            let hook = args[0].clone();
            let (w, h) = {
                let mut c = canvas.borrow_mut();
                c.redraw_hook = Some(hook.clone());
                (c.width, c.height)
            };
            apply(&hook, &[SrsValue::Integer(w), SrsValue::Integer(h)])
                .map_err(|e| format!("{:?}", e))?;
            Ok(SrsValue::Unspecified)
        }
    });
}

fn with_temp_plot_lib<F: FnOnce()>(f: F) {
    let _guard = HOME_LOCK.lock().unwrap();
    let mut temp_home = std::env::temp_dir();
    temp_home.push(format!(
        "srsgtk-plot-e2e-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let libs_dir = temp_home.join(".config").join("srs").join("libs");
    std::fs::create_dir_all(&libs_dir).unwrap();
    std::fs::write(libs_dir.join("plot.scm"), PLOT_SCM).unwrap();

    let prev_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &temp_home);
    }
    f();
    match prev_home {
        Some(val) => unsafe { std::env::set_var("HOME", val) },
        None => unsafe { std::env::remove_var("HOME") },
    }
    let _ = std::fs::remove_dir_all(&temp_home);
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn curve_lines(commands: &[RecordedCommand]) -> Vec<(f64, f64, f64, f64)> {
    let curve_color = Rgb(0.9, 0.3, 0.2);
    commands
        .iter()
        .filter_map(|cmd| match cmd {
            RecordedCommand::Line {
                x1,
                y1,
                x2,
                y2,
                color,
            } if *color == curve_color => Some((*x1, *y1, *x2, *y2)),
            _ => None,
        })
        .collect()
}

#[test]
fn fixture_plot_cos_draws_axes_continuous_curve_and_resizes() {
    with_temp_plot_lib(|| {
        let env = global_env();
        eval_all(FUNCTION_SAMPLES_SCM, &env);
        let canvas = MockCanvas::new(800, 450);
        install_mock_canvas(&env, &canvas);

        let load_expr = format!(
            "(load {:?})",
            fixture_path("plot_cos.scm").to_string_lossy()
        );
        eval_all(&load_expr, &env);

        {
            let borrowed = canvas.borrow();
            let axes_count = borrowed
                .commands
                .iter()
                .filter(|c| matches!(c, RecordedCommand::Line { color, .. } if *color == Rgb(0.3, 0.3, 0.3)))
                .count();
            assert_eq!(axes_count, 6, "expected 2 axes + 4 boundary ticks");

            let lines = curve_lines(&borrowed.commands);
            assert_eq!(
                lines.len(),
                400,
                "cos over [-6.28, 6.28] should be 1 continuous segment of 400 lines"
            );
            assert!((lines.first().unwrap().0 - 40.0).abs() < 1e-6);
            assert!((lines.last().unwrap().2 - 760.0).abs() < 1e-6);
            for (w1, w2) in lines.windows(2).map(|w| (w[0], w[1])) {
                assert!((w1.2 - w2.0).abs() < 1e-9 && (w1.3 - w2.1).abs() < 1e-9);
            }
        }

        MockCanvas::resize(&canvas, 1200, 700);
        {
            let borrowed = canvas.borrow();
            let lines = curve_lines(&borrowed.commands);
            assert_eq!(lines.len(), 400);
            assert!((lines.first().unwrap().0 - 40.0).abs() < 1e-6);
            assert!((lines.last().unwrap().2 - 1160.0).abs() < 1e-6);
            let min_y = lines
                .iter()
                .map(|l| l.1.min(l.3))
                .fold(f64::INFINITY, f64::min);
            let max_y = lines
                .iter()
                .map(|l| l.1.max(l.3))
                .fold(f64::NEG_INFINITY, f64::max);
            assert!((min_y - 40.0).abs() < 1e-3);
            assert!((max_y - 660.0).abs() < 1e-3);
        }
    });
}

#[test]
fn fixture_plot_inv_splits_across_asymptote_without_parasitic_line() {
    with_temp_plot_lib(|| {
        let env = global_env();
        eval_all(FUNCTION_SAMPLES_SCM, &env);
        let canvas = MockCanvas::new(800, 450);
        install_mock_canvas(&env, &canvas);

        let load_expr = format!(
            "(load {:?})",
            fixture_path("plot_inv.scm").to_string_lossy()
        );
        eval_all(&load_expr, &env);

        let borrowed = canvas.borrow();
        let lines = curve_lines(&borrowed.commands);
        assert!(!lines.is_empty());

        // x = 0 maps to pixel x = 400.0 on an 800px canvas over [-5, 5].
        let mut left_count = 0;
        let mut right_count = 0;
        for (x1, y1, x2, y2) in &lines {
            assert!(x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite());
            assert!(*x1 >= 40.0 - 1e-6 && *x2 <= 760.0 + 1e-6);
            assert!(*y1 >= 40.0 - 1e-6 && *y1 <= 410.0 + 1e-6);
            assert!(*y2 >= 40.0 - 1e-6 && *y2 <= 410.0 + 1e-6);
            assert!(
                (*x1 < 400.0 && *x2 < 400.0) || (*x1 > 400.0 && *x2 > 400.0),
                "parasitic segment crossing asymptote x=0: ({x1}, {y1}) -> ({x2}, {y2})"
            );
            if *x1 < 400.0 {
                left_count += 1;
            } else {
                right_count += 1;
            }
        }
        assert!(left_count > 50 && right_count > 50);
    });
}

#[test]
fn fixture_plot_tan_splits_all_asymptotes_without_parasitic_line() {
    with_temp_plot_lib(|| {
        let env = global_env();
        eval_all(FUNCTION_SAMPLES_SCM, &env);
        let canvas = MockCanvas::new(800, 450);
        install_mock_canvas(&env, &canvas);

        let load_expr = format!(
            "(load {:?})",
            fixture_path("plot_tan.scm").to_string_lossy()
        );
        eval_all(&load_expr, &env);

        let borrowed = canvas.borrow();
        let lines = curve_lines(&borrowed.commands);
        assert!(!lines.is_empty());

        let pi_2 = std::f64::consts::FRAC_PI_2;
        #[allow(clippy::approx_constant)]
        let xmin = -6.28;
        let asymptotes_x: Vec<f64> = [-3.0 * pi_2, -pi_2, pi_2, 3.0 * pi_2]
            .into_iter()
            .map(|x| 40.0 + (720.0 / 12.56) * (x - xmin))
            .collect();

        for (x1, y1, x2, y2) in &lines {
            assert!(*y1 >= 40.0 - 1e-6 && *y1 <= 410.0 + 1e-6);
            assert!(*y2 >= 40.0 - 1e-6 && *y2 <= 410.0 + 1e-6);
            for ax in &asymptotes_x {
                assert!(
                    !(*x1 <= *ax && *ax <= *x2),
                    "parasitic segment crossing tan asymptote at px {ax}: ({x1}, {y1}) -> ({x2}, {y2})"
                );
            }
        }
    });
}

#[test]
fn asymmetric_interval_inv_splits_asymptote_when_zero_not_sampled() {
    let env = env_with_plot();
    let canvas = MockCanvas::new(800, 450);
    install_mock_canvas(&env, &canvas);

    eval_all(
        "(define f (lambda (x) (/ 1 x))) (plot-function f (- 0 2) 5)",
        &env,
    );

    let borrowed = canvas.borrow();
    let lines = curve_lines(&borrowed.commands);
    let x0_px = 40.0 + (720.0 / 7.0) * (0.0 - (-2.0));
    for (x1, y1, x2, y2) in &lines {
        assert!(
            !(*x1 <= x0_px && x0_px <= *x2),
            "parasitic segment crossing x=0 on asymmetric interval: ({x1}, {y1}) -> ({x2}, {y2})"
        );
    }
}

#[test]
fn isolated_sample_point_uses_draw_point() {
    let env = env_with_plot();
    let canvas = MockCanvas::new(800, 450);
    install_mock_canvas(&env, &canvas);

    eval_all(
        "(define spike (lambda (x) (if (= x 0.0) 0.0 (/ 0.0 0.0)))) (plot-function spike (- 0 1) 1)",
        &env,
    );

    let borrowed = canvas.borrow();
    let points: Vec<(f64, f64, Rgb)> = borrowed
        .commands
        .iter()
        .filter_map(|cmd| match cmd {
            RecordedCommand::Point { x, y, color } => Some((*x, *y, *color)),
            _ => None,
        })
        .collect();
    assert_eq!(points.len(), 1);
    assert!((points[0].0 - 400.0).abs() < 1e-6);
    assert!((points[0].1 - 225.0).abs() < 1e-6);
    assert_eq!(points[0].2, Rgb(0.9, 0.3, 0.2));
}
