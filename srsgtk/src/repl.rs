//! Scheme REPL widget: an output log (read-only, scrollable) plus an
//! input entry supporting multi-line forms and command history.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::gdk::Key;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Entry, EventControllerKey, Orientation, PolicyType, ScrolledWindow, TextBuffer,
    TextView, WrapMode,
};

use libsrs::interpretor::repl::{EvalOutcome, eval_source};
use libsrs::types::core::{Env, SrsValue};

/// Mutable state shared by the REPL widget's callbacks.
struct ReplState {
    env: Rc<Env>,
    /// Source accumulated so far for a form spanning multiple lines
    /// (empty when no form is in progress).
    pending: RefCell<String>,
    /// Previously submitted (complete) top-level forms, oldest first.
    history: RefCell<Vec<String>>,
    /// Index into `history` currently shown in the entry while
    /// navigating with Up/Down, and the draft that was being typed
    /// before navigation started.
    history_cursor: Cell<Option<usize>>,
    draft: RefCell<String>,
}

/// Builds the REPL widget: a scrollable output log on top of a single-line
/// input entry. Returns the top-level widget to embed in the window.
/// Evaluates forms in `env`, which the caller is responsible for
/// populating (e.g. with the base global bindings and any host-specific
/// primitives such as the canvas drawing procedures).
pub fn build_repl_widget(env: Rc<Env>) -> GtkBox {
    let output_buffer = TextBuffer::builder().build();
    let error_tag = output_buffer
        .create_tag(Some("error"), &[("foreground", &"#e06c75")])
        .expect("creating the 'error' text tag should not fail");
    let prompt_tag = output_buffer
        .create_tag(Some("prompt"), &[("foreground", &"#61afef")])
        .expect("creating the 'prompt' text tag should not fail");

    let output_view = TextView::builder()
        .buffer(&output_buffer)
        .editable(false)
        .cursor_visible(false)
        .monospace(true)
        .wrap_mode(WrapMode::WordChar)
        .build();

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .vexpand(true)
        .hexpand(true)
        .child(&output_view)
        .build();

    let input = Entry::builder()
        .placeholder_text("srs>")
        .hexpand(true)
        .build();

    let state = Rc::new(ReplState {
        env,
        pending: RefCell::new(String::new()),
        history: RefCell::new(Vec::new()),
        history_cursor: Cell::new(None),
        draft: RefCell::new(String::new()),
    });

    append_line(&output_buffer, &output_view, "srs REPL", None);

    {
        let state = state.clone();
        let output_buffer = output_buffer.clone();
        let output_view = output_view.clone();
        let error_tag = error_tag.clone();
        let prompt_tag = prompt_tag.clone();
        input.connect_activate(move |entry| {
            let line = entry.text().to_string();
            let prompt = if state.pending.borrow().is_empty() {
                "srs>"
            } else {
                "..."
            };
            append_line(
                &output_buffer,
                &output_view,
                &format!("{prompt} {line}"),
                Some(&prompt_tag),
            );

            let source = {
                let pending = state.pending.borrow();
                if pending.is_empty() {
                    line.clone()
                } else {
                    format!("{}\n{}", pending, line)
                }
            };

            match eval_source(&source, &state.env) {
                Ok(EvalOutcome::Incomplete) => {
                    *state.pending.borrow_mut() = source;
                    entry.set_placeholder_text(Some("..."));
                }
                Ok(EvalOutcome::Done(values)) => {
                    for value in values {
                        if !matches!(value, SrsValue::Unspecified) {
                            append_line(&output_buffer, &output_view, &value.to_string(), None);
                        }
                    }
                    state.history.borrow_mut().push(source);
                    state.pending.borrow_mut().clear();
                    entry.set_placeholder_text(Some("srs>"));
                }
                Err(e) => {
                    append_line(
                        &output_buffer,
                        &output_view,
                        &format!("erreur: {e}"),
                        Some(&error_tag),
                    );
                    state.pending.borrow_mut().clear();
                    entry.set_placeholder_text(Some("srs>"));
                }
            }

            entry.set_text("");
            state.history_cursor.set(None);
        });
    }

    let key_controller = EventControllerKey::new();
    {
        let state = state.clone();
        let input = input.clone();
        key_controller.connect_key_pressed(move |_controller, key, _code, _modifiers| match key {
            Key::Up => {
                navigate_history(&state, &input, -1);
                glib::Propagation::Stop
            }
            Key::Down => {
                navigate_history(&state, &input, 1);
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        });
    }
    input.add_controller(key_controller);

    let container = GtkBox::new(Orientation::Vertical, 4);
    container.set_vexpand(true);
    container.set_hexpand(true);
    container.append(&scrolled);
    container.append(&input);

    container
}

/// Moves the history cursor by `delta` (-1 for older, +1 for newer) and
/// updates the entry's text accordingly. Only navigates when no
/// multi-line form is pending, to avoid clobbering the entry's role as a
/// continuation prompt.
fn navigate_history(state: &Rc<ReplState>, entry: &Entry, delta: i32) {
    if !state.pending.borrow().is_empty() {
        return;
    }

    let history = state.history.borrow();
    if history.is_empty() {
        return;
    }

    let current = state.history_cursor.get();
    let next = match (current, delta) {
        (None, -1) => Some(history.len() - 1),
        (None, 1) => None,
        (Some(i), -1) => Some(i.saturating_sub(1)),
        (Some(i), 1) if i + 1 < history.len() => Some(i + 1),
        (Some(_), 1) => None,
        _ => current,
    };

    if current.is_none() && delta == -1 {
        *state.draft.borrow_mut() = entry.text().to_string();
    }

    match next {
        Some(i) => entry.set_text(&history[i]),
        None => entry.set_text(&state.draft.borrow()),
    }
    entry.set_position(-1);
    state.history_cursor.set(next);
}

/// Appends `text` followed by a newline to the output log, applying `tag`
/// (if any) to the whole line, then scrolls the view to keep it visible.
fn append_line(buffer: &TextBuffer, view: &TextView, text: &str, tag: Option<&gtk4::TextTag>) {
    let mut start = buffer.end_iter();
    let start_offset = start.offset();
    buffer.insert(&mut start, text);
    buffer.insert(&mut buffer.end_iter(), "\n");

    if let Some(tag) = tag {
        let start_iter = buffer.iter_at_offset(start_offset);
        let end_iter = buffer.end_iter();
        buffer.apply_tag(tag, &start_iter, &end_iter);
    }

    let end_mark = buffer.create_mark(None, &buffer.end_iter(), false);
    view.scroll_mark_onscreen(&end_mark);
    buffer.delete_mark(&end_mark);
}
