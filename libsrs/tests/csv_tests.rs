use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

fn eval_all(scm: &str) -> Result<SrsValue, String> {
    eval_all_in_env(scm, &global_env())
}

fn eval_all_in_env(scm: &str, env: &Rc<libsrs::types::core::Env>) -> Result<SrsValue, String> {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, env).map_err(|e| e.to_string())?;
    }
    Ok(result)
}

fn eval_src(scm: &str) -> SrsValue {
    eval_all(scm).unwrap()
}

fn integer_value(value: SrsValue) -> i64 {
    match value {
        SrsValue::Integer(n) => n,
        other => panic!("expected integer, got {}", other),
    }
}

static TMP_COUNTER: AtomicU64 = AtomicU64::new(1);

fn make_temp_file(contents: &str) -> String {
    let dir = std::env::temp_dir();
    let n = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = dir.join(format!("s_rs_csv_test_{}.csv", n));
    std::fs::write(&path, contents).unwrap();
    path.to_str().unwrap().to_string()
}

fn escape_path(path: &str) -> String {
    path.replace('\\', "\\\\")
}

fn csv_lib_path() -> String {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib_path = manifest.parent().unwrap().join("libs").join("csv.scm");
    escape_path(lib_path.to_str().unwrap())
}

#[test]
fn csv_read_file_parses_comma_fixture() {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = escape_path(manifest.join("tests/fixtures/csv/users.csv").to_str().unwrap());
    let lib = csv_lib_path();
    let src = format!(
        "(load \"{}\") (define csv (csv-read-file \"{}\" \",\")) (vector-length (csv-header csv))",
        lib, fixture
    );
    assert_eq!(integer_value(eval_src(&src)), 3);
}

#[test]
fn csv_read_file_parses_semicolon_fixture() {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = escape_path(manifest.join("tests/fixtures/csv/users_semicolon.csv").to_str().unwrap());
    let lib = csv_lib_path();
    let src = format!(
        "(load \"{}\") (define csv (csv-read-file \"{}\" \";\")) (vector-length (csv-header csv))",
        lib, fixture
    );
    assert_eq!(integer_value(eval_src(&src)), 3);
}

#[test]
fn csv_for_each_row_iterates_all_rows() {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = escape_path(manifest.join("tests/fixtures/csv/users.csv").to_str().unwrap());
    let lib = csv_lib_path();
    let src = format!(
        "(load \"{}\")
         (define out (open-output-string))
         (csv-for-each-row \"{}\" \",\" (lambda (h r) (display \"x\" out)))
         (string-length (get-output-string out))",
        lib, fixture
    );
    assert_eq!(integer_value(eval_src(&src)), 3);
}

#[test]
fn csv_for_each_row_header_is_stable_and_parsed() {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = escape_path(manifest.join("tests/fixtures/csv/users.csv").to_str().unwrap());
    let lib = csv_lib_path();
    let src = format!(
        "(load \"{}\")
         (define out (open-output-string))
         (csv-for-each-row \"{}\" \",\" (lambda (h r) (display (car h) out)))
         (get-output-string out)",
        lib, fixture
    );
    let result = match eval_src(&src) {
        SrsValue::String(s) => s.borrow().clone(),
        other => panic!("expected string, got {}", other),
    };
    assert_eq!(result, "prenomprenomprenom");
}

#[test]
fn csv_for_each_row_handles_empty_file() {
    let path = make_temp_file("");
    let lib = csv_lib_path();
    let src = format!(
        "(load \"{}\")
         (define out (open-output-string))
         (csv-for-each-row \"{}\" \",\" (lambda (h r) (display \"x\" out)))
         (string-length (get-output-string out))",
        lib, escape_path(&path)
    );
    assert_eq!(integer_value(eval_src(&src)), 0);
}

#[test]
fn csv_call_with_file_propagates_callback_error() {
    // The underlying dynamic-wind already guarantees that the port is closed
    // even when the callback raises (see control_tests). This test only checks
    // that csv-call-with-file does not swallow errors from the callback.
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = escape_path(manifest.join("tests/fixtures/csv/users.csv").to_str().unwrap());
    let lib = csv_lib_path();
    let src = format!(
        "(load \"{}\")\n         (csv-call-with-file \"{}\" \",\" (lambda (next-row header) (/ 1 0)))",
        lib, fixture
    );
    let err = eval_all(&src).expect_err("expected division by zero");
    assert!(err.contains("division by zero"), "unexpected error: {err}");
}

#[test]
fn csv_parse_line_char_eq_bench() {
    // Benchmark of the char=? optimization requested in issue #91.
    // The naive version allocates a fresh string for every character it
    // compares to the delimiter; the optimized version compares characters
    // directly. Both parsers are loaded once into a shared environment; the
    // CSV line is read from a temp file so the measurement does not pay the
    // cost of lexing a huge string literal in the test source.
    let lib = csv_lib_path();
    let env = global_env();
    eval_all_in_env(
        &format!(
            "(load \"{}\")
             (define csv-parse-line-naive
               (lambda (line delimiter)
                 (let ((len (string-length line)) (d (string-ref delimiter 0)))
                   (do ((i 0 (+ i 1))
                        (field '() (if (string=? (list->string (list (string-ref line i)))
                                                (list->string (list d)))
                                       '()
                                       (cons (string-ref line i) field)))
                        (fields '() (if (string=? (list->string (list (string-ref line i)))
                                                (list->string (list d)))
                                        (cons (list->string (reverse field)) fields)
                                        fields)))
                       ((= i len) (reverse (cons (list->string (reverse field)) fields)))
                     'ok))))",
            lib
        ),
        &env,
    )
    .unwrap();

    let fast_input = "a,b,c,".repeat(1_667); // ~10_000 characters
    let fast_path = make_temp_file(&fast_input);
    let fast_src = format!(
        "(define p (open-input-file \"{}\")) (define line (read-line p)) (close-input-port p) (length (csv-parse-line line \",\"))",
        escape_path(&fast_path)
    );
    let start = Instant::now();
    let fast_len = integer_value(eval_all_in_env(&fast_src, &env).unwrap());
    let fast_elapsed = start.elapsed();
    assert_eq!(fast_len, 3 * 1_667 + 1);

    let naive_input = "a,b,c,".repeat(167); // ~1_000 characters
    let naive_path = make_temp_file(&naive_input);
    let naive_src = format!(
        "(define p (open-input-file \"{}\")) (define line (read-line p)) (close-input-port p) (length (csv-parse-line-naive line \",\"))",
        escape_path(&naive_path)
    );
    let start = Instant::now();
    let naive_len = integer_value(eval_all_in_env(&naive_src, &env).unwrap());
    let naive_elapsed = start.elapsed();
    assert_eq!(naive_len, 3 * 167 + 1);

    let fast_per_char = fast_elapsed.as_nanos() as f64 / fast_input.len() as f64;
    let naive_per_char = naive_elapsed.as_nanos() as f64 / naive_input.len() as f64;
    eprintln!(
        "csv-parse-line perf: char=? {:?} ({} chars), naive {:?} ({} chars); per-char: {:.1} ns / {:.1} ns",
        fast_elapsed,
        fast_input.len(),
        naive_elapsed,
        naive_input.len(),
        fast_per_char,
        naive_per_char
    );

    // Smoke check: both versions must complete in a reasonable amount of time.
    assert!(fast_elapsed.as_secs() < 10, "char=? parser is unexpectedly slow");
    assert!(naive_elapsed.as_secs() < 10, "naive parser is unexpectedly slow");
}
