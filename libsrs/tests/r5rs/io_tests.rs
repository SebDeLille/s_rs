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

fn string_value(value: SrsValue) -> String {
    match value {
        SrsValue::String(s) => s.borrow().clone(),
        other => panic!("expected string, got {}", other),
    }
}

fn char_value(value: SrsValue) -> char {
    match value {
        SrsValue::Character(c) => c,
        other => panic!("expected character, got {}", other),
    }
}

fn boolean_value(value: SrsValue) -> bool {
    match value {
        SrsValue::Boolean(b) => b,
        other => panic!("expected boolean, got {}", other),
    }
}

fn eval_src(scm: &str) -> SrsValue {
    eval_all(scm).unwrap()
}

#[test]
fn eof_object_returns_eof_value() {
    assert!(matches!(eval_src("(eof-object)"), SrsValue::Eof));
}

#[test]
fn eof_object_p_returns_true_for_eof_object() {
    assert!(matches!(
        eval_src("(eof-object? (eof-object))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn eof_object_p_returns_false_for_non_eof_values() {
    assert!(matches!(
        eval_src("(eof-object? 42)"),
        SrsValue::Boolean(false)
    ));
}

#[test]
fn current_input_port_returns_port_value() {
    assert!(matches!(
        eval_src("(current-input-port)"),
        SrsValue::Port(_)
    ));
}

#[test]
fn current_output_port_returns_port_value() {
    assert!(matches!(
        eval_src("(current-output-port)"),
        SrsValue::Port(_)
    ));
}

#[test]
fn current_input_port_is_singleton() {
    assert!(matches!(
        eval_src("(eq? (current-input-port) (current-input-port))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn current_output_port_is_singleton() {
    assert!(matches!(
        eval_src("(eq? (current-output-port) (current-output-port))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn current_input_port_binds_internal_variable() {
    assert!(matches!(
        eval_src("(eq? (current-input-port) *current-input-port*)"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn current_output_port_binds_internal_variable() {
    assert!(matches!(
        eval_src("(eq? (current-output-port) *current-output-port*)"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn read_char_rejects_non_port_argument() {
    assert!(eval_all("(read-char 42)").is_err());
}

#[test]
fn read_char_rejects_too_many_arguments() {
    assert!(eval_all("(read-char (current-input-port) 1)").is_err());
}

#[test]
fn peek_char_rejects_non_port_argument() {
    assert!(eval_all("(peek-char 42)").is_err());
}

#[test]
fn peek_char_rejects_too_many_arguments() {
    assert!(eval_all("(peek-char (current-input-port) 1)").is_err());
}

#[test]
fn read_char_rejects_output_port() {
    assert!(eval_all("(read-char (current-output-port))").is_err());
    assert!(eval_all("(peek-char (current-output-port))").is_err());
}

#[test]
fn read_char_on_string_port_returns_character_or_eof() {
    assert_eq!(
        char_value(eval_src("(read-char (open-input-string \"x\"))")),
        'x'
    );
    assert!(matches!(
        eval_src("(read-char (open-input-string \"\"))"),
        SrsValue::Eof
    ));
}

#[test]
fn peek_char_on_string_port_returns_character_or_eof() {
    assert_eq!(
        char_value(eval_src("(peek-char (open-input-string \"x\"))")),
        'x'
    );
    assert!(matches!(
        eval_src("(peek-char (open-input-string \"\"))"),
        SrsValue::Eof
    ));
}

#[test]
fn peek_then_read_return_same_character_on_string_port() {
    assert_eq!(
        eval_src("(let ((p (open-input-string \"x\"))) (peek-char p))").to_string(),
        "#\\x"
    );
    assert_eq!(
        eval_src("(let ((p (open-input-string \"ab\"))) (peek-char p) (read-char p))").to_string(),
        "#\\a"
    );
}

#[test]
fn read_line_rejects_non_port_argument() {
    assert!(eval_all("(read-line 42)").is_err());
}

#[test]
fn read_line_rejects_too_many_arguments() {
    assert!(eval_all("(read-line (current-input-port) 1)").is_err());
}

#[test]
fn read_line_rejects_output_port() {
    assert!(eval_all("(read-line (current-output-port))").is_err());
}

#[test]
fn char_ready_rejects_non_port_argument() {
    assert!(eval_all("(char-ready? 42)").is_err());
}

#[test]
fn char_ready_rejects_too_many_arguments() {
    assert!(eval_all("(char-ready? (current-input-port) 1)").is_err());
}

#[test]
fn char_ready_rejects_output_port() {
    assert!(eval_all("(char-ready? (current-output-port))").is_err());
}

#[test]
fn char_ready_without_args_returns_boolean() {
    assert!(matches!(eval_src("(char-ready?)"), SrsValue::Boolean(true)));
}

#[test]
fn char_ready_with_input_port_returns_boolean() {
    assert!(matches!(
        eval_src("(char-ready? (current-input-port))"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn read_line_on_string_port_returns_string_or_eof() {
    assert_eq!(
        string_value(eval_src("(read-line (open-input-string \"abc\ndef\"))")),
        "abc"
    );
    assert!(matches!(
        eval_src("(read-line (open-input-string \"\"))"),
        SrsValue::Eof
    ));
}

#[test]
fn open_input_string_creates_input_port() {
    assert!(eval_all("(open-input-string \"hello\")").is_ok());
}

#[test]
fn open_input_string_requires_string() {
    assert!(eval_all("(open-input-string 42)").is_err());
    assert!(eval_all("(open-input-string)").is_err());
    assert!(eval_all("(open-input-string \"a\" \"b\")").is_err());
}

#[test]
fn open_output_string_creates_output_port() {
    assert!(eval_all("(open-output-string)").is_ok());
    assert!(eval_all("(open-output-string 1)").is_err());
}

#[test]
fn get_output_string_returns_accumulated_content() {
    let result =
        eval_src("(let ((p (open-output-string))) (write-char #\\x p) (get-output-string p))");
    assert_eq!(string_value(result), "x");
}

#[test]
fn get_output_string_requires_output_string_port() {
    assert!(eval_all("(get-output-string)").is_err());
    assert!(eval_all("(get-output-string (current-output-port))").is_err());
    assert!(eval_all("(get-output-string (open-input-string \"x\"))").is_err());
}

#[test]
fn write_char_requires_character() {
    assert!(eval_all("(write-char \"x\")").is_err());
}

#[test]
fn write_char_rejects_input_port() {
    assert!(eval_all("(write-char #\\x (open-input-string \"x\"))").is_err());
}

#[test]
fn char_ready_on_string_port_reflects_remaining_data() {
    assert!(boolean_value(eval_src(
        "(char-ready? (open-input-string \"x\"))"
    )));
    assert!(!boolean_value(eval_src(
        "(let ((p (open-input-string \"\"))) (char-ready? p))"
    )));
}

#[test]
fn display_writes_to_output_string_port() {
    let result =
        eval_src("(let ((p (open-output-string))) (display \"hi\" p) (get-output-string p))");
    assert_eq!(string_value(result), "hi");
}

#[test]
fn display_rejects_input_port() {
    assert!(eval_all("(display \"x\" (open-input-string \"x\"))").is_err());
}

#[test]
fn display_accepts_current_output_port() {
    assert!(eval_all("(display \"x\" (current-output-port))").is_ok());
}

#[test]
fn newline_writes_to_output_string_port() {
    let result = eval_src("(let ((p (open-output-string))) (newline p) (get-output-string p))");
    assert_eq!(string_value(result), "\n");
}

#[test]
fn newline_rejects_input_port() {
    assert!(eval_all("(newline (open-input-string \"x\"))").is_err());
}

#[test]
fn write_char_uses_current_output_port_by_default() {
    assert!(eval_all("(write-char #\\a)").is_ok());
}

#[test]
fn write_char_writes_to_output_string_port() {
    let result =
        eval_src("(let ((p (open-output-string))) (write-char #\\a p) (get-output-string p))");
    assert_eq!(string_value(result), "a");
}

#[test]
fn write_string_writes_to_output_string_port() {
    let result = eval_src(
        "(let ((p (open-output-string))) (write-string \"hello\" p) (get-output-string p))",
    );
    assert_eq!(string_value(result), "hello");
}

#[test]
fn write_string_supports_start_end_bounds() {
    let result = eval_src(
        "(let ((p (open-output-string))) (write-string \"hello\" p 1 4) (get-output-string p))",
    );
    assert_eq!(string_value(result), "ell");
}

#[test]
fn write_string_rejects_input_port() {
    assert!(eval_all("(write-string \"x\" (open-input-string \"x\"))").is_err());
}

#[test]
fn write_string_requires_string() {
    assert!(eval_all("(write-string 42)").is_err());
    assert!(eval_all("(write-string)").is_err());
}

#[test]
fn write_string_with_too_many_args_fails() {
    assert!(eval_all("(let ((p (open-output-string))) (write-string \"a\" p 0 1 2))").is_err());
}
