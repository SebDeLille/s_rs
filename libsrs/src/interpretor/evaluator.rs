use crate::types::core::SrsValue;
use crate::types::env::{Env, EnvRef};
use crate::types::error::{SrsError, SrsErrorKind, SrsResult};

pub struct Evaluator {
    env: EnvRef,
}

impl Evaluator {
    pub fn new() -> Self {
        let env = Env::root();
        Self::register_primitives(&env);
        Evaluator { env }
    }

    /// Single registry for primitive operators. Add a new operator here and
    /// in `Evaluator::apply`; no other mapping is needed.
    fn register_primitives(env: &EnvRef) {
        env.define("+", SrsValue::Id("__add".to_string()));
        env.define("-", SrsValue::Id("__sub".to_string()));
        env.define("*", SrsValue::Id("__mul".to_string()));
        env.define("/", SrsValue::Id("__div".to_string()));
        env.define("exit", SrsValue::Id("__exit".to_string()));
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
        self.env.get(name).ok_or_else(|| {
            SrsError::with_message(
                SrsErrorKind::UnknownIdentifier,
                format!("unknown identifier: {}", name),
            )
        })
    }

    fn resolve(&self, name: &str) -> SrsValue {
        self.env
            .get(name)
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
            "__add" => Self::add(&args),
            "__sub" => Self::sub(&args),
            "__mul" => Self::mul(&args),
            "__div" => Self::div(&args),
            "__exit" => Self::exit(&args),
            _ => Err(SrsError::with_message(SrsErrorKind::UnknownIdentifier, format!("unknown operator: {}", op_name))),
        }
    }

    fn add(args: &[SrsValue]) -> SrsResult<SrsValue> {
        if args.is_empty() {
            return Ok(SrsValue::Integer(0));
        }
        Self::fold_numbers(args, |a, b| a + b, |a, b| a + b)
    }

    fn sub(args: &[SrsValue]) -> SrsResult<SrsValue> {
        if args.is_empty() {
            return Err(SrsError::new(SrsErrorKind::NotEnoughArguments));
        }
        Self::fold_numbers(args, |a, b| a - b, |a, b| a - b)
    }

    fn mul(args: &[SrsValue]) -> SrsResult<SrsValue> {
        if args.is_empty() {
            return Ok(SrsValue::Integer(1));
        }
        Self::fold_numbers(args, |a, b| a * b, |a, b| a * b)
    }

    fn div(args: &[SrsValue]) -> SrsResult<SrsValue> {
        if args.is_empty() {
            return Err(SrsError::new(SrsErrorKind::NotEnoughArguments));
        }
        for d in &args[1..] {
            if Self::is_numeric_zero(d) {
                return Err(SrsError::with_message(
                    SrsErrorKind::TypeMismatch,
                    "division by zero",
                ));
            }
        }
        Self::fold_numbers(args, |a, b| a / b, |a, b| a / b)
    }

    fn exit(args: &[SrsValue]) -> SrsResult<SrsValue> {
        match args {
            [] => Err(SrsError::Exit(0)),
            [SrsValue::Integer(code)] => Err(SrsError::Exit(*code)),
            [_] => Err(SrsError::new(SrsErrorKind::TypeMismatch)),
            [_, _, ..] => Err(SrsError::new(SrsErrorKind::TooManyArguments)),
        }
    }

    fn is_numeric_zero(value: &SrsValue) -> bool {
        match value {
            SrsValue::Integer(0) => true,
            SrsValue::Float(x) if *x == 0.0 => true,
            _ => false,
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

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretor::lexical_analyzer::get_lexemes;
    use crate::interpretor::translator::translate_all;
    use crate::types::error::SrsError;

    fn eval(scm: &str) -> SrsValue {
        let values = translate_all(get_lexemes(scm).unwrap()).unwrap();
        let evaluator = Evaluator::new();
        evaluator.eval(&values[0]).unwrap()
    }

    fn eval_err(scm: &str) -> SrsError {
        let values = translate_all(get_lexemes(scm).unwrap()).unwrap();
        let evaluator = Evaluator::new();
        evaluator.eval(&values[0]).unwrap_err()
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
    fn missing_args_fails_for_sub() {
        let evaluator = Evaluator::new();
        let values = translate_all(get_lexemes("(- )").unwrap()).unwrap();
        assert!(evaluator.eval(&values[0]).is_err());
    }

    #[test]
    fn unknown_operator_fails() {
        let evaluator = Evaluator::new();
        let values = translate_all(get_lexemes("(foo 1)").unwrap()).unwrap();
        assert!(evaluator.eval(&values[0]).is_err());
    }

    #[test]
    fn exit_no_arg_returns_code_zero() {
        assert_eq!(SrsError::Exit(0), eval_err("(exit)"));
    }

    #[test]
    fn exit_with_integer_returns_code() {
        assert_eq!(SrsError::Exit(5), eval_err("(exit 5)"));
    }

    #[test]
    fn exit_with_too_many_args_fails() {
        assert_eq!(SrsErrorKind::TooManyArguments, eval_err("(exit 1 2)").kind().unwrap());
    }

    #[test]
    fn exit_with_non_integer_fails() {
        assert_eq!(SrsErrorKind::TypeMismatch, eval_err("(exit #t)").kind().unwrap());
    }
}
