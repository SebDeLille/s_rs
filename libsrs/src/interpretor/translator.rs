use std::slice::Iter;

use crate::interpretor::lexeme::{Lexeme, LexemeType};
use crate::types::core::SrsValue;
use crate::types::error::{SrsError, SrsErrorKind, SrsResult};

pub fn translate_all(elements: Vec<Lexeme>) -> SrsResult<Vec<SrsValue>> {
    let mut values: Vec<SrsValue> = Vec::new();
    let mut it = elements.iter();
    while let Some(value) = translate(&mut it)? {
        values.push(value);
    }
    Ok(values)
}

fn translate(it: &mut Iter<'_, Lexeme>) -> SrsResult<Option<SrsValue>> {
    while let Some(lexeme) = it.next() {
        return if lexeme.lexeme_type == LexemeType::LPAR {
            translate_list(it)
        } else {
            translate_atom(lexeme)
        };
    }
    Ok(None)
}

fn translate_list(it: &mut Iter<'_, Lexeme>) -> SrsResult<Option<SrsValue>> {
    let mut list = Vec::new();
    while let Some(lexeme) = it.next() {
        match lexeme.lexeme_type {
            LexemeType::LPAR => match translate_list(it)? {
                Some(v) => list.push(v),
                None => return Err(SrsError::new(SrsErrorKind::UnexpectedEnd)),
            },
            LexemeType::RPAR => return Ok(Some(SrsValue::List(list))),
            _ => match translate_atom(lexeme)? {
                Some(v) => list.push(v),
                None => return Err(SrsError::new(SrsErrorKind::UnexpectedEnd)),
            },
        }
    }
    Err(SrsError::new(SrsErrorKind::UnbalancedParen))
}

fn translate_atom(lexeme: &Lexeme) -> SrsResult<Option<SrsValue>> {
    match lexeme.lexeme_type {
        LexemeType::INTEGER => Ok(Some(SrsValue::Integer(lexeme.value.parse::<i64>()?))),
        LexemeType::FLOAT => Ok(Some(SrsValue::Float(lexeme.value.parse::<f64>()?))),
        LexemeType::ID => Ok(Some(SrsValue::Id(lexeme.value.clone()))),
        LexemeType::STRING => Ok(Some(SrsValue::String(lexeme.value.clone()))),
        LexemeType::TRUE => Ok(Some(SrsValue::Bool(true))),
        LexemeType::FALSE => Ok(Some(SrsValue::Bool(false))),
        LexemeType::ADD => Ok(Some(SrsValue::Id("+".to_string()))),
        LexemeType::SUB => Ok(Some(SrsValue::Id("-".to_string()))),
        LexemeType::MUL => Ok(Some(SrsValue::Id("*".to_string()))),
        LexemeType::DIV => Ok(Some(SrsValue::Id("/".to_string()))),
        LexemeType::EQ => Ok(Some(SrsValue::Id("=".to_string()))),
        LexemeType::GT => Ok(Some(SrsValue::Id(">".to_string()))),
        LexemeType::GTE => Ok(Some(SrsValue::Id(">=".to_string()))),
        LexemeType::LT => Ok(Some(SrsValue::Id("<".to_string()))),
        LexemeType::LTE => Ok(Some(SrsValue::Id("<=".to_string()))),
        LexemeType::QUOTE => Ok(None),
        _ => Err(SrsError::new(SrsErrorKind::UnknownType)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretor::lexical_analyzer::get_lexemes;

    #[test]
    fn easy() {
        let scm = r###"
            (define generate
              (lambda (init nb step)
                (cons init
                  (if (= nb 0)
                    ()
                    (generate (+ init step) (- nb 1) step)))))"###;
        let lexemes = get_lexemes(scm).unwrap();
        let result = translate_all(lexemes);
        assert!(result.is_ok());
        assert_eq!(1, result.unwrap().len());
    }

    #[test]
    fn test_integer() {
        let v = translate_all(get_lexemes("23 56").unwrap()).unwrap();
        assert_eq!(2, v.len());
        assert!(matches!(v[0], SrsValue::Integer(23)));
        assert!(matches!(v[1], SrsValue::Integer(56)));
    }

    #[test]
    fn test_basic_expression() {
        let v = translate_all(get_lexemes("(add 2 3)").unwrap()).unwrap();
        assert_eq!(1, v.len());
        assert!(v[0].is_list());
    }

    #[test]
    fn nested_expression() {
        let v = translate_all(get_lexemes("(+ (* 2 3) 4)").unwrap()).unwrap();
        assert_eq!(1, v.len());
        if let SrsValue::List(list) = &v[0] {
            assert_eq!(3, list.len());
        } else {
            panic!("expected list");
        }
    }
}
