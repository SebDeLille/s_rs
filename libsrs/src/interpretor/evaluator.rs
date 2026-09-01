use crate::types::core::SrsValue;
use crate::types::error::{SrsError, SrsErrorKind, SrsResult};
use crate::types::memory::SrsMemory;

pub struct Evaluator<'a> {
    memory: SrsMemory<'a>,
}

impl<'a> Evaluator<'a> {
    pub fn new() -> Self {
        let mut memory = SrsMemory::new();
        Self::register_primitives(&mut memory);
        Evaluator { memory }
    }

    fn register_primitives(memory: &mut SrsMemory) {
        memory.add("+", SrsValue::Id("__add".to_string()));
        memory.add("-", SrsValue::Id("__sub".to_string()));
        memory.add("*", SrsValue::Id("__mul".to_string()));
        memory.add("/", SrsValue::Id("__div".to_string()));
    }

    pub fn eval(&self, value: &SrsValue) -> SrsResult<SrsValue> {
        match value {
            SrsValue::List(list) => self.eval_list(list),
            SrsValue::Id(name) => self.eval_id(name),
            literal @ (SrsValue::Integer(_) | SrsValue::Float(_) | SrsValue::Bool(_) | SrsValue::String(_)) => Ok(literal.clone()),
            SrsValue::Nil => Ok(SrsValue::Nil),
            SrsValue::Vector(_) => Err(SrsError::new(SrsErrorKind::TypeMismatch)),
        }
    }

    fn eval_list(&self, list: &[SrsValue]) -> SrsResult<SrsValue> {
        if list.is_empty() {
            return Ok(SrsValue::Nil);
        }

        if let SrsValue::Id(op) = &list[0] {
            let op = self.resolve(op);
            let args = list.get(1..).unwrap_or(&[]);
            return self.apply(&op, args);
        }

        Err(SrsError::new(SrsErrorKind::UnknownType))
    }

    fn eval_id(&self, name: &str) -> SrsResult<SrsValue> {
        self.memory
            .get(name)
            .cloned()
            .or_else(|| self.resolve_primitive(name))
            .ok_or_else(|| {
                SrsError::with_message(
                    SrsErrorKind::UnknownIdentifier,
                    format!("unknown identifier: {}", name),
                )
            })
    }

    fn resolve_primitive(&self, name: &str) -> Option<SrsValue> {
        match name {
            "+" | "-" | "*" | "/" => Some(SrsValue::Id(format!("__{}", name))),
            _ => None,
        }
    }

    fn resolve(&self, name: &str) -> SrsValue {
        self.memory
            .get(name)
            .cloned()
            .unwrap_or_else(|| SrsValue::Id(name.to_string()))
    }

    fn apply(&self, op: &SrsValue, raw_args: &[SrsValue]) -> SrsResult<SrsValue> {
        let op_name = match op {
            SrsValue::Id(n) => n.as_str(),
            _ => return Err(SrsError::new(SrsErrorKind::TypeMismatch)),
        };

        let mut args = Vec::with_capacity(raw_args.len());
        for a in raw_args {
            args.push(self.eval(a)?);
        }

        match op_name {
            "__add" => Self::fold_numbers(&args, |a, b| a + b, |a, b| a + b),
            "__sub" => Self::fold_numbers(&args, |a, b| a - b, |a, b| a - b),
            "__mul" => Self::fold_numbers(&args, |a, b| a * b, |a, b| a * b),
            "__div" => Self::fold_numbers(&args, |a, b| a / b, |a, b| a / b),
            _ => Err(SrsError::with_message(SrsErrorKind::UnknownIdentifier, format!("unknown operator: {}", op_name))),
        }
    }

    fn fold_numbers(
        args: &[SrsValue],
        iop: fn(i64, i64) -> i64,
        fop: fn(f64, f64) -> f64,
    ) -> SrsResult<SrsValue> {
        if args.is_empty() {
            return Err(SrsError::new(SrsErrorKind::NotEnoughArguments));
        }

        let mut result = args[0].clone();
        for value in &args[1..] {
            result = match (&result, value) {
                (SrsValue::Integer(a), SrsValue::Integer(b)) => SrsValue::Integer(iop(*a, *b)),
                (SrsValue::Integer(a), SrsValue::Float(b)) => SrsValue::Float(fop(*a as f64, *b)),
                (SrsValue::Float(a), SrsValue::Integer(b)) => SrsValue::Float(fop(*a, *b as f64)),
                (SrsValue::Float(a), SrsValue::Float(b)) => SrsValue::Float(fop(*a, *b)),
                _ => return Err(SrsError::new(SrsErrorKind::TypeMismatch)),
            };
        }

        Ok(result)
    }
}

impl Default for Evaluator<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretor::lexical_analyzer::get_lexemes;
    use crate::interpretor::translator::translate_all;

    fn eval(scm: &str) -> SrsValue {
        let values = translate_all(get_lexemes(scm).unwrap()).unwrap();
        let evaluator = Evaluator::new();
        evaluator.eval(&values[0]).unwrap()
    }

    #[test]
    fn basic_integer() {
        assert_eq!(SrsValue::Integer(2), eval("2"));
    }

    #[test]
    fn add_integers() {
        assert_eq!(SrsValue::Integer(5), eval("(+ 2 3)"));
    }

    #[test]
    fn nested_expression() {
        assert_eq!(SrsValue::Integer(10), eval("(+ (* 2 3) 4)"));
    }

    #[test]
    fn float_coercion() {
        let result = eval("(+ 1 2.5)");
        if let SrsValue::Float(f) = result {
            assert!((f - 3.5).abs() < f64::EPSILON);
        } else {
            panic!("expected float");
        }
    }

    #[test]
    fn missing_args_fails() {
        let evaluator = Evaluator::new();
        let values = translate_all(get_lexemes("(+ )").unwrap()).unwrap();
        assert!(evaluator.eval(&values[0]).is_err());
    }

    #[test]
    fn unknown_operator_fails() {
        let evaluator = Evaluator::new();
        let values = translate_all(get_lexemes("(foo 1)").unwrap()).unwrap();
        assert!(evaluator.eval(&values[0]).is_err());
    }
}
