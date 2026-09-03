use crate::interpretor::lexeme::{Lexeme, LexemeType};
use crate::types::error::{SrsError, SrsErrorKind, SrsResult};

/// State of the lexer's single-character DFA.
///
/// The "done" variants are transient markers produced when a lexeme
/// ends on the *current* character, which must then be reprocessed as
/// the start of the next lexeme. They are consumed by `is_star`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexerState {
    /// Start / accepting state. Whitespace keeps us here.
    Start,
    /// Reading an integer literal.
    Integer,
    /// Integer lexeme just emitted; current char still needs to be processed.
    IntegerDone,
    /// Reading the fractional part of a float (digits after '.').
    Float,
    /// Float lexeme just emitted; current char still needs to be processed.
    FloatDone,
    /// Reading an identifier or keyword.
    Id,
    /// Identifier lexeme just emitted; current char still needs to be processed.
    IdDone,
    /// Inside a double-quoted string.
    InString,
    /// After a `#`, expecting `t`, `f`, or `(`.
    Sharp,
    /// After a `<`, deciding between `<=` and `<`.
    Lt,
    /// `<` lexeme just emitted; current char still needs to be processed.
    LtDone,
    /// After a `>`, deciding between `>=` and `>`.
    Gt,
    /// `>` lexeme just emitted; current char still needs to be processed.
    GtDone,
    /// After a backslash inside a string, escaping the next character.
    Escape,
}

pub fn get_lexemes(s: &str) -> SrsResult<Vec<Lexeme>> {
    let mut list: Vec<Lexeme> = Vec::new();
    let mut status = LexerState::Start;
    let mut buffer = String::new();

    for c in s.chars() {
        filter(&c, &mut status, &mut buffer)?
            .into_iter()
            .for_each(|l| list.push(l));
    }

    if let Some(l) = filter_end(&status, &buffer)? {
        list.push(l);
    }

    Ok(list)
}

fn emit(l: LexemeType) -> Option<Lexeme> {
    Some(Lexeme::new(l, String::new()))
}

fn emit_value(l: LexemeType, buffer: &str) -> Option<Lexeme> {
    Some(Lexeme::new(l, buffer.to_string()))
}

/// States where the current character *terminates* the previous lexeme
/// and must be reprocessed as the start of the next lexeme.
fn is_star(status: LexerState) -> bool {
    matches!(
        status,
        LexerState::IntegerDone
            | LexerState::FloatDone
            | LexerState::IdDone
            | LexerState::LtDone
            | LexerState::GtDone
    )
}

fn filter(c: &char, status: &mut LexerState, buffer: &mut String) -> SrsResult<Vec<Lexeme>> {
    let mut result = Vec::new();

    if let Some(l) = filter_nobuffer(c, status, buffer)? {
        buffer.clear();
        result.push(l);
    }
    if is_star(*status) {
        *status = LexerState::Start;
        if let Some(l) = filter_nobuffer(c, status, buffer)? {
            buffer.clear();
            result.push(l);
        }
    }

    Ok(result)
}

fn filter_nobuffer(
    c: &char,
    status: &mut LexerState,
    buffer: &mut String,
) -> SrsResult<Option<Lexeme>> {
    match status {
        LexerState::Start => {
            if c.is_ascii_digit() {
                buffer.push(*c);
                *status = LexerState::Integer;
                Ok(None)
            } else if c.is_whitespace() {
                Ok(None)
            } else if c.is_alphabetic() || *c == '_' {
                buffer.push(*c);
                *status = LexerState::Id;
                Ok(None)
            } else if *c == '"' {
                *status = LexerState::InString;
                Ok(None)
            } else if *c == '#' {
                *status = LexerState::Sharp;
                Ok(None)
            } else if *c == '(' {
                Ok(emit(LexemeType::LPAR))
            } else if *c == ')' {
                Ok(emit(LexemeType::RPAR))
            } else if *c == '+' {
                Ok(emit(LexemeType::ADD))
            } else if *c == '-' {
                Ok(emit(LexemeType::SUB))
            } else if *c == '*' {
                Ok(emit(LexemeType::MUL))
            } else if *c == '/' {
                Ok(emit(LexemeType::DIV))
            } else if *c == '\'' {
                Ok(emit(LexemeType::QUOTE))
            } else if *c == '=' {
                Ok(emit(LexemeType::EQ))
            } else if *c == '<' {
                *status = LexerState::Lt;
                Ok(None)
            } else if *c == '>' {
                *status = LexerState::Gt;
                Ok(None)
            } else {
                Err(SrsError::with_message(
                    SrsErrorKind::UnsupportedChar,
                    format!("Unsupported char: {}", *c),
                ))
            }
        }
        LexerState::Integer => {
            if c.is_ascii_digit() {
                buffer.push(*c);
                Ok(None)
            } else if c.is_whitespace() {
                *status = LexerState::Start;
                Ok(emit_value(LexemeType::INTEGER, buffer))
            } else if *c == '.' {
                *status = LexerState::Float;
                buffer.push(*c);
                Ok(None)
            } else if c.is_alphabetic() || *c == '_' {
                *status = LexerState::Id;
                buffer.push(*c);
                Ok(None)
            } else {
                *status = LexerState::IntegerDone;
                Ok(emit_value(LexemeType::INTEGER, buffer))
            }
        }
        LexerState::Float => {
            if c.is_ascii_digit() {
                buffer.push(*c);
                Ok(None)
            } else if c.is_whitespace() {
                *status = LexerState::Start;
                Ok(emit_value(LexemeType::FLOAT, buffer))
            } else {
                *status = LexerState::FloatDone;
                Ok(emit_value(LexemeType::FLOAT, buffer))
            }
        }
        LexerState::Id => {
            if c.is_alphanumeric() || *c == '_' {
                buffer.push(*c);
                Ok(None)
            } else if c.is_whitespace() {
                *status = LexerState::Start;
                Ok(emit_value(LexemeType::ID, buffer))
            } else {
                *status = LexerState::IdDone;
                Ok(emit_value(LexemeType::ID, buffer))
            }
        }
        LexerState::InString => match c {
            '"' => {
                *status = LexerState::Start;
                Ok(emit_value(LexemeType::STRING, buffer))
            }
            '\\' => {
                *status = LexerState::Escape;
                Ok(None)
            }
            _ => {
                buffer.push(*c);
                Ok(None)
            }
        },
        LexerState::Sharp => match c {
            't' | 'T' => {
                *status = LexerState::Start;
                Ok(emit(LexemeType::TRUE))
            }
            'f' | 'F' => {
                *status = LexerState::Start;
                Ok(emit(LexemeType::FALSE))
            }
            '(' => {
                *status = LexerState::Start;
                Ok(emit(LexemeType::SHARP))
            }
            _ => Err(SrsError::new(SrsErrorKind::UnsupportedChar)),
        },
        LexerState::Lt => {
            if *c == '=' {
                *status = LexerState::Start;
                Ok(emit(LexemeType::LTE))
            } else {
                *status = LexerState::LtDone;
                Ok(emit(LexemeType::LT))
            }
        }
        LexerState::Gt => {
            if *c == '=' {
                *status = LexerState::Start;
                Ok(emit(LexemeType::GTE))
            } else {
                *status = LexerState::GtDone;
                Ok(emit(LexemeType::GT))
            }
        }
        LexerState::Escape => match c {
            '\\' | '"' => {
                *status = LexerState::InString;
                buffer.push(*c);
                Ok(None)
            }
            _ => Err(SrsError::new(SrsErrorKind::UnsupportedChar)),
        },
        LexerState::IntegerDone
        | LexerState::FloatDone
        | LexerState::IdDone
        | LexerState::LtDone
        | LexerState::GtDone => Ok(None),
    }
}

fn filter_end(status: &LexerState, buffer: &str) -> SrsResult<Option<Lexeme>> {
    match status {
        LexerState::Integer => Ok(emit_value(LexemeType::INTEGER, buffer)),
        LexerState::Float => Ok(emit_value(LexemeType::FLOAT, buffer)),
        LexerState::Id => Ok(emit_value(LexemeType::ID, buffer)),
        LexerState::InString | LexerState::Escape => {
            Err(SrsError::new(SrsErrorKind::UncompletedString))
        }
        LexerState::Start
        | LexerState::Sharp
        | LexerState::Lt
        | LexerState::Gt
        | LexerState::IntegerDone
        | LexerState::FloatDone
        | LexerState::IdDone
        | LexerState::LtDone
        | LexerState::GtDone => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretor::lexeme::LexemeType;

    #[test]
    fn test_number_and_float() {
        let v = get_lexemes(" 23 98.0 25").unwrap();
        assert_eq!(3, v.len());
        assert_eq!(LexemeType::INTEGER, v[0].lexeme_type);
        assert_eq!("23", v[0].value);
        assert_eq!(LexemeType::FLOAT, v[1].lexeme_type);
        assert_eq!("98.0", v[1].value);
        assert_eq!(LexemeType::INTEGER, v[2].lexeme_type);
    }

    #[test]
    fn test_id() {
        let v = get_lexemes("asz a_b ss ").unwrap();
        assert_eq!(3, v.len());
        assert_eq!(LexemeType::ID, v[0].lexeme_type);
    }

    #[test]
    fn test_string() {
        let v = get_lexemes(" \"ma demo \"").unwrap();
        assert_eq!(1, v.len());
        assert_eq!("ma demo ", v[0].value);
    }

    #[test]
    fn test_all() {
        let scm = r###"
            (define generate
              (lambda (init nb step)
                (cons init
                  (if (= nb 0)
                    '()
                    (generate (+ init step) (- nb 1) step)))))"###;

        let v = get_lexemes(scm).unwrap();
        assert_eq!(41, v.len());
        assert_eq!(LexemeType::ID, v[2].lexeme_type);
    }

    #[test]
    fn check_str1() {
        let v = get_lexemes("\"abcd\"").unwrap();
        assert_eq!(1, v.len());
        assert_eq!("abcd", v[0].value);
    }

    #[test]
    fn check_str2() {
        let r = get_lexemes("\"abcd\\\"");
        assert!(r.is_err());
    }

    #[test]
    fn check_str3() {
        let v = get_lexemes("\"ab\\\"cd\\\\\"").unwrap();
        assert_eq!(1, v.len());
    }
}
