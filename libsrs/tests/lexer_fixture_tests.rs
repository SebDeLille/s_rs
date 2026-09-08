use libsrs::interpretor::lexical_analyzer::{Lexeme, get_lexemes};

const MATRIX_SCM: &str = include_str!("fixtures/scheme/matrix.scm");

#[test]
fn matrix_fixture_lexes_without_error() {
    let result = get_lexemes(MATRIX_SCM);
    assert!(result.is_ok(), "matrix.scm failed to lex: {:?}", result);
}

#[test]
fn matrix_fixture_contains_expected_tokens() {
    let lexemes = get_lexemes(MATRIX_SCM).unwrap();

    assert_eq!(lexemes.first(), Some(&Lexeme::LParen));
    assert!(lexemes.contains(&Lexeme::Id("define".to_string())));
    assert!(lexemes.contains(&Lexeme::Id("make-matrix".to_string())));
    assert!(lexemes.contains(&Lexeme::Id("lambda".to_string())));
    assert!(lexemes.contains(&Lexeme::Id("vector-set!".to_string())));
    assert!(lexemes.contains(&Lexeme::Id("cond".to_string())));
    assert!(lexemes.contains(&Lexeme::Id("else".to_string())));
}
