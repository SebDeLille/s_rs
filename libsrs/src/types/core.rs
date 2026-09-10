use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io::{BufReader, Read};
use std::rc::Rc;

/// Lexical environment: variable bindings + optional parent scope.
/// Wrapped in Rc<RefCell<_>> so closures can capture and share it.
#[derive(Debug)]
pub struct Env {
    pub bindings: RefCell<HashMap<String, SrsValue>>,
    pub parent: Option<Rc<Env>>,
}

impl Env {
    pub fn new(parent: Option<Rc<Env>>) -> Rc<Env> {
        Rc::new(Env {
            bindings: RefCell::new(HashMap::new()),
            parent,
        })
    }

    pub fn get(&self, name: &str) -> Option<SrsValue> {
        if let Some(v) = self.bindings.borrow().get(name) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.get(name))
    }

    pub fn define(&self, name: String, value: SrsValue) {
        self.bindings.borrow_mut().insert(name, value);
    }

    /// Mutates an existing binding in the nearest enclosing scope.
    /// Returns false if the name is unbound anywhere in the chain.
    pub fn set(&self, name: &str, value: SrsValue) -> bool {
        if self.bindings.borrow().contains_key(name) {
            self.bindings.borrow_mut().insert(name.to_string(), value);
            return true;
        }
        match &self.parent {
            Some(p) => p.set(name, value),
            None => false,
        }
    }
}

/// User-defined closure: parameter list, variadic rest param (if any), body, captured env.
#[derive(Clone)]
pub struct Lambda {
    pub params: Vec<String>,
    pub rest: Option<String>,
    pub body: Vec<SrsValue>,
    pub env: Rc<Env>,
}

impl fmt::Debug for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lambda")
            .field("params", &self.params)
            .field("rest", &self.rest)
            .field("body", &self.body)
            .finish()
    }
}

/// Native (builtin) procedure implemented in Rust.
/// Boxed in an `Rc` so that native procedures can be closures capturing
/// shared state (e.g. a Cairo surface for GTK canvas primitives), not just
/// plain function pointers.
pub type NativeFn = Rc<dyn Fn(&[SrsValue]) -> Result<SrsValue, String>>;

#[derive(Clone)]
pub struct Native {
    pub name: &'static str,
    pub func: NativeFn,
}

impl fmt::Debug for Native {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Native").field("name", &self.name).finish()
    }
}

#[derive(Debug, Clone)]
pub enum SrsValue {
    Integer(i64),
    Float(f64),
    Rational(i64, i64),
    Boolean(bool),
    Character(char),
    /// Mutable string, shared for `string-set!` semantics.
    String(Rc<RefCell<String>>),
    Symbol(String),
    Nil,
    /// Mutable cons cell, shared for `set-car!`/`set-cdr!` and cyclic structures.
    Pair(Rc<RefCell<(SrsValue, SrsValue)>>),
    /// Mutable vector, shared for `vector-set!`.
    Vector(Rc<RefCell<Vec<SrsValue>>>),
    /// Result of `set!`, `define`, etc. — has no printable representation.
    Unspecified,
    /// Result of reading past end of input.
    Eof,
    Procedure(Rc<Lambda>),
    Native(Native),
    /// Delayed computation for `delay`/`force`.
    Promise(Rc<RefCell<PromiseState>>),
    /// I/O port backed by stdin/stdout — mutable because reads/writes advance state.
    Port(Rc<RefCell<PortData>>),
}

impl fmt::Display for SrsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SrsValue::Integer(n) => write!(f, "{}", n),
            SrsValue::Float(x) => write!(f, "{}", x),
            SrsValue::Rational(num, den) => write!(f, "{}/{}", num, den),
            SrsValue::Boolean(true) => write!(f, "#t"),
            SrsValue::Boolean(false) => write!(f, "#f"),
            SrsValue::Character(c) => write!(f, "#\\{}", format_char(*c)),
            SrsValue::String(s) => write!(f, "\"{}\"", s.borrow()),
            SrsValue::Symbol(s) => write!(f, "{}", s),
            SrsValue::Nil => write!(f, "()"),
            SrsValue::Pair(_) => {
                write!(f, "(")?;
                let mut first = true;
                let mut cur = self.clone();
                loop {
                    match cur {
                        SrsValue::Pair(p) => {
                            let (car, cdr) = p.borrow().clone();
                            if !first {
                                write!(f, " ")?;
                            }
                            first = false;
                            write!(f, "{}", car)?;
                            cur = cdr;
                        }
                        SrsValue::Nil => break,
                        other => {
                            write!(f, " . {}", other)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
            SrsValue::Vector(v) => {
                write!(f, "#(")?;
                for (i, item) in v.borrow().iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            SrsValue::Unspecified => Ok(()),
            SrsValue::Eof => write!(f, "#<eof>"),
            SrsValue::Procedure(_) => write!(f, "#<procedure>"),
            SrsValue::Native(n) => write!(f, "#<procedure:{}>", n.name),
            SrsValue::Promise(_) => write!(f, "#<promise>"),
            SrsValue::Port(_) => write!(f, "#<port>"),
        }
    }
}

impl SrsValue {
    /// Formats the value the way R5RS `display` does: strings are printed
    /// without surrounding quotes/escapes and characters are printed as
    /// themselves rather than as a `#\name` literal. Nested values (inside
    /// pairs and vectors) are formatted the same way, recursively. Every
    /// other value looks the same as its [`fmt::Display`] (i.e. `write`)
    /// representation.
    pub fn display_repr(&self) -> String {
        match self {
            SrsValue::Character(c) => c.to_string(),
            SrsValue::String(s) => s.borrow().clone(),
            SrsValue::Pair(_) => {
                let mut out = String::from("(");
                let mut first = true;
                let mut cur = self.clone();
                loop {
                    match cur {
                        SrsValue::Pair(p) => {
                            let (car, cdr) = p.borrow().clone();
                            if !first {
                                out.push(' ');
                            }
                            first = false;
                            out.push_str(&car.display_repr());
                            cur = cdr;
                        }
                        SrsValue::Nil => break,
                        other => {
                            out.push_str(" . ");
                            out.push_str(&other.display_repr());
                            break;
                        }
                    }
                }
                out.push(')');
                out
            }
            SrsValue::Vector(v) => {
                let mut out = String::from("#(");
                for (i, item) in v.borrow().iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    out.push_str(&item.display_repr());
                }
                out.push(')');
                out
            }
            other => other.to_string(),
        }
    }
}

impl PortData {
    /// Builds a port reading from standard input.
    pub fn stdin() -> Self {
        PortData::InputStdin {
            reader: BufReader::new(Box::new(std::io::stdin())),
            peeked: None,
        }
    }

    /// Builds a port writing to standard output.
    pub fn stdout() -> Self {
        PortData::OutputStdout
    }

    /// Builds an input port from a string.
    pub fn input_string(s: impl Into<String>) -> Self {
        PortData::InputString(s.into().chars().collect::<Vec<_>>(), 0)
    }

    /// Builds an output port accumulating into a string.
    pub fn output_string() -> Self {
        PortData::OutputString(String::new())
    }

    /// Builds an input port reading from `path`, or returns an I/O error.
    pub fn input_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        use std::fs::File;
        let file = File::open(path).map_err(|e| format!("open-input-file: {}", e))?;
        Ok(PortData::InputFile {
            reader: BufReader::new(Box::new(file)),
            peeked: None,
        })
    }

    /// Builds an input port from a byte slice, for unit tests.
    #[cfg(test)]
    pub fn test_input(data: &'static [u8]) -> Self {
        use std::io::Cursor;
        PortData::InputStdin {
            reader: BufReader::new(Box::new(Cursor::new(data))),
            peeked: None,
        }
    }

    /// Shared logic for reading/peeking one character from an
    /// [`InputStdin`][PortData::InputStdin] or
    /// [`InputFile`][PortData::InputFile] port.
    fn take_or_read_peeked(
        reader: &mut BufReader<Box<dyn Read>>,
        peeked: &mut Option<char>,
        take: bool,
    ) -> Result<Option<char>, String> {
        if let Some(c) = peeked.take() {
            if !take {
                *peeked = Some(c);
            }
            return Ok(Some(c));
        }
        read_char_from_reader(reader)
    }

    /// Reads and consumes one character from the input port, returning
    /// [`SrsValue::Eof`] at end of file.
    pub fn read_char(&mut self) -> Result<SrsValue, String> {
        match self {
            PortData::InputStdin { reader, peeked } | PortData::InputFile { reader, peeked } => {
                Self::take_or_read_peeked(reader, peeked, true)
                    .map(|opt| opt.map(SrsValue::Character).unwrap_or(SrsValue::Eof))
            }
            PortData::InputString(chars, pos) => {
                if let Some(&c) = chars.get(*pos) {
                    *pos += 1;
                    Ok(SrsValue::Character(c))
                } else {
                    Ok(SrsValue::Eof)
                }
            }
            PortData::OutputStdout | PortData::OutputString(_) => {
                Err("read-char: not an input port".to_string())
            }
        }
    }

    /// Reads one character from the input port without consuming it,
    /// returning [`SrsValue::Eof`] at end of file.
    pub fn peek_char(&mut self) -> Result<SrsValue, String> {
        match self {
            PortData::InputStdin { reader, peeked } | PortData::InputFile { reader, peeked } => {
                let c = Self::take_or_read_peeked(reader, peeked, false)?;
                if peeked.is_none() {
                    *peeked = c;
                }
                Ok(c.map(SrsValue::Character).unwrap_or(SrsValue::Eof))
            }
            PortData::InputString(chars, pos) => {
                if let Some(&c) = chars.get(*pos) {
                    Ok(SrsValue::Character(c))
                } else {
                    Ok(SrsValue::Eof)
                }
            }
            PortData::OutputStdout | PortData::OutputString(_) => {
                Err("peek-char: not an input port".to_string())
            }
        }
    }

    /// Returns `true` if this port can be used for input.
    pub fn is_input_port(&self) -> bool {
        matches!(
            self,
            PortData::InputStdin { .. } | PortData::InputFile { .. } | PortData::InputString(..)
        )
    }

    /// Returns `true` if this port can be used for output.
    pub fn is_output_port(&self) -> bool {
        matches!(self, PortData::OutputStdout | PortData::OutputString(_))
    }

    /// Reads characters from the input port until a newline (excluded) or
    /// end of file, returning the accumulated string. If the first read
    /// yields EOF, returns [`SrsValue::Eof`].
    pub fn read_line(&mut self) -> Result<SrsValue, String> {
        match self {
            PortData::InputStdin { reader, peeked } | PortData::InputFile { reader, peeked } => {
                let mut buf = String::new();
                loop {
                    let c = Self::take_or_read_peeked(reader, peeked, true);
                    match c? {
                        Some('\n') => return Ok(SrsValue::String(Rc::new(RefCell::new(buf)))),
                        Some(ch) => buf.push(ch),
                        None => {
                            if buf.is_empty() {
                                return Ok(SrsValue::Eof);
                            }
                            return Ok(SrsValue::String(Rc::new(RefCell::new(buf))));
                        }
                    }
                }
            }
            PortData::InputString(chars, pos) => {
                let mut buf = String::new();
                while let Some(&c) = chars.get(*pos) {
                    *pos += 1;
                    if c == '\n' {
                        return Ok(SrsValue::String(Rc::new(RefCell::new(buf))));
                    }
                    buf.push(c);
                }
                if buf.is_empty() {
                    Ok(SrsValue::Eof)
                } else {
                    Ok(SrsValue::String(Rc::new(RefCell::new(buf))))
                }
            }
            PortData::OutputStdout | PortData::OutputString(_) => {
                Err("read-line: not an input port".to_string())
            }
        }
    }

    /// Writes a character to the output port.
    pub fn write_char(&mut self, c: char) -> Result<(), String> {
        match self {
            PortData::OutputString(buf) => {
                buf.push(c);
                Ok(())
            }
            PortData::OutputStdout => {
                use std::io::Write;
                print!("{}", c);
                std::io::stdout()
                    .flush()
                    .map_err(|e| format!("write-char: {}", e))
            }
            _ => Err("write-char: not an output port".to_string()),
        }
    }

    /// Writes a string to the output port, optionally bounded by `start` and
    /// `end` character offsets.
    pub fn write_string(
        &mut self,
        s: &str,
        start: Option<usize>,
        end: Option<usize>,
    ) -> Result<(), String> {
        let start = start.unwrap_or(0);
        let end = end.unwrap_or(s.chars().count());
        if start > end {
            return Err("write-string: start greater than end".to_string());
        }
        let slice: String = s.chars().skip(start).take(end - start).collect();
        match self {
            PortData::OutputString(buf) => {
                buf.push_str(&slice);
                Ok(())
            }
            PortData::OutputStdout => {
                use std::io::Write;
                print!("{}", slice);
                std::io::stdout()
                    .flush()
                    .map_err(|e| format!("write-string: {}", e))
            }
            _ => Err("write-string: not an output port".to_string()),
        }
    }

    /// Returns the contents accumulated in an output-string port.
    pub fn get_output_string(&self) -> Result<String, String> {
        match self {
            PortData::OutputString(buf) => Ok(buf.clone()),
            _ => Err("get-output-string: not an output-string port".to_string()),
        }
    }
}

fn read_char_from_reader<R: Read>(reader: &mut R) -> Result<Option<char>, String> {
    use std::io::ErrorKind;

    let mut first = [0u8; 1];
    if let Err(e) = reader.read_exact(&mut first) {
        return if e.kind() == ErrorKind::UnexpectedEof {
            Ok(None)
        } else {
            Err(format!("read-char: {}", e))
        };
    }

    let len = utf8_char_len(first[0]);
    let mut buf = [0u8; 4];
    buf[0] = first[0];
    if len > 1 {
        reader
            .read_exact(&mut buf[1..len])
            .map_err(|e| format!("read-char: {}", e))?;
    }

    std::str::from_utf8(&buf[..len])
        .map_err(|_| "read-char: invalid UTF-8".to_string())
        .map(|s| s.chars().next())
}

fn utf8_char_len(first_byte: u8) -> usize {
    if first_byte & 0b1000_0000 == 0 {
        1
    } else if first_byte & 0b1110_0000 == 0b1100_0000 {
        2
    } else if first_byte & 0b1111_0000 == 0b1110_0000 {
        3
    } else if first_byte & 0b1111_1000 == 0b1111_0000 {
        4
    } else {
        1
    }
}

fn format_char(c: char) -> String {
    match c {
        ' ' => "space".to_string(),
        '\n' => "newline".to_string(),
        '\t' => "tab".to_string(),
        '\r' => "return".to_string(),
        '\0' => "null".to_string(),
        c => c.to_string(),
    }
}

#[derive(Debug, Clone)]
pub enum PromiseState {
    Forced(SrsValue),
    Delayed(Vec<SrsValue>, Rc<Env>),
}

/// Runtime data behind an [`SrsValue::Port`].
pub enum PortData {
    /// Standard input backed by an opaque reader.
    InputStdin {
        reader: BufReader<Box<dyn Read>>,
        peeked: Option<char>,
    },
    /// File input backed by an opaque reader.
    InputFile {
        reader: BufReader<Box<dyn Read>>,
        peeked: Option<char>,
    },
    /// Standard output.
    OutputStdout,
    /// Input port reading characters from a string using a byte index.
    InputString(Vec<char>, usize),
    /// Output port accumulating characters into a string.
    OutputString(String),
}

impl fmt::Debug for PortData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortData::InputStdin { peeked, .. } => f
                .debug_struct("InputStdin")
                .field("peeked", peeked)
                .finish(),
            PortData::InputFile { peeked, .. } => {
                f.debug_struct("InputFile").field("peeked", peeked).finish()
            }
            PortData::OutputStdout => write!(f, "OutputStdout"),
            PortData::InputString(chars, pos) => f
                .debug_tuple("InputString")
                .field(&chars.iter().collect::<String>())
                .field(pos)
                .finish(),
            PortData::OutputString(buf) => f.debug_tuple("OutputString").field(buf).finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn display_repr_of_string_has_no_quotes() {
        let value = SrsValue::String(Rc::new(RefCell::new("hi".to_string())));
        assert_eq!(value.display_repr(), "hi");
        assert_eq!(value.to_string(), "\"hi\"");
    }

    #[test]
    fn display_repr_of_character_is_the_bare_char() {
        let value = SrsValue::Character('a');
        assert_eq!(value.display_repr(), "a");
        assert_eq!(value.to_string(), "#\\a");
    }

    #[test]
    fn display_repr_of_integer_matches_display_trait() {
        let value = SrsValue::Integer(42);
        assert_eq!(value.display_repr(), "42");
    }

    #[test]
    fn display_repr_recurses_into_pairs() {
        let value = SrsValue::Pair(Rc::new(RefCell::new((
            SrsValue::String(Rc::new(RefCell::new("hi".to_string()))),
            SrsValue::Nil,
        ))));
        assert_eq!(value.display_repr(), "(hi)");
        assert_eq!(value.to_string(), "(\"hi\")");
    }

    #[test]
    fn display_repr_recurses_into_vectors() {
        let value = SrsValue::Vector(Rc::new(RefCell::new(vec![
            SrsValue::Character('x'),
            SrsValue::Integer(1),
        ])));
        assert_eq!(value.display_repr(), "#(x 1)");
    }

    #[test]
    fn port_displays_as_tagged_object() {
        let value = SrsValue::Port(Rc::new(RefCell::new(PortData::stdout())));
        assert_eq!(value.to_string(), "#<port>");
        assert_eq!(value.display_repr(), "#<port>");
    }

    #[test]
    fn utf8_char_len_detects_sequence_lengths() {
        assert_eq!(utf8_char_len(b'a'), 1);
        assert_eq!(utf8_char_len(0xc2), 2);
        assert_eq!(utf8_char_len(0xe2), 3);
        assert_eq!(utf8_char_len(0xf0), 4);
    }

    #[test]
    fn read_char_from_reader_reads_ascii_and_multibyte_chars() {
        let mut cursor = Cursor::new("aé€");
        assert_eq!(read_char_from_reader(&mut cursor).unwrap(), Some('a'));
        assert_eq!(read_char_from_reader(&mut cursor).unwrap(), Some('é'));
        assert_eq!(read_char_from_reader(&mut cursor).unwrap(), Some('€'));
    }

    #[test]
    fn read_char_from_reader_returns_eof_on_empty_input() {
        let mut cursor = Cursor::new("");
        assert_eq!(read_char_from_reader(&mut cursor).unwrap(), None);
    }

    #[test]
    fn read_char_from_reader_returns_error_on_invalid_utf8() {
        let mut cursor = Cursor::new(vec![0xff]);
        assert!(read_char_from_reader(&mut cursor).is_err());
    }

    #[test]
    fn output_port_read_char_errors() {
        let mut port = PortData::stdout();
        assert!(port.read_char().is_err());
        assert!(port.peek_char().is_err());
    }

    #[test]
    fn read_line_returns_strings_until_eof() {
        let mut port = PortData::test_input(b"abc\ndef");
        assert!(
            matches!(port.read_line().unwrap(), SrsValue::String(s) if s.borrow().as_str() == "abc")
        );
        assert!(
            matches!(port.read_line().unwrap(), SrsValue::String(s) if s.borrow().as_str() == "def")
        );
        assert!(matches!(port.read_line().unwrap(), SrsValue::Eof));
    }

    #[test]
    fn read_line_returns_eof_on_empty_input() {
        let mut port = PortData::test_input(b"");
        assert!(matches!(port.read_line().unwrap(), SrsValue::Eof));
    }

    #[test]
    fn output_port_read_line_errors() {
        let mut port = PortData::stdout();
        assert!(port.read_line().is_err());
    }

    #[test]
    fn input_string_port_reads_chars_and_eof() {
        let mut port = PortData::input_string("ab");
        assert!(matches!(
            port.read_char().unwrap(),
            SrsValue::Character('a')
        ));
        assert!(matches!(
            port.peek_char().unwrap(),
            SrsValue::Character('b')
        ));
        assert!(matches!(
            port.peek_char().unwrap(),
            SrsValue::Character('b')
        ));
        assert!(matches!(
            port.read_char().unwrap(),
            SrsValue::Character('b')
        ));
        assert!(matches!(port.read_char().unwrap(), SrsValue::Eof));
    }

    #[test]
    fn input_string_port_read_line_with_newlines() {
        let mut port = PortData::input_string("abc\ndef");
        let first = port.read_line().unwrap();
        assert!(matches!(first, SrsValue::String(s) if s.borrow().as_str() == "abc"));
        let second = port.read_line().unwrap();
        assert!(matches!(second, SrsValue::String(s) if s.borrow().as_str() == "def"));
        assert!(matches!(port.read_line().unwrap(), SrsValue::Eof));
    }

    #[test]
    fn input_string_port_read_line_eof_on_empty() {
        let mut port = PortData::input_string("");
        assert!(matches!(port.read_line().unwrap(), SrsValue::Eof));
    }

    #[test]
    fn output_string_port_accumulates_chars() {
        let mut port = PortData::output_string();
        port.write_char('x').unwrap();
        port.write_char('y').unwrap();
        assert_eq!(port.get_output_string().unwrap(), "xy");
    }

    #[test]
    fn output_string_port_accumulates_substring() {
        let mut port = PortData::output_string();
        port.write_string("hello", Some(1), Some(4)).unwrap();
        assert_eq!(port.get_output_string().unwrap(), "ell");
    }

    #[test]
    fn output_string_port_write_string_without_bounds_writes_all() {
        let mut port = PortData::output_string();
        port.write_string("hi", None, None).unwrap();
        assert_eq!(port.get_output_string().unwrap(), "hi");
    }

    #[test]
    fn get_output_string_rejects_non_output_string_port() {
        let port = PortData::stdout();
        assert!(port.get_output_string().is_err());
        let port = PortData::input_string("x");
        assert!(port.get_output_string().is_err());
    }
}
