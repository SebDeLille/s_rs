use crate::interpretor::lexeme::{Lexeme, LexemeType};
use crate::types::error::{SrsError, SrsErrorKind, SrsResult};

pub fn get_lexemes(s: &str) -> SrsResult<Vec<Lexeme>> {
    let mut list: Vec<Lexeme> = Vec::new();
    let mut status = 0u8;
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

fn is_star(status: u8) -> bool {
    matches!(status, 4 | 7 | 24 | 27)
}

fn filter(c: &char, status: &mut u8, buffer: &mut String) -> SrsResult<Vec<Lexeme>> {
    let mut result = Vec::new();

    if let Some(l) = filter_nobuffer(c, status, buffer)? {
        buffer.clear();
        result.push(l);
    }
    if is_star(*status) {
        *status = 0;
        if let Some(l) = filter_nobuffer(c, status, buffer)? {
            buffer.clear();
            result.push(l);
        }
    }

    Ok(result)
}

fn filter_nobuffer(
    c: &char,
    status: &mut u8,
    buffer: &mut String,
) -> SrsResult<Option<Lexeme>> {
    match status {
        0 => {
            if c.is_ascii_digit() {
                buffer.push(*c);
                *status = 1;
                Ok(None)
            } else if c.is_whitespace() {
                Ok(None)
            } else if c.is_alphabetic() || *c == '_' {
                buffer.push(*c);
                *status = 5;
                Ok(None)
            } else if *c == '"' {
                *status = 8;
                Ok(None)
            } else if *c == '#' {
                *status = 10;
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
                *status = 22;
                Ok(None)
            } else if *c == '>' {
                *status = 25;
                Ok(None)
            } else {
                Err(SrsError::with_message(
                    SrsErrorKind::UnsupportedChar,
                    format!("Unsupported char: {}", *c),
                ))
            }
        }
        1 => {
            if c.is_ascii_digit() {
                buffer.push(*c);
                Ok(None)
            } else if c.is_whitespace() {
                *status = 0;
                Ok(emit_value(LexemeType::INTEGER, buffer))
            } else if *c == '.' {
                *status = 2;
                buffer.push(*c);
                Ok(None)
            } else if c.is_alphabetic() || *c == '_' {
                *status = 5;
                buffer.push(*c);
                Ok(None)
            } else {
                *status = 4;
                Ok(emit_value(LexemeType::INTEGER, buffer))
            }
        }
        2 => {
            if c.is_ascii_digit() {
                buffer.push(*c);
                Ok(None)
            } else if c.is_whitespace() {
                *status = 0;
                Ok(emit_value(LexemeType::FLOAT, buffer))
            } else {
                *status = 4;
                Ok(emit_value(LexemeType::FLOAT, buffer))
            }
        }
        5 => {
            if c.is_alphanumeric() || *c == '_' {
                buffer.push(*c);
                Ok(None)
            } else if c.is_whitespace() {
                *status = 0;
                Ok(emit_value(LexemeType::ID, buffer))
            } else {
                *status = 7;
                Ok(emit_value(LexemeType::ID, buffer))
            }
        }
        8 => match c {
            '"' => {
                *status = 0;
                Ok(emit_value(LexemeType::STRING, buffer))
            }
            '\\' => {
                *status = 28;
                Ok(None)
            }
            _ => {
                buffer.push(*c);
                Ok(None)
            }
        },
        10 => match c {
            't' | 'T' => {
                *status = 0;
                Ok(emit(LexemeType::TRUE))
            }
            'f' | 'F' => {
                *status = 0;
                Ok(emit(LexemeType::FALSE))
            }
            '(' => {
                *status = 0;
                Ok(emit(LexemeType::SHARP))
            }
            _ => Err(SrsError::new(SrsErrorKind::UnsupportedChar)),
        },
        22 => {
            if *c == '=' {
                *status = 0;
                Ok(emit(LexemeType::LTE))
            } else {
                *status = 24;
                Ok(emit(LexemeType::LT))
            }
        }
        25 => {
            if *c == '=' {
                *status = 0;
                Ok(emit(LexemeType::GTE))
            } else {
                *status = 27;
                Ok(emit(LexemeType::GT))
            }
        }
        28 => match c {
            '\\' | '"' => {
                *status = 8;
                buffer.push(*c);
                Ok(None)
            }
            _ => Err(SrsError::new(SrsErrorKind::UnsupportedChar)),
        },
        _ => Ok(None),
    }
}

fn filter_end(status: &u8, buffer: &str) -> SrsResult<Option<Lexeme>> {
    match status {
        1 => Ok(emit_value(LexemeType::INTEGER, buffer)),
        2 => Ok(emit_value(LexemeType::FLOAT, buffer)),
        5 => Ok(emit_value(LexemeType::ID, buffer)),
        8 | 28 => Err(SrsError::new(SrsErrorKind::UncompletedString)),
        _ => Ok(None),
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
