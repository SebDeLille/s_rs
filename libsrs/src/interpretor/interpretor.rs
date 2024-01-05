use crate::interpretor::lexeme::{Lexeme, LexemeType};

pub fn get_lexemes(s: &String) -> Vec<Lexeme> {
    let mut list: Vec<Lexeme> = Vec::new();
    let mut status = 0_u8;
    let mut buffer = String::new();

    for c in s.chars() {
        let l = filter(&c, &mut status, &mut buffer);
        if l.is_some() {
            list.push(l.unwrap());
            buffer.clear();
        }
    }

    list
}

fn filter(c: &char, status: &mut u8, buffer: &mut String) -> Option<Lexeme> {
    match status {
        0 => {
            if c.is_digit(10) {
                buffer.push(*c);
                *status = 1;
                return None
            } else {
                return None
            }
        }
        1 => {
            if c.is_digit(10) {
                buffer.push(*c);
                return None
            } else if c.is_whitespace() {
                *status = 0;
                return Some(Lexeme::new(LexemeType::INTEGER, buffer.clone()))
            } else if *c == '.' {
                *status = 2;
                buffer.push(*c);
                return None
            }
            else {
                return  None
            }
        }
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use crate::interpretor::interpretor::get_lexemes;

    #[test]
    fn demo() {
        get_lexemes(&"(+ 2 3)".to_string());
    }
}