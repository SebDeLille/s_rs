use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

fn eval_src(scm: &str) -> SrsValue {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let env = global_env();
    let mut result = SrsValue::Unspecified;
    for value in &values {
        result = eval(value, &env).unwrap();
    }
    result
}

fn eval_error(scm: &str) -> String {
    let values = read_all(get_lexemes(scm).unwrap()).unwrap();
    let env = global_env();
    eval(&values[0], &env).unwrap_err().to_string()
}

fn as_str(value: &SrsValue) -> String {
    match value {
        SrsValue::String(cell) => cell.borrow().clone(),
        other => panic!("expected string, got {:?}", other),
    }
}

#[test]
fn string_p_recognizes_strings() {
    assert!(matches!(
        eval_src("(string? \"hello\")"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(string? 123)"),
        SrsValue::Boolean(false)
    ));
}

#[test]
fn string_length_returns_character_count() {
    assert!(matches!(
        eval_src("(string-length \"hello\")"),
        SrsValue::Integer(5)
    ));
}

#[test]
fn string_ref_returns_character_at_index() {
    assert!(matches!(
        eval_src("(string-ref \"hello\" 1)"),
        SrsValue::Character('e')
    ));
}

#[test]
fn string_ref_out_of_range_is_an_error() {
    let values = read_all(get_lexemes("(string-ref \"hi\" 5)").unwrap()).unwrap();
    let env = global_env();
    assert!(eval(&values[0], &env).is_err());
}

#[test]
fn string_eq_compares_character_sequences() {
    assert!(matches!(
        eval_src("(string=? \"abc\" \"abc\")"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(string=? \"abc\" \"def\")"),
        SrsValue::Boolean(false)
    ));
    assert!(matches!(
        eval_src("(string=? \"a\" \"a\" \"a\")"),
        SrsValue::Boolean(true)
    ));
}

#[test]
fn substring_extracts_slice() {
    assert_eq!(as_str(&eval_src("(substring \"hello\" 1 4)")), "ell");
}

#[test]
fn substring_start_equals_end_returns_empty() {
    assert_eq!(as_str(&eval_src("(substring \"hello\" 2 2)")), "");
}

#[test]
fn string_append_concatenates_arguments() {
    assert_eq!(
        as_str(&eval_src("(string-append \"foo\" \"bar\")")),
        "foobar"
    );
}

#[test]
fn string_append_with_no_arguments_returns_empty_string() {
    assert_eq!(as_str(&eval_src("(string-append)")), "");
}

#[test]
fn list_to_string_converts_characters() {
    assert_eq!(as_str(&eval_src("(list->string '(#\\f #\\o #\\o))")), "foo");
    assert!(eval_error("(list->string 1)").contains("wrong type"));
    assert!(eval_error("(list->string '(#\\a 1))").contains("characters"));
    assert!(eval_error("(list->string)").contains("not enough"));
    assert!(eval_error("(list->string '() '())").contains("too many"));
}

#[test]
fn string_to_list_converts_string_and_supports_ranges() {
    assert_eq!(
        eval_src("(string->list \"abc\")").to_string(),
        "(#\\a #\\b #\\c)"
    );
    assert_eq!(
        eval_src("(string->list \"abcd\" 1 3)").to_string(),
        "(#\\b #\\c)"
    );
    assert!(eval_error("(string->list 42)").contains("wrong type"));
    assert!(eval_error("(string->list)").contains("not enough"));
    assert!(eval_error("(string->list \"a\" 0 1 2)").contains("too many"));
    assert!(eval_error("(string->list \"a\" -1)").contains("negative"));
}

#[test]
fn string_index_returns_character_offset_or_false() {
    assert!(matches!(
        eval_src("(string-index \"héllo\" #\\l)"),
        SrsValue::Integer(2)
    ));
    assert!(matches!(
        eval_src("(string-index \"abc\" #\\z)"),
        SrsValue::Boolean(false)
    ));
    assert!(eval_error("(string-index 3 #\\a)").contains("wrong type"));
    assert!(eval_error("(string-index \"a\" \"a\")").contains("wrong type"));
    assert!(eval_error("(string-index \"a\")").contains("not enough"));
    assert!(eval_error("(string-index \"a\" #\\a #\\b)").contains("too many"));
}

#[test]
fn string_copy_returns_a_fresh_string_and_supports_ranges() {
    assert_eq!(as_str(&eval_src("(string-copy \"héllo\" 1 3)")), "él");
    assert!(eval_error("(string-copy 4)").contains("wrong type"));
    assert!(eval_error("(string-copy)").contains("not enough"));
    assert!(eval_error("(string-copy \"abc\" 0 1 2)").contains("too many"));
    assert!(eval_error("(string-copy \"abc\" 0 4)").contains("out of range"));
}

#[test]
fn number_to_string_formats_numbers_and_radices() {
    assert_eq!(as_str(&eval_src("(number->string 42)")), "42");
    assert_eq!(as_str(&eval_src("(number->string 255 16)")), "ff");
    assert_eq!(as_str(&eval_src("(number->string 1.5)")), "1.5");
    assert!(eval_error("(number->string \"42\")").contains("wrong type"));
    assert!(eval_error("(number->string)").contains("not enough"));
    assert!(eval_error("(number->string 1 3)").contains("radix"));
    assert!(eval_error("(number->string 1 2.0)").contains("expected integer radix"));
    assert!(eval_error("(number->string 1 10 10)").contains("too many"));
}

#[test]
fn string_to_number_parses_integers_floats_and_radices() {
    assert!(matches!(
        eval_src("(string->number \"100\")"),
        SrsValue::Integer(100)
    ));
    assert!(matches!(
        eval_src("(string->number \"100\" 16)"),
        SrsValue::Integer(256)
    ));
    assert!(matches!(
        eval_src("(string->number \"1e2\")"),
        SrsValue::Float(100.0)
    ));
    assert!(matches!(
        eval_src("(string->number \"abc\")"),
        SrsValue::Boolean(false)
    ));
    assert!(eval_error("(string->number 42)").contains("wrong type"));
    assert!(eval_error("(string->number)").contains("not enough"));
    assert!(eval_error("(string->number \"1\" 3)").contains("radix"));
    assert!(eval_error("(string->number \"1\" 10 10)").contains("too many"));
}

#[test]
fn character_predicate_and_conversions() {
    assert!(matches!(eval_src("(char? #\\a)"), SrsValue::Boolean(true)));
    assert!(matches!(
        eval_src("(char? \"a\")"),
        SrsValue::Boolean(false)
    ));
    assert!(matches!(
        eval_src("(char->integer #\\A)"),
        SrsValue::Integer(65)
    ));
    assert!(matches!(
        eval_src("(integer->char 65)"),
        SrsValue::Character('A')
    ));
    assert!(matches!(
        eval_src("(char=? (integer->char (char->integer #\\x)) #\\x)"),
        SrsValue::Boolean(true)
    ));
    assert!(eval_error("(char=? #\\a 1)").contains("wrong type"));
    assert!(eval_error("(char=?)").contains("not enough"));
    assert!(matches!(
        eval_src("(char-whitespace? #\\space)"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(char-whitespace? #\\a)"),
        SrsValue::Boolean(false)
    ));
    assert!(matches!(
        eval_src("(char-numeric? #\\7)"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(char-alphabetic? #\\a)"),
        SrsValue::Boolean(true)
    ));
    assert!(matches!(
        eval_src("(char-upcase #\\a)"),
        SrsValue::Character('A')
    ));
    assert!(matches!(
        eval_src("(char-downcase #\\A)"),
        SrsValue::Character('a')
    ));
    assert!(eval_error("(char->integer 1)").contains("wrong type"));
    assert!(eval_error("(char->integer)").contains("not enough"));
    assert!(eval_error("(char->integer #\\a #\\b)").contains("too many"));
    assert!(eval_error("(integer->char #\\a)").contains("wrong type"));
    assert!(eval_error("(integer->char -1)").contains("Unicode scalar"));
    assert!(eval_error("(integer->char)").contains("not enough"));
    assert!(eval_error("(integer->char 65 66)").contains("too many"));
    assert!(eval_error("(char-whitespace? 1)").contains("wrong type"));
    assert!(eval_error("(char-whitespace?)").contains("not enough"));
    assert!(eval_error("(char-whitespace? #\\a #\\b)").contains("too many"));
    assert!(eval_error("(char-numeric? \"7\")").contains("wrong type"));
    assert!(eval_error("(char-numeric?)").contains("not enough"));
    assert!(eval_error("(char-numeric? #\\7 #\\8)").contains("too many"));
    assert!(eval_error("(char-alphabetic? 1)").contains("wrong type"));
    assert!(eval_error("(char-alphabetic?)").contains("not enough"));
    assert!(eval_error("(char-alphabetic? #\\a #\\b)").contains("too many"));
    assert!(eval_error("(char-upcase 1)").contains("wrong type"));
    assert!(eval_error("(char-upcase)").contains("not enough"));
    assert!(eval_error("(char-upcase #\\a #\\b)").contains("too many"));
    assert!(eval_error("(char-downcase 1)").contains("wrong type"));
    assert!(eval_error("(char-downcase)").contains("not enough"));
    assert!(eval_error("(char-downcase #\\a #\\b)").contains("too many"));
    assert!(eval_error("(char? )").contains("not enough"));
    assert!(eval_error("(char? #\\a #\\b)").contains("too many"));
}
