#[derive(Clone, Debug, PartialEq)]
pub enum LexemeType {
    ERROR,
    LPAR,
    RPAR,
    ID,
    INTEGER,
    FLOAT,
    STRING,
    CHAR,
    NOT,
    EQ,
    GT,
    GTE,
    LT,
    LTE,
    ADD,
    SUB,
    MUL,
    DIV,
    TRUE,
    FALSE,
    QUOTE,
    SHARP
}

#[derive(Clone)]
pub struct Lexeme {
    pub lexeme_type: LexemeType,
    pub value: String,
}

impl Lexeme {
    pub fn new(t: LexemeType, v: String) -> Self {
        Lexeme {
            lexeme_type: t,
            value: v
        }
    }
}