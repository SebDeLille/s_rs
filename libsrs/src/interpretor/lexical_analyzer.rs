use crate::interpretor::error::SrsError;
use crate::interpretor::lexeme::{Lexeme, LexemeType};

pub fn get_lexemes(s: &String) -> Result<Vec<Lexeme>, SrsError> {
    let mut list: Vec<Lexeme> = Vec::new();
    let mut status = 0_u8;
    let mut buffer = String::new();

    for c in s.chars() {
        filter(&c, &mut status, &mut buffer)?.iter().for_each(|l| { list.push(l.clone()) });
    }

    if let Some(l) = filter_end(&status, &buffer)? {
        list.push(l);
    }

    Ok(list)
}

fn is_star(status: &u8) -> bool {
    match status {
        4 | 7 | 24 | 27 => true,
        _ => false
    }
}

fn filter(c: &char, status: &mut u8, buffer: &mut String) -> Result<Vec<Lexeme>, SrsError> {
    let mut result = Vec::new();

    if let Some(l) = filter_nobuffer(c, status, buffer)? {
        buffer.clear();
        result.push(l);
    }
    if is_star(status) {
        *status = 0;
        if let Some(l) = filter_nobuffer(c, status, buffer)? {
            buffer.clear();
            result.push(l);
        }
    }

    Ok(result)
}

fn filter_nobuffer(c: &char, status: &mut u8, buffer: &mut String) -> Result<Option<Lexeme>, SrsError> {
    // println!("status : {}, c : {}", status, *c);
    match status {
        0 => {
            if c.is_digit(10) {
                buffer.push(*c);
                *status = 1;
                return Ok(None);
            } else if c.is_whitespace() {
                return Ok(None);
            } else if c.is_alphabetic() {
                buffer.push(*c);
                *status = 5;
                return Ok(None);
            } else if *c == '"' {
                *status = 8;
                return Ok(None);
            } else if *c == '#' {
                *status = 10;
                return Ok(None);
            } else if *c == '(' {
                return Ok(Some(Lexeme::new(LexemeType::LPAR, "".to_string())));
            } else if *c == ')' {
                return Ok(Some(Lexeme::new(LexemeType::RPAR, "".to_string())));
            } else if *c == '+' {
                return Ok(Some(Lexeme::new(LexemeType::ADD, "".to_string())));
            } else if *c == '-' {
                return Ok(Some(Lexeme::new(LexemeType::SUB, "".to_string())));
            } else if *c == '*' {
                return Ok(Some(Lexeme::new(LexemeType::MUL, "".to_string())));
            } else if *c == '/' {
                return Ok(Some(Lexeme::new(LexemeType::DIV, "".to_string())));
            } else if *c == '\'' {
                return Ok(Some(Lexeme::new(LexemeType::QUOTE, "".to_string())));
            } else if *c == '=' {
                return Ok(Some(Lexeme::new(LexemeType::EQ, "".to_string())));
            } else if *c == '<' {
                *status = 22;
                return Ok(None);
            } else if *c == '>' {
                *status = 25;
                return Ok(None);
            } else {
                return Err(SrsError::new("Unsupported char".to_string()));
            }
        }
        1 => {
            if c.is_digit(10) {
                buffer.push(*c);
                return Ok(None);
            } else if c.is_whitespace() {
                *status = 0;
                return Ok(Some(Lexeme::new(LexemeType::INTEGER, buffer.clone())));
            } else if *c == '.' {
                *status = 2;
                buffer.push(*c);
                return Ok(None);
            } else if c.is_alphabetic() {
                *status = 5;
                buffer.push(*c);
                return Ok(None);
            } else {
                *status = 4;
                return Ok(Some(Lexeme::new(LexemeType::INTEGER, buffer.clone())));
            }
        }
        2 => {
            if c.is_digit(10) {
                buffer.push(*c);
                return Ok(None);
            } else if c.is_whitespace() {
                *status = 0;
                return Ok(Some(Lexeme::new(LexemeType::INTEGER, buffer.clone())));
            } else {
                *status = 4;
                return Ok(Some(Lexeme::new(LexemeType::INTEGER, buffer.clone())));
            }
        }
        5 => {
            if c.is_alphanumeric() || *c == '_' {
                buffer.push(*c);
                return Ok(None);
            } else if c.is_whitespace() {
                *status = 0;
                return Ok(Some(Lexeme::new(LexemeType::ID, buffer.clone())));
            } else {
                *status = 7;
                return Ok(Some(Lexeme::new(LexemeType::ID, buffer.clone())));
            }
        }
        8 => {
            match c {
                '"' => {
                    *status = 0;
                    return Ok(Some(Lexeme::new(LexemeType::STRING, buffer.clone())));
                }
                '\\' => {
                    *status = 28;
                    return Ok(None);
                }
                _ => {
                    buffer.push(*c);
                    return Ok(None);
                }
            }
        }
        10 => {
            match *c {
                't' | 'T' => {
                    *status = 0;
                    return Ok(Some(Lexeme::new(LexemeType::TRUE, "".to_string())));
                }
                'f' | 'F' => {
                    *status = 0;
                    return Ok(Some(Lexeme::new(LexemeType::FALSE, "".to_string())));
                }
                '(' => {
                    *status = 0;
                    return Ok(Some(Lexeme::new(LexemeType::SHARP, "".to_string())));
                }
                _ => return Err(SrsError::new("Unsupported char".to_string()))
            }
        }
        22 => {
            match *c {
                '=' => {
                    *status = 0;
                    return Ok(Some(Lexeme::new(LexemeType::LTE, "".to_string())));
                }
                _ => {
                    *status = 24;
                    return Ok(Some(Lexeme::new(LexemeType::LT, "".to_string())));
                }
            }
        }
        25 => {
            match *c {
                '=' => {
                    *status = 0;
                    return Ok(Some(Lexeme::new(LexemeType::GTE, "".to_string())));
                }
                _ => {
                    *status = 27;
                    return Ok(Some(Lexeme::new(LexemeType::GT, "".to_string())));
                }
            }
        }
        28 => {
            match *c {
                '\\' | '\"' => {
                    *status = 8;
                    buffer.push(*c);
                    return Ok(None);
                }
                _ => return Err(SrsError::new("Unsupported char".to_string()))
            }
        }
        _ => Ok(None)
    }
}

fn filter_end(status: &u8, buffer: &String) -> Result<Option<Lexeme>, SrsError> {
    match status {
        1 => Ok(Some(Lexeme::new(LexemeType::INTEGER, buffer.clone()))),
        5 => Ok(Some(Lexeme::new(LexemeType::ID, buffer.clone()))),
        8 | 28 => Err(SrsError::new("Uncompleted string".to_string())),
        _ => Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::interpretor::lexical_analyzer::get_lexemes;
    use crate::interpretor::lexeme::LexemeType;

    #[test]
    fn test_number() {
        let result = get_lexemes(&" 23 98.0 25".to_string());
        match result {
            Ok(v) => {
                assert_eq!(3, v.len());
                v.iter().for_each(|value| { println!("|{}|", value.value) })
            }
            Err(e) => panic!("{}", e.get_message())
        }
    }

    #[test]
    fn test_id() {
        let result = get_lexemes(&"asz a_b ss ".to_string());
        match result {
            Ok(v) => {
                assert_eq!(3, v.len());
                v.iter().for_each(|value| { println!("|{}|", value.value) })
            }
            Err(e) => panic!("{}", e.get_message())
        }
    }

    #[test]
    fn test_string() {
        let result = get_lexemes(&" \"ma demo \"".to_string());
        match result {
            Ok(v) => {
                assert_eq!(1, v.len());
                assert_eq!("ma demo ", v.get(0).unwrap().value);
            }
            Err(e) => panic!("{}", e.get_message())
        }
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

        let result = get_lexemes(&scm.to_string());
        match result {
            Ok(v) => {
                assert_eq!(41, v.len());
                match v[2].lexeme_type {
                    LexemeType::ID => {}
                    _ => panic!("Bad type")
                }
                v.iter().for_each(|value| { println!("|{:?}|", value.lexeme_type) })
            }
            Err(e) => panic!("{}", e.get_message())
        }
    }

    #[test]
    fn check_str1() {
        let result = get_lexemes(&"\"abcd\"".to_string());
        match result {
            Ok(v) => {
                assert_eq!(1, v.len());
                assert_eq!("abcd".to_string(), v[0].value);
            }
            Err(e) => panic!("{}", e.get_message())
        }
    }

    #[test]
    fn check_str2() {
        let result = get_lexemes(&"\"abcd\\\"".to_string());
        match result {
            Ok(_) => {
                panic!("An error must be semt");
            }
            Err(_) => {}
        }
    }

    #[test]
    fn check_str3() {
        let result = get_lexemes(&"\"ab\\\"cd\\\\\"".to_string());
        match result {
            Ok(v) => {
                println!("{}", v[0].value.clone());
                assert_eq!(1, v.len());
            }
            Err(e) => panic!("{}", e.get_message())
        }
    }
}