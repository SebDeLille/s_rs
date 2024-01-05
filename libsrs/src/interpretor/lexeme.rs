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

pub struct Lexeme {
    lexeme_type: LexemeType,
    value: String,
}

impl Lexeme {
    pub fn new(t: LexemeType, v: String) -> Self {
        Lexeme {
            lexeme_type: t,
            value: v
        }
    }
}