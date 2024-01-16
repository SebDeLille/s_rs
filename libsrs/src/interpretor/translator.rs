use std::slice::Iter;
use crate::interpretor::lexeme::{Lexeme, LexemeType};
use crate::types::core::{SrsElement, SrsValue};
use crate::types::error::{SrsError, SrsResult};
use crate::types::id::SrsId;
use crate::types::integer::SrsInteger;
use crate::types::list::SrsList;
use crate::types::string::SrsString;

pub fn translate_all(elements: Vec<Lexeme>) -> Result<Vec<Box<dyn SrsElement>>, SrsError> {
    let mut values: Vec<Box<dyn SrsElement>> = Vec::new();

    let mut it = elements.iter();
    while let Some(value) = translate(&mut it)? {
        values.push(value);
    }

    Ok(values)
}

fn translate(it: &mut Iter<'_, Lexeme>) -> SrsResult<SrsValue> {
    while let Some(lexeme) = it.next() {
        return if lexeme.lexeme_type == LexemeType::LPAR {
            translate_list(it)
        } else {
            translate_atom(&lexeme)
        };
    }
    Ok(None)
}

fn translate_list(it: &mut Iter<'_, Lexeme>) -> SrsResult<SrsValue> {
    let mut list = SrsList::default();
    while let Some(lexeme) = it.next() {
        match lexeme.lexeme_type {
            LexemeType::LPAR => {
                match translate_list(it) {
                    Ok(v) => list.add_tail(v)?,
                    Err(e) => return Err(e)
                }
            },
            LexemeType::RPAR => return Ok(Some(Box::new(list))),
            _ => {
                match translate_atom(lexeme) {
                    Ok(v) => list.add_tail(v)?,
                    Err(e) => return Err(e)
                }
            }
        }
    }
    Err(SrsError{})
}

fn translate_atom(lexeme: &Lexeme) -> SrsResult<SrsValue> {
    match lexeme.lexeme_type {
        LexemeType::INTEGER => Ok(Some(Box::new(SrsInteger::new(lexeme.value.parse::<i64>()?)))),
        LexemeType::ID => Ok(Some(Box::new(SrsId::new(lexeme.value.clone())))),
        LexemeType::STRING => Ok(Some(Box::new(SrsString::new(lexeme.value.clone())))),
        _ => Err(SrsError {})
    }
}

#[cfg(test)]
mod tests {
    use crate::interpretor::lexical_analyzer::get_lexemes;
    use crate::interpretor::translator::translate_all;

    #[test]
    fn easy() {
        let scm = r###"
            (define generate
              (lambda (init nb step)
                (cons init
                  (if (= nb 0)
                    '()
                    (generate (+ init step) (- nb 1) step)))))"###;
        let result1 = get_lexemes(&scm.to_string());
        if result1.is_ok() {
            let _ = translate_all(result1.unwrap());
        }
    }

    #[test]
    fn test_integer() {
        let scm = "23 56";
        let tmp_res = get_lexemes(&scm.to_string());
        if tmp_res.is_ok() {
            let result = translate_all(tmp_res.unwrap());
            match result {
                Ok(v) => {
                    assert_eq!(2, v.len());
                    v.iter().for_each(|i| { println!("{:?}", i.get_type()) });
                }
                Err(_) => panic!("perdu")
            }
        }
    }

    #[test]
    fn test_basic_expression() {
        let scm = "(add 2 3)";
        let tmp_res = get_lexemes(&scm.to_string());
        if tmp_res.is_ok() {
            let result = translate_all(tmp_res.unwrap());
            match result {
                Ok(v) => {
                    assert_eq!(1, v.len());
                }
                Err(_) => panic!("perdu")
            }
        }
    }
}

