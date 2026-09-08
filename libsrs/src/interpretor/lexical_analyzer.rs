use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Lexeme {
    LParen,
    RParen,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    VectorOpen,
    Dot,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    String(String),
    Id(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LexerState {
    Start,
    LineComment,
    Sharp,
    BlockComment,
    BlockCommentBar,
    BlockCommentHash,
    Unquote,
    CharSharp,
    CharName,
    Id,
    Sign,
    DotStart,
    DotDot,
    Integer,
    Float,
    Exponent,
    ExponentSign,
    ExponentDigits,
    InString,
    Escape,
}

#[derive(Debug, Clone, PartialEq)]
struct State {
    kind: LexerState,
    token: String,
}

pub fn get_lexemes(input: &str) -> Result<Vec<Lexeme>, String> {
    let mut state = State {
        kind: LexerState::Start,
        token: String::new(),
    };
    let mut lexemes: Vec<Lexeme> = Vec::new();
    let mut chars = input.chars().peekable();
    let mut block_depth: u32 = 0;

    while let Some(c) = take(&mut chars) {
        let (new_state, emit, reprocess) = transition(&mut state, c, &mut chars, &mut block_depth)?;

        state.kind = new_state;

        if let Some(lexeme) = emit {
            lexemes.push(lexeme);
        }

        if reprocess {
            state.token.clear();
            state.kind = LexerState::Start;
            match c {
                '(' => lexemes.push(Lexeme::LParen),
                ')' => lexemes.push(Lexeme::RParen),
                '\'' => lexemes.push(Lexeme::Quote),
                '`' => lexemes.push(Lexeme::Quasiquote),
                ',' => lexemes.push(Lexeme::Unquote),
                '#' => state.kind = LexerState::Sharp,
                '.' => state.kind = LexerState::DotStart,
                '"' => state.kind = LexerState::InString,
                c if is_initial(c) => {
                    state.kind = LexerState::Id;
                    state.token.push(c);
                }
                c if c.is_ascii_digit() => {
                    state.kind = LexerState::Integer;
                    state.token.push(c);
                }
                c if is_sign(c) => {
                    state.kind = LexerState::Sign;
                    state.token.push(c);
                }
                c if c.is_whitespace() => {
                    state.kind = LexerState::Start;
                }
                ';' => state.kind = LexerState::LineComment,
                _ => {
                    return Err(format!("unexpected character '{}'", c));
                }
            }
        }
    }

    finish(state, &mut lexemes)?;
    Ok(lexemes)
}

fn take(chars: &mut Peekable<Chars<'_>>) -> Option<char> {
    chars.next()
}

fn is_initial(c: char) -> bool {
    c.is_ascii_alphabetic()
        || matches!(
            c,
            '!' | '$' | '%' | '&' | '*' | '/' | ':' | '<' | '=' | '>' | '?' | '^' | '_' | '~'
        )
}

fn is_subsequent(c: char) -> bool {
    is_initial(c) || c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | '@')
}

fn is_sign(c: char) -> bool {
    c == '+' || c == '-'
}

fn is_delimiter(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '(' | ')' | '\'' | '`' | ',' | '"' | ';' | '#' | '|' | '[' | ']' | '{' | '}'
        )
}

fn transition(
    state: &mut State,
    c: char,
    _chars: &mut Peekable<Chars<'_>>,
    block_depth: &mut u32,
) -> Result<(LexerState, Option<Lexeme>, bool), String> {
    use LexerState::*;

    match state.kind {
        Start => match c {
            c if c.is_whitespace() => Ok((Start, None, false)),
            ';' => Ok((LineComment, None, false)),
            '#' => Ok((Sharp, None, false)),
            '(' => Ok((Start, Some(Lexeme::LParen), false)),
            ')' => Ok((Start, Some(Lexeme::RParen), false)),
            '\'' => Ok((Start, Some(Lexeme::Quote), false)),
            '`' => Ok((Start, Some(Lexeme::Quasiquote), false)),
            ',' => Ok((Unquote, None, false)),
            '"' => Ok((InString, None, false)),
            '.' => Ok((DotStart, None, false)),
            c if is_initial(c) => {
                state.token.push(c);
                Ok((Id, None, false))
            }
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((Integer, None, false))
            }
            c if is_sign(c) => {
                state.token.push(c);
                Ok((Sign, None, false))
            }
            _ => Err(format!("unexpected character '{}'", c)),
        },

        LineComment => Ok((if c == '\n' { Start } else { LineComment }, None, false)),

        Sharp => match c {
            '|' => {
                *block_depth = 1;
                Ok((BlockComment, None, false))
            }
            't' | 'T' => Ok((Start, Some(Lexeme::Boolean(true)), false)),
            'f' | 'F' => Ok((Start, Some(Lexeme::Boolean(false)), false)),
            '(' => Ok((Start, Some(Lexeme::VectorOpen), false)),
            '\\' => Ok((CharSharp, None, false)),
            _ => Err(format!("unexpected sharp dispatch '{}'", c)),
        },

        BlockComment => match c {
            '|' => Ok((BlockCommentBar, None, false)),
            '#' => Ok((BlockCommentHash, None, false)),
            _ => Ok((BlockComment, None, false)),
        },

        BlockCommentBar => match c {
            '#' => {
                *block_depth -= 1;
                if *block_depth == 0 {
                    Ok((Start, None, false))
                } else {
                    Ok((BlockComment, None, false))
                }
            }
            '|' => Ok((BlockCommentBar, None, false)),
            _ => Ok((BlockComment, None, false)),
        },

        BlockCommentHash => match c {
            '|' => {
                *block_depth += 1;
                Ok((BlockComment, None, false))
            }
            _ => Ok((BlockComment, None, false)),
        },

        Unquote => match c {
            '@' => Ok((Start, Some(Lexeme::UnquoteSplicing), false)),
            c if c.is_whitespace() || is_delimiter(c) => Ok((Start, Some(Lexeme::Unquote), true)),
            _ => Ok((Start, Some(Lexeme::Unquote), true)),
        },

        CharSharp => match c {
            c if c.is_ascii_alphabetic() => {
                state.token.push(c);
                Ok((CharName, None, false))
            }
            _ => Ok((Start, Some(Lexeme::Character(c)), false)),
        },

        CharName => match c {
            c if c.is_ascii_alphabetic() => {
                state.token.push(c);
                Ok((CharName, None, false))
            }
            c if c.is_whitespace() || is_delimiter(c) => {
                let ch = resolve_char_name(&state.token)?;
                Ok((Start, Some(Lexeme::Character(ch)), true))
            }
            _ => {
                let ch = resolve_char_name(&state.token)?;
                Ok((Start, Some(Lexeme::Character(ch)), true))
            }
        },

        Id => match c {
            c if is_subsequent(c) => {
                state.token.push(c);
                Ok((Id, None, false))
            }
            c if c.is_whitespace() || is_delimiter(c) => {
                let id = std::mem::take(&mut state.token);
                Ok((Start, Some(Lexeme::Id(id)), true))
            }
            _ => {
                let id = std::mem::take(&mut state.token);
                Ok((Start, Some(Lexeme::Id(id)), true))
            }
        },

        Sign => match c {
            c if c.is_whitespace() || is_delimiter(c) => {
                let sign = std::mem::take(&mut state.token);
                Ok((Start, Some(Lexeme::Id(sign)), true))
            }
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((Integer, None, false))
            }
            '.' => Ok((DotStart, None, false)),
            c if is_initial(c) || is_subsequent(c) => {
                state.token.push(c);
                Ok((Id, None, false))
            }
            _ => Ok((Start, Some(Lexeme::Id(state.token.clone())), true)),
        },

        DotStart => match c {
            c if c.is_ascii_digit() => {
                state.token.insert(0, '.');
                state.token.push(c);
                Ok((Float, None, false))
            }
            '.' => Ok((DotDot, None, false)),
            c if c.is_whitespace() || is_delimiter(c) => Ok((Start, Some(Lexeme::Dot), true)),
            _ => Ok((Start, Some(Lexeme::Dot), true)),
        },

        DotDot => match c {
            '.' => {
                state.token = "...".to_string();
                Ok((Start, Some(Lexeme::Id(state.token.clone())), false))
            }
            _ => Err(format!("invalid token '..{}'", c)),
        },

        Integer => match c {
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((Integer, None, false))
            }
            '.' => {
                state.token.push(c);
                Ok((Float, None, false))
            }
            'e' | 'E' => {
                state.token.push(c);
                Ok((Exponent, None, false))
            }
            '+' | '-' => {
                state.token.push(c);
                Ok((Id, None, false))
            }
            c if is_initial(c) => {
                state.token.push(c);
                Ok((Id, None, false))
            }
            c if c.is_whitespace() || is_delimiter(c) => {
                let value = state
                    .token
                    .parse()
                    .map_err(|e| format!("bad integer: {}", e))?;
                Ok((Start, Some(Lexeme::Integer(value)), true))
            }
            _ => {
                let value = state
                    .token
                    .parse()
                    .map_err(|e| format!("bad integer: {}", e))?;
                Ok((Start, Some(Lexeme::Integer(value)), true))
            }
        },

        Float => match c {
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((Float, None, false))
            }
            'e' | 'E' => {
                state.token.push(c);
                Ok((Exponent, None, false))
            }
            c if c.is_whitespace() || is_delimiter(c) => {
                let value = parse_float(&state.token)?;
                Ok((Start, Some(Lexeme::Float(value)), true))
            }
            _ => {
                let value = parse_float(&state.token)?;
                Ok((Start, Some(Lexeme::Float(value)), true))
            }
        },

        Exponent => match c {
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((ExponentDigits, None, false))
            }
            '+' | '-' => {
                state.token.push(c);
                Ok((ExponentSign, None, false))
            }
            _ => Err(format!("expected exponent digits, got '{}'", c)),
        },

        ExponentSign => match c {
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((ExponentDigits, None, false))
            }
            _ => Err(format!("expected exponent digits after sign, got '{}'", c)),
        },

        ExponentDigits => match c {
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((ExponentDigits, None, false))
            }
            c if c.is_whitespace() || is_delimiter(c) => {
                let value = parse_float(&state.token)?;
                Ok((Start, Some(Lexeme::Float(value)), true))
            }
            _ => {
                let value = parse_float(&state.token)?;
                Ok((Start, Some(Lexeme::Float(value)), true))
            }
        },

        InString => match c {
            '"' => {
                let s = std::mem::take(&mut state.token);
                Ok((Start, Some(Lexeme::String(s)), false))
            }
            '\\' => Ok((Escape, None, false)),
            _ => {
                state.token.push(c);
                Ok((InString, None, false))
            }
        },

        Escape => match c {
            '\\' => {
                state.token.push('\\');
                Ok((InString, None, false))
            }
            '"' => {
                state.token.push('"');
                Ok((InString, None, false))
            }
            'n' => {
                state.token.push('\n');
                Ok((InString, None, false))
            }
            't' => {
                state.token.push('\t');
                Ok((InString, None, false))
            }
            'r' => {
                state.token.push('\r');
                Ok((InString, None, false))
            }
            _ => Err(format!("unknown escape sequence '\\{}'", c)),
        },
    }
}

fn finish(state: State, lexemes: &mut Vec<Lexeme>) -> Result<(), String> {
    use LexerState::*;

    match state.kind {
        Start | LineComment => Ok(()),
        Integer => {
            let value = state
                .token
                .parse()
                .map_err(|e| format!("bad integer: {}", e))?;
            lexemes.push(Lexeme::Integer(value));
            Ok(())
        }
        Float | ExponentDigits => {
            let value = parse_float(&state.token)?;
            lexemes.push(Lexeme::Float(value));
            Ok(())
        }
        Sign | Id | DotDot => {
            lexemes.push(Lexeme::Id(state.token));
            Ok(())
        }
        DotStart => {
            lexemes.push(Lexeme::Dot);
            Ok(())
        }
        CharName => {
            let ch = resolve_char_name(&state.token)?;
            lexemes.push(Lexeme::Character(ch));
            Ok(())
        }
        CharSharp => Err(format!("unterminated token: {:?}", state)),
        Unquote => {
            lexemes.push(Lexeme::Unquote);
            Ok(())
        }
        Sharp => Err("unterminated sharp dispatch".to_string()),
        BlockComment | BlockCommentBar | BlockCommentHash => {
            Err("unterminated block comment".to_string())
        }
        InString | Escape => Err("unterminated string".to_string()),
        Exponent | ExponentSign => Err("unterminated exponent".to_string()),
    }
}

fn parse_float(token: &str) -> Result<f64, String> {
    token
        .parse()
        .map_err(|e| format!("bad float '{}': {}", token, e))
}

fn resolve_char_name(name: &str) -> Result<char, String> {
    match name {
        "space" => Ok(' '),
        "newline" => Ok('\n'),
        _ if name.len() == 1 => Ok(name.chars().next().unwrap()),
        _ => Err(format!("unknown character name '#\\{}'", name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parens() {
        assert_eq!(
            get_lexemes("()").unwrap(),
            vec![Lexeme::LParen, Lexeme::RParen]
        );
    }

    #[test]
    fn quote_unquote() {
        assert_eq!(
            get_lexemes("'`,@").unwrap(),
            vec![Lexeme::Quote, Lexeme::Quasiquote, Lexeme::UnquoteSplicing,]
        );
        assert_eq!(
            get_lexemes("'`, (unquote-splicing x)").unwrap(),
            vec![
                Lexeme::Quote,
                Lexeme::Quasiquote,
                Lexeme::Unquote,
                Lexeme::LParen,
                Lexeme::Id("unquote-splicing".to_string()),
                Lexeme::Id("x".to_string()),
                Lexeme::RParen,
            ]
        );
    }

    #[test]
    fn line_comment() {
        assert_eq!(
            get_lexemes("; hello\n42").unwrap(),
            vec![Lexeme::Integer(42)]
        );
    }

    #[test]
    fn block_comment() {
        assert_eq!(
            get_lexemes("#| nested #| comment |# || #| ignored |# |# 7").unwrap(),
            vec![Lexeme::Integer(7)]
        );
    }

    #[test]
    fn integer_and_float() {
        assert_eq!(
            get_lexemes("42 3.14 1e10 -3.2e-5").unwrap(),
            vec![
                Lexeme::Integer(42),
                Lexeme::Float(3.14),
                Lexeme::Float(1e10),
                Lexeme::Float(-3.2e-5),
            ]
        );
    }

    #[test]
    fn identifiers() {
        assert_eq!(
            get_lexemes("foo ->string 1+ list->vector + - ...").unwrap(),
            vec![
                Lexeme::Id("foo".to_string()),
                Lexeme::Id("->string".to_string()),
                Lexeme::Id("1+".to_string()),
                Lexeme::Id("list->vector".to_string()),
                Lexeme::Id("+".to_string()),
                Lexeme::Id("-".to_string()),
                Lexeme::Id("...".to_string()),
            ]
        );
    }

    #[test]
    fn booleans_chars() {
        assert_eq!(
            get_lexemes("#t #f #\\x #\\space #\\newline").unwrap(),
            vec![
                Lexeme::Boolean(true),
                Lexeme::Boolean(false),
                Lexeme::Character('x'),
                Lexeme::Character(' '),
                Lexeme::Character('\n'),
            ]
        );
    }

    #[test]
    fn strings() {
        assert_eq!(
            get_lexemes("\"hello\\nworld\"").unwrap(),
            vec![Lexeme::String("hello\nworld".to_string())]
        );
    }

    #[test]
    fn vector_open() {
        assert_eq!(
            get_lexemes("#(1 2)").unwrap(),
            vec![
                Lexeme::VectorOpen,
                Lexeme::Integer(1),
                Lexeme::Integer(2),
                Lexeme::RParen,
            ]
        );
    }

    #[test]
    fn dot_symbol() {
        assert_eq!(
            get_lexemes("(1 . 2)").unwrap(),
            vec![
                Lexeme::LParen,
                Lexeme::Integer(1),
                Lexeme::Dot,
                Lexeme::Integer(2),
                Lexeme::RParen,
            ]
        );
    }

    #[test]
    fn unknown_char_name_errors() {
        assert!(get_lexemes("#\\foobar").is_err());
    }

    #[test]
    fn unterminated_string_errors() {
        assert!(get_lexemes("\"hello").is_err());
    }
}
