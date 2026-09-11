use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea, Orientation, Paned};

use libsrs::interpretor::evaluator::global_env_with_frontend;
use libsrs::interpretor::repl::{EvalOutcome, eval_source};
use libsrs::interpretor::startup::load_startup_scripts;

mod graphics;
mod repl;

const APP_ID: &str = "org.srs.srsgtk";
const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

/// Scheme prelude bundled with srsgtk (not a Rust primitive): pure
/// `.scm` code relying only on generic procedures already exposed by
/// `libsrs`. Evaluated once at startup, before user startup scripts, so
/// it's available to any script.
const FUNCTION_SAMPLES_SCM: &str = include_str!("../../libs/function-samples.scm");

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let paned = Paned::new(Orientation::Vertical);
    paned.set_wide_handle(true);

    let drawing_area = DrawingArea::new();
    drawing_area.set_vexpand(true);
    drawing_area.set_hexpand(true);

    let env = global_env_with_frontend("gtk");
    graphics::install(&env, &drawing_area);
    match eval_source(FUNCTION_SAMPLES_SCM, &env) {
        Ok(EvalOutcome::Done(_)) => {}
        Ok(EvalOutcome::Incomplete) => {
            eprintln!("erreur: prelude function-samples: unexpected end of input");
        }
        Err(message) => eprintln!("erreur: prelude function-samples: {}", message),
    }
    load_startup_scripts(&env);

    let repl_widget = repl::build_repl_widget(env);
    repl_widget.set_vexpand(true);
    repl_widget.set_hexpand(true);

    paned.set_start_child(Some(&drawing_area));
    paned.set_end_child(Some(&repl_widget));
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("srsgtk")
        .default_width(WINDOW_WIDTH)
        .default_height(WINDOW_HEIGHT)
        .child(&paned)
        .build();

    // 3/4 haut (zone graphique) - 1/4 bas (REPL), proportion appliquée sur
    // la position du séparateur en fonction de la hauteur de la fenêtre.
    window.connect_show(move |window| {
        let height = window.default_height();
        paned.set_position(height * 3 / 4);
    });

    window.present();
}
