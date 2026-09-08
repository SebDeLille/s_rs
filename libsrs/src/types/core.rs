#[derive(Debug, Clone)]
pub enum SrsValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    String(String),
    Symbol(String),
    Nil,
    Pair(Box<SrsValue>, Box<SrsValue>),
    Vector(Vec<SrsValue>),
}
