use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

/// Position of a character in the source, 1-based for both line and column
/// (matching common editor conventions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// The specific reason a [`LexError`] was raised.
#[derive(Debug, Clone, PartialEq)]
pub enum LexErrorKind {
    UnexpectedChar(char),
    UnexpectedSharpDispatch(char),
    InvalidDotDotSequence(char),
    ExpectedExponentDigit(char),
    UnknownCharName(String),
    UnknownEscape(char),
    InvalidInteger(String),
    InvalidFloat(String),
    UnterminatedCharacter,
    UnterminatedSharpDispatch,
    UnterminatedBlockComment,
    UnterminatedString,
    UnterminatedExponent,
    /// Internal invariant violation: the block-comment nesting counter
    /// underflowed. This should be unreachable given the state machine's
    /// design; if it triggers, it indicates a bug in the lexer itself
    /// rather than in the input source.
    BlockCommentDepthUnderflow,
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErrorKind::UnexpectedChar(c) => write!(f, "unexpected character '{}'", c),
            LexErrorKind::UnexpectedSharpDispatch(c) => {
                write!(f, "unexpected sharp dispatch '#{}'", c)
            }
            LexErrorKind::InvalidDotDotSequence(c) => write!(f, "invalid token '..{}'", c),
            LexErrorKind::ExpectedExponentDigit(c) => {
                write!(f, "expected exponent digit, got '{}'", c)
            }
            LexErrorKind::UnknownCharName(name) => {
                write!(f, "unknown character name '#\\{}'", name)
            }
            LexErrorKind::UnknownEscape(c) => write!(f, "unknown escape sequence '\\{}'", c),
            LexErrorKind::InvalidInteger(token) => write!(f, "invalid integer literal '{}'", token),
            LexErrorKind::InvalidFloat(token) => write!(f, "invalid float literal '{}'", token),
            LexErrorKind::UnterminatedCharacter => write!(f, "unterminated character literal"),
            LexErrorKind::UnterminatedSharpDispatch => write!(f, "unterminated sharp dispatch"),
            LexErrorKind::UnterminatedBlockComment => write!(f, "unterminated block comment"),
            LexErrorKind::UnterminatedString => write!(f, "unterminated string literal"),
            LexErrorKind::UnterminatedExponent => write!(f, "unterminated exponent"),
            LexErrorKind::BlockCommentDepthUnderflow => {
                write!(f, "internal lexer error: block comment depth underflow")
            }
        }
    }
}

/// A lexical error, carrying both the reason ([`LexErrorKind`]) and the
/// [`Position`] in the source where it was detected.
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub pos: Position,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.pos, self.kind)
    }
}

impl std::error::Error for LexError {}

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

pub fn get_lexemes(input: &str) -> Result<Vec<Lexeme>, LexError> {
    let mut state = State {
        kind: LexerState::Start,
        token: String::new(),
    };
    let mut lexemes: Vec<Lexeme> = Vec::new();
    let mut chars = input.chars().peekable();
    let mut block_depth: u32 = 0;
    let mut cursor = Cursor::new();

    while let Some(c) = take(&mut chars) {
        let pos = cursor.pos();
        let (new_state, emit, reprocess) =
            transition(&mut state, c, &mut chars, &mut block_depth, pos)?;

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
                    return Err(LexError {
                        kind: LexErrorKind::UnexpectedChar(c),
                        pos,
                    });
                }
            }
        }

        cursor.advance(c);
    }

    finish(state, &mut lexemes, cursor.pos())?;
    Ok(lexemes)
}

/// Tracks the current line/column while scanning the input, so that errors
/// can point at a precise location in the source.
struct Cursor {
    line: usize,
    column: usize,
}

impl Cursor {
    fn new() -> Self {
        Cursor { line: 1, column: 1 }
    }

    fn pos(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
        }
    }

    fn advance(&mut self, c: char) {
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
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
    pos: Position,
) -> Result<(LexerState, Option<Lexeme>, bool), LexError> {
    use LexerState::*;

    let err = |kind: LexErrorKind| Err(LexError { kind, pos });

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
            _ => err(LexErrorKind::UnexpectedChar(c)),
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
            _ => err(LexErrorKind::UnexpectedSharpDispatch(c)),
        },

        BlockComment => match c {
            '|' => Ok((BlockCommentBar, None, false)),
            '#' => Ok((BlockCommentHash, None, false)),
            _ => Ok((BlockComment, None, false)),
        },

        BlockCommentBar => match c {
            '#' => {
                *block_depth = block_depth.checked_sub(1).ok_or(LexError {
                    kind: LexErrorKind::BlockCommentDepthUnderflow,
                    pos,
                })?;
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
                let ch = resolve_char_name(&state.token, pos)?;
                Ok((Start, Some(Lexeme::Character(ch)), true))
            }
            _ => {
                let ch = resolve_char_name(&state.token, pos)?;
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
            _ => err(LexErrorKind::InvalidDotDotSequence(c)),
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
                let value = parse_integer(&state.token, pos)?;
                Ok((Start, Some(Lexeme::Integer(value)), true))
            }
            _ => {
                let value = parse_integer(&state.token, pos)?;
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
                let value = parse_float(&state.token, pos)?;
                Ok((Start, Some(Lexeme::Float(value)), true))
            }
            _ => {
                let value = parse_float(&state.token, pos)?;
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
            _ => err(LexErrorKind::ExpectedExponentDigit(c)),
        },

        ExponentSign => match c {
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((ExponentDigits, None, false))
            }
            _ => err(LexErrorKind::ExpectedExponentDigit(c)),
        },

        ExponentDigits => match c {
            c if c.is_ascii_digit() => {
                state.token.push(c);
                Ok((ExponentDigits, None, false))
            }
            c if c.is_whitespace() || is_delimiter(c) => {
                let value = parse_float(&state.token, pos)?;
                Ok((Start, Some(Lexeme::Float(value)), true))
            }
            _ => {
                let value = parse_float(&state.token, pos)?;
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
            _ => err(LexErrorKind::UnknownEscape(c)),
        },
    }
}

fn finish(state: State, lexemes: &mut Vec<Lexeme>, pos: Position) -> Result<(), LexError> {
    use LexerState::*;

    let err = |kind: LexErrorKind| Err(LexError { kind, pos });

    match state.kind {
        Start | LineComment => Ok(()),
        Integer => {
            let value = parse_integer(&state.token, pos)?;
            lexemes.push(Lexeme::Integer(value));
            Ok(())
        }
        Float | ExponentDigits => {
            let value = parse_float(&state.token, pos)?;
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
            let ch = resolve_char_name(&state.token, pos)?;
            lexemes.push(Lexeme::Character(ch));
            Ok(())
        }
        CharSharp => err(LexErrorKind::UnterminatedCharacter),
        Unquote => {
            lexemes.push(Lexeme::Unquote);
            Ok(())
        }
        Sharp => err(LexErrorKind::UnterminatedSharpDispatch),
        BlockComment | BlockCommentBar | BlockCommentHash => {
            err(LexErrorKind::UnterminatedBlockComment)
        }
        InString | Escape => err(LexErrorKind::UnterminatedString),
        Exponent | ExponentSign => err(LexErrorKind::UnterminatedExponent),
    }
}

fn parse_integer(token: &str, pos: Position) -> Result<i64, LexError> {
    token.parse().map_err(|_| LexError {
        kind: LexErrorKind::InvalidInteger(token.to_string()),
        pos,
    })
}

fn parse_float(token: &str, pos: Position) -> Result<f64, LexError> {
    token.parse().map_err(|_| LexError {
        kind: LexErrorKind::InvalidFloat(token.to_string()),
        pos,
    })
}

fn resolve_char_name(name: &str, pos: Position) -> Result<char, LexError> {
    match name {
        "space" => Ok(' '),
        "newline" => Ok('\n'),
        _ if name.len() == 1 => Ok(name.chars().next().unwrap()),
        _ => Err(LexError {
            kind: LexErrorKind::UnknownCharName(name.to_string()),
            pos,
        }),
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
    #[allow(clippy::approx_constant)]
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
        let err = get_lexemes("#\\foobar").unwrap_err();
        assert_eq!(
            err.kind,
            LexErrorKind::UnknownCharName("foobar".to_string())
        );
        assert_eq!(err.pos, Position { line: 1, column: 9 });
    }

    #[test]
    fn unterminated_string_errors() {
        let err = get_lexemes("\"hello").unwrap_err();
        assert_eq!(err.kind, LexErrorKind::UnterminatedString);
        assert_eq!(err.pos, Position { line: 1, column: 7 });
    }

    #[test]
    fn error_reports_line_and_column_across_newlines() {
        let err = get_lexemes("(+ 1 2)\n(foo #\\bogusname)").unwrap_err();
        assert_eq!(
            err.kind,
            LexErrorKind::UnknownCharName("bogusname".to_string())
        );
        assert_eq!(
            err.pos,
            Position {
                line: 2,
                column: 17
            }
        );
    }

    #[test]
    fn unexpected_character_error() {
        let err = get_lexemes("(foo | bar)").unwrap_err();
        assert_eq!(err.kind, LexErrorKind::UnexpectedChar('|'));
        assert_eq!(err.pos, Position { line: 1, column: 6 });
    }

    #[test]
    fn unterminated_block_comment_errors() {
        let err = get_lexemes("#| nested #| comment |# ").unwrap_err();
        assert_eq!(err.kind, LexErrorKind::UnterminatedBlockComment);
    }

    #[test]
    fn invalid_dot_dot_sequence_errors() {
        let err = get_lexemes("..x").unwrap_err();
        assert_eq!(err.kind, LexErrorKind::InvalidDotDotSequence('x'));
    }

    #[test]
    fn unknown_escape_errors() {
        let err = get_lexemes("\"bad \\q escape\"").unwrap_err();
        assert_eq!(err.kind, LexErrorKind::UnknownEscape('q'));
    }

    #[test]
    fn lex_error_display_includes_position() {
        let err = get_lexemes("(foo | bar)").unwrap_err();
        assert_eq!(err.to_string(), "1:6: unexpected character '|'");
    }
}
