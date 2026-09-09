//! Native Scheme primitives exposing a simple drawing canvas backed by a
//! `gtk4::DrawingArea` and rendered with Cairo.
//!
//! State is kept as a list of recorded draw commands (`CanvasState`),
//! captured by the native closures via `Rc<RefCell<_>>`. The
//! `DrawingArea`'s draw function replays this list every time the widget
//! is redrawn (e.g. on resize), and each primitive calls `queue_draw()`
//! after mutating the state so the new command is reflected immediately.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::DrawingArea;
use gtk4::prelude::*;

use libsrs::types::core::{Env, Native, SrsValue};

/// RGB color in the `[0.0, 1.0]` range, as expected by Cairo.
#[derive(Clone, Copy)]
struct Color {
    r: f64,
    g: f64,
    b: f64,
}

const DEFAULT_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
};

const BACKGROUND_COLOR: Color = Color {
    r: 0.15,
    g: 0.15,
    b: 0.18,
};

/// A single recorded drawing operation, with the color that was current
/// when it was issued.
enum DrawCommand {
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
    },
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        color: Color,
    },
    Circle {
        x: f64,
        y: f64,
        r: f64,
        color: Color,
    },
}

/// Shared canvas state: the commands recorded so far and the color used
/// by the next drawing primitive.
struct CanvasState {
    commands: Vec<DrawCommand>,
    current_color: Color,
}

impl CanvasState {
    fn new() -> Self {
        CanvasState {
            commands: Vec::new(),
            current_color: DEFAULT_COLOR,
        }
    }
}

/// Installs the graphics primitives (`clear-canvas`, `set-color`,
/// `draw-line`, `draw-rect`, `draw-circle`) into `env`, wired to redraw
/// `drawing_area` via Cairo.
pub fn install(env: &Rc<Env>, drawing_area: &DrawingArea) {
    let state = Rc::new(RefCell::new(CanvasState::new()));

    drawing_area.set_draw_func({
        let state = state.clone();
        move |_area, cr, width, height| {
            cr.set_source_rgb(BACKGROUND_COLOR.r, BACKGROUND_COLOR.g, BACKGROUND_COLOR.b);
            cr.rectangle(0.0, 0.0, width as f64, height as f64);
            let _ = cr.fill();

            for command in &state.borrow().commands {
                match command {
                    DrawCommand::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        color,
                    } => {
                        cr.set_source_rgb(color.r, color.g, color.b);
                        cr.move_to(*x1, *y1);
                        cr.line_to(*x2, *y2);
                        let _ = cr.stroke();
                    }
                    DrawCommand::Rect { x, y, w, h, color } => {
                        cr.set_source_rgb(color.r, color.g, color.b);
                        cr.rectangle(*x, *y, *w, *h);
                        let _ = cr.stroke();
                    }
                    DrawCommand::Circle { x, y, r, color } => {
                        cr.set_source_rgb(color.r, color.g, color.b);
                        cr.arc(*x, *y, *r, 0.0, std::f64::consts::TAU);
                        let _ = cr.stroke();
                    }
                }
            }
        }
    });

    define_native(env, "clear-canvas", {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |args| {
            match args {
                [] => {}
                _ => return Err("too many arguments to clear-canvas".to_string()),
            }
            state.borrow_mut().commands.clear();
            drawing_area.queue_draw();
            Ok(SrsValue::Unspecified)
        }
    });

    define_native(env, "set-color", {
        let state = state.clone();
        move |args| {
            let [r, g, b] = args else {
                return Err("set-color: expected 3 arguments (r g b)".to_string());
            };
            let color = Color {
                r: numeric_to_f64(r, "set-color")?,
                g: numeric_to_f64(g, "set-color")?,
                b: numeric_to_f64(b, "set-color")?,
            };
            state.borrow_mut().current_color = color;
            Ok(SrsValue::Unspecified)
        }
    });

    define_native(env, "draw-line", {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |args| {
            let [x1, y1, x2, y2] = args else {
                return Err("draw-line: expected 4 arguments (x1 y1 x2 y2)".to_string());
            };
            let x1 = numeric_to_f64(x1, "draw-line")?;
            let y1 = numeric_to_f64(y1, "draw-line")?;
            let x2 = numeric_to_f64(x2, "draw-line")?;
            let y2 = numeric_to_f64(y2, "draw-line")?;
            let color = state.borrow().current_color;
            state.borrow_mut().commands.push(DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                color,
            });
            drawing_area.queue_draw();
            Ok(SrsValue::Unspecified)
        }
    });

    define_native(env, "draw-rect", {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |args| {
            let [x, y, w, h] = args else {
                return Err("draw-rect: expected 4 arguments (x y w h)".to_string());
            };
            let x = numeric_to_f64(x, "draw-rect")?;
            let y = numeric_to_f64(y, "draw-rect")?;
            let w = numeric_to_f64(w, "draw-rect")?;
            let h = numeric_to_f64(h, "draw-rect")?;
            let color = state.borrow().current_color;
            state
                .borrow_mut()
                .commands
                .push(DrawCommand::Rect { x, y, w, h, color });
            drawing_area.queue_draw();
            Ok(SrsValue::Unspecified)
        }
    });

    define_native(env, "draw-circle", {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |args| {
            let [x, y, r] = args else {
                return Err("draw-circle: expected 3 arguments (x y r)".to_string());
            };
            let x = numeric_to_f64(x, "draw-circle")?;
            let y = numeric_to_f64(y, "draw-circle")?;
            let r = numeric_to_f64(r, "draw-circle")?;
            let color = state.borrow().current_color;
            state
                .borrow_mut()
                .commands
                .push(DrawCommand::Circle { x, y, r, color });
            drawing_area.queue_draw();
            Ok(SrsValue::Unspecified)
        }
    });
}

/// Defines a native procedure named `name` in `env`, wrapping `func` as a
/// [`Native`] closure.
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

/// Converts a numeric [`SrsValue`] to `f64`, rejecting non-numeric
/// arguments with an error naming `proc_name`.
fn numeric_to_f64(value: &SrsValue, proc_name: &str) -> Result<f64, String> {
    match value {
        SrsValue::Integer(n) => Ok(*n as f64),
        SrsValue::Float(f) => Ok(*f),
        SrsValue::Rational(n, d) => Ok(*n as f64 / *d as f64),
        _ => Err(format!("wrong type: expected number to {}", proc_name)),
    }
}
