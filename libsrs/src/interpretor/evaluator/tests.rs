use super::*;
use crate::interpretor::lexical_analyzer::get_lexemes;
use crate::interpretor::reader::read_all;

fn eval_src(src: &str) -> Result<SrsValue, EvalError> {
    let values = read_all(get_lexemes(src).unwrap()).unwrap();
    let env = global_env();
    let mut result = Ok(SrsValue::Unspecified);
    for value in &values {
        result = eval(value, &env);
        if let Err(ref err) = result {
            return Err(err.clone());
        }
    }
    result
}

fn ok(src: &str) -> SrsValue {
    eval_src(src).unwrap()
}

#[test]
fn integer_is_self_evaluating() {
    assert!(matches!(ok("42"), SrsValue::Integer(42)));
}

#[test]
fn add_two_integers() {
    assert!(matches!(ok("(+ 3 4)"), SrsValue::Integer(7)));
}

#[test]
fn add_single_integer() {
    assert!(matches!(ok("(+ 3)"), SrsValue::Integer(3)));
}

#[test]
fn add_no_args_returns_identity() {
    assert!(matches!(ok("(+)"), SrsValue::Integer(0)));
}

#[test]
fn sub_two_integers() {
    assert!(matches!(ok("(- 5 2)"), SrsValue::Integer(3)));
}

#[test]
fn sub_single_integer_negates() {
    assert!(matches!(ok("(- 5)"), SrsValue::Integer(-5)));
}

#[test]
fn sub_no_args_fails() {
    assert!(eval_src("(-)").is_err());
}

#[test]
fn multiply_several_integers() {
    assert!(matches!(ok("(* 2 3 4)"), SrsValue::Integer(24)));
}

#[test]
fn multiply_single_integer() {
    assert!(matches!(ok("(* 4)"), SrsValue::Integer(4)));
}

#[test]
fn multiply_no_args_returns_identity() {
    assert!(matches!(ok("(*)"), SrsValue::Integer(1)));
}

#[test]
fn divide_two_integers() {
    assert!(matches!(ok("(/ 10 2)"), SrsValue::Integer(5)));
}

#[test]
fn divide_by_zero_fails() {
    assert!(eval_src("(/ 1 0)").is_err());
}

#[test]
fn nested_expression() {
    assert!(matches!(ok("(+ (* 2 3) 4)"), SrsValue::Integer(10)));
}

#[test]
fn float_coercion() {
    match ok("(+ 1 2.5)") {
        SrsValue::Float(f) => assert!((f - 3.5).abs() < f64::EPSILON),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn max_of_several_integers() {
    assert!(matches!(ok("(max 1 3 2)"), SrsValue::Integer(3)));
}

#[test]
fn max_single_argument() {
    assert!(matches!(ok("(max 5)"), SrsValue::Integer(5)));
}

#[test]
fn max_no_args_fails() {
    assert!(eval_src("(max)").is_err());
}

#[test]
fn max_inexact_contagion() {
    match ok("(max 1 2.0)") {
        SrsValue::Float(f) => assert!((f - 2.0).abs() < f64::EPSILON),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn sin_of_zero() {
    match ok("(sin 0)") {
        SrsValue::Float(f) => assert!((f - 0.0).abs() < f64::EPSILON),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn cos_of_zero() {
    match ok("(cos 0)") {
        SrsValue::Float(f) => assert!((f - 1.0).abs() < f64::EPSILON),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn tan_of_zero() {
    match ok("(tan 0)") {
        SrsValue::Float(f) => assert!((f - 0.0).abs() < f64::EPSILON),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn atan_of_zero() {
    match ok("(atan 0)") {
        SrsValue::Float(f) => assert!((f - 0.0).abs() < f64::EPSILON),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn sin_of_one_matches_std_computation() {
    match ok("(sin 1)") {
        SrsValue::Float(f) => assert!((f - 1f64.sin()).abs() < 1e-9),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn trig_accepts_float_argument() {
    match ok("(cos 0.0)") {
        SrsValue::Float(f) => assert!((f - 1.0).abs() < f64::EPSILON),
        other => panic!("expected float, got {:?}", other),
    }
}

#[test]
fn sin_no_args_fails() {
    assert!(eval_src("(sin)").is_err());
}

#[test]
fn sin_too_many_args_fails() {
    assert!(eval_src("(sin 1 2)").is_err());
}

#[test]
fn sin_wrong_type_fails() {
    assert!(eval_src("(sin \"a\")").is_err());
}

#[test]
fn num_eq_true() {
    assert!(matches!(ok("(= 1 1 1)"), SrsValue::Boolean(true)));
}

#[test]
fn num_eq_false() {
    assert!(matches!(ok("(= 1 1 2)"), SrsValue::Boolean(false)));
}

#[test]
fn num_eq_int_float_mix() {
    assert!(matches!(ok("(= 1 1.0)"), SrsValue::Boolean(true)));
}

#[test]
fn lt_increasing() {
    assert!(matches!(ok("(< 1 2 3)"), SrsValue::Boolean(true)));
}

#[test]
fn lt_not_increasing() {
    assert!(matches!(ok("(< 1 3 2)"), SrsValue::Boolean(false)));
}

#[test]
fn gt_decreasing() {
    assert!(matches!(ok("(> 3 2 1)"), SrsValue::Boolean(true)));
}

#[test]
fn gt_not_decreasing() {
    assert!(matches!(ok("(> 3 1 2)"), SrsValue::Boolean(false)));
}

#[test]
fn le_allows_equal() {
    assert!(matches!(ok("(<= 1 1 2)"), SrsValue::Boolean(true)));
}

#[test]
fn le_rejects_decrease() {
    assert!(matches!(ok("(<= 1 2 1)"), SrsValue::Boolean(false)));
}

#[test]
fn ge_allows_equal() {
    assert!(matches!(ok("(>= 2 2 1)"), SrsValue::Boolean(true)));
}

#[test]
fn ge_rejects_increase() {
    assert!(matches!(ok("(>= 2 1 2)"), SrsValue::Boolean(false)));
}

#[test]
fn compare_single_arg_is_true() {
    assert!(matches!(ok("(< 1)"), SrsValue::Boolean(true)));
    assert!(matches!(ok("(= 1)"), SrsValue::Boolean(true)));
}

#[test]
fn compare_no_args_fails() {
    assert!(eval_src("(=)").is_err());
    assert!(eval_src("(<)").is_err());
}

#[test]
fn compare_wrong_type_fails() {
    assert!(eval_src("(< 1 \"a\")").is_err());
}

#[test]
fn not_false_is_true() {
    assert!(matches!(ok("(not #f)"), SrsValue::Boolean(true)));
}

#[test]
fn not_true_is_false() {
    assert!(matches!(ok("(not #t)"), SrsValue::Boolean(false)));
}

#[test]
fn not_non_boolean_is_false() {
    assert!(matches!(ok("(not 0)"), SrsValue::Boolean(false)));
    assert!(matches!(ok("(not \"a\")"), SrsValue::Boolean(false)));
}

#[test]
fn not_no_args_fails() {
    assert!(eval_src("(not)").is_err());
}

#[test]
fn not_too_many_args_fails() {
    assert!(eval_src("(not #t #f)").is_err());
}

#[test]
fn define_returns_unspecified() {
    assert!(matches!(ok("(define x 42)"), SrsValue::Unspecified));
}

#[test]
fn define_then_reread() {
    assert!(matches!(ok("(define x 42) x"), SrsValue::Integer(42)));
}

#[test]
fn define_with_expression_initializer() {
    assert!(matches!(ok("(define x (+ 2 3)) x"), SrsValue::Integer(5)));
}

#[test]
fn redefine_replaces_value() {
    assert!(matches!(
        ok("(define x 1) (define x 2) x"),
        SrsValue::Integer(2)
    ));
}

#[test]
fn define_undefined_initializer_fails() {
    assert!(eval_src("(define x unknown)").is_err());
}

#[test]
fn define_non_symbol_name_fails() {
    assert!(eval_src("(define 1 2)").is_err());
}

#[test]
fn define_missing_initializer_fails() {
    assert!(eval_src("(define x)").is_err());
}

#[test]
fn define_too_many_args_fails() {
    assert!(eval_src("(define x 1 2)").is_err());
}

#[test]
fn unbound_variable_fails() {
    assert!(eval_src("unknown").is_err());
}

#[test]
fn calling_a_non_procedure_fails() {
    assert!(eval_src("(1 2 3)").is_err());
}

#[test]
fn lambda_produces_a_procedure() {
    assert!(matches!(ok("(lambda (x) x)"), SrsValue::Procedure(_)));
}

#[test]
fn calling_a_lambda_applies_it() {
    assert!(matches!(
        ok("((lambda (x) (+ x 1)) 41)"),
        SrsValue::Integer(42)
    ));
}

#[test]
fn define_then_call_lambda() {
    assert!(matches!(
        ok("(define square (lambda (x) (* x x))) (square 5)"),
        SrsValue::Integer(25)
    ));
}

#[test]
fn lambda_multiple_params() {
    assert!(matches!(
        ok("((lambda (a b c) (+ a b c)) 1 2 3)"),
        SrsValue::Integer(6)
    ));
}

#[test]
fn lambda_body_evaluates_in_sequence() {
    assert!(matches!(
        ok("(define x 0) ((lambda () (define x 1) (define x 2) x))"),
        SrsValue::Integer(2)
    ));
}

#[test]
fn lambda_closes_over_environment() {
    assert!(matches!(
        ok("(define make-adder (lambda (n) (lambda (x) (+ x n)))) \
            (define add5 (make-adder 5)) (add5 10)"),
        SrsValue::Integer(15)
    ));
}

#[test]
fn lambda_variadic_all_args() {
    assert!(matches!(
        ok("((lambda args args) 1 2 3)"),
        SrsValue::Pair(_)
    ));
}

#[test]
fn lambda_dotted_rest_param() {
    assert!(matches!(
        ok("((lambda (a . rest) rest) 1 2 3)"),
        SrsValue::Pair(_)
    ));
}

#[test]
fn lambda_dotted_rest_param_empty() {
    assert!(matches!(ok("((lambda (a . rest) rest) 1)"), SrsValue::Nil));
}

#[test]
fn lambda_no_params() {
    assert!(matches!(ok("((lambda () 42))"), SrsValue::Integer(42)));
}

#[test]
fn lambda_too_few_args_fails() {
    assert!(eval_src("((lambda (a b) a) 1)").is_err());
}

#[test]
fn lambda_too_many_args_fails() {
    assert!(eval_src("((lambda (a) a) 1 2)").is_err());
}

#[test]
fn lambda_missing_body_fails() {
    assert!(eval_src("(lambda (x))").is_err());
}

#[test]
fn lambda_non_symbol_param_fails() {
    assert!(eval_src("(lambda (1) 1)").is_err());
}

#[test]
fn if_true_branch() {
    assert!(matches!(ok("(if #t 1 2)"), SrsValue::Integer(1)));
}

#[test]
fn if_false_branch() {
    assert!(matches!(ok("(if #f 1 2)"), SrsValue::Integer(2)));
}

#[test]
fn if_without_alt_when_true() {
    assert!(matches!(ok("(if #t 42)"), SrsValue::Integer(42)));
}

#[test]
fn if_without_alt_when_false_is_unspecified() {
    assert!(matches!(ok("(if #f 42)"), SrsValue::Unspecified));
}

#[test]
fn if_treats_non_boolean_as_true() {
    assert!(matches!(ok("(if 0 1 2)"), SrsValue::Integer(1)));
}

#[test]
fn if_only_evaluates_taken_branch() {
    assert!(matches!(ok("(if #t 1 (/ 1 0))"), SrsValue::Integer(1)));
    assert!(matches!(ok("(if #f (/ 1 0) 2)"), SrsValue::Integer(2)));
}

#[test]
fn if_too_few_args_fails() {
    assert!(eval_src("(if #t)").is_err());
}

#[test]
fn if_too_many_args_fails() {
    assert!(eval_src("(if #t 1 2 3)").is_err());
}

#[test]
fn do_sums_from_zero_to_nine() {
    let src = "(do ((i 0 (+ i 1))
                     (sum 0 (+ sum i)))
                    ((= i 10) sum))";
    assert!(matches!(ok(src), SrsValue::Integer(45)));
}

#[test]
fn do_without_step_keeps_value() {
    let src = "(do ((x 1) (i 0 (+ i 1)))
                    ((= i 3) x))";
    assert!(matches!(ok(src), SrsValue::Integer(1)));
}

#[test]
fn do_steps_are_evaluated_simultaneously() {
    // `last`'s step reads the *previous* value of `i`, proving steps are
    // all evaluated before any variable is rebound.
    let src = "(do ((i 0 (+ i 1))
                     (last -1 i))
                    ((= i 3) last))";
    assert!(matches!(ok(src), SrsValue::Integer(2)));
}

#[test]
fn do_with_no_result_exprs_is_unspecified() {
    assert!(matches!(
        ok("(do ((i 0 (+ i 1))) ((= i 3)))"),
        SrsValue::Unspecified
    ));
}

#[test]
fn do_test_only_runs_once_when_immediately_true() {
    assert!(matches!(ok("(do ((i 0)) (#t i))"), SrsValue::Integer(0)));
}

#[test]
fn do_too_few_args_fails() {
    assert!(eval_src("(do ((i 0)))").is_err());
}

#[test]
fn display_returns_unspecified() {
    assert!(matches!(ok("(display 42)"), SrsValue::Unspecified));
}

#[test]
fn display_of_a_sequence_evaluates_all() {
    // Chains a couple of displays followed by a normal expression to
    // make sure `display` doesn't disturb subsequent evaluation.
    assert!(matches!(
        ok("(display \"a\") (display 1) (+ 1 2)"),
        SrsValue::Integer(3)
    ));
}

#[test]
fn display_no_args_fails() {
    assert!(eval_src("(display)").is_err());
}

#[test]
fn display_too_many_args_fails() {
    assert!(eval_src("(display 1 2)").is_err());
}

#[test]
fn newline_returns_unspecified() {
    assert!(matches!(ok("(newline)"), SrsValue::Unspecified));
}

#[test]
fn newline_too_many_args_fails() {
    assert!(eval_src("(newline 1)").is_err());
}

#[test]
fn cons_creates_a_pair() {
    assert!(matches!(ok("(cons 1 2)"), SrsValue::Pair(_)));
}

#[test]
fn car_of_cons() {
    assert!(matches!(ok("(car (cons 1 2))"), SrsValue::Integer(1)));
}

#[test]
fn cdr_of_cons() {
    assert!(matches!(ok("(cdr (cons 1 2))"), SrsValue::Integer(2)));
}

#[test]
fn car_of_nested_cons() {
    assert!(matches!(ok("(car (cons (cons 1 2) 3))"), SrsValue::Pair(_)));
}

#[test]
fn cdr_of_nested_cons_is_a_pair() {
    assert!(matches!(
        ok("(cdr (cons 1 (cons 2 (cons 3 (cons 4 4)))))"),
        SrsValue::Pair(_)
    ));
}

#[test]
fn cons_no_args_fails() {
    assert!(eval_src("(cons)").is_err());
}

#[test]
fn cons_one_arg_fails() {
    assert!(eval_src("(cons 1)").is_err());
}

#[test]
fn cons_too_many_args_fails() {
    assert!(eval_src("(cons 1 2 3)").is_err());
}

#[test]
fn car_no_args_fails() {
    assert!(eval_src("(car)").is_err());
}

#[test]
fn car_too_many_args_fails() {
    assert!(eval_src("(car (cons 1 2) (cons 3 4))").is_err());
}

#[test]
fn car_wrong_type_fails() {
    assert!(eval_src("(car 1)").is_err());
}

#[test]
fn cdr_no_args_fails() {
    assert!(eval_src("(cdr)").is_err());
}

#[test]
fn cdr_too_many_args_fails() {
    assert!(eval_src("(cdr (cons 1 2) (cons 3 4))").is_err());
}

#[test]
fn cdr_wrong_type_fails() {
    assert!(eval_src("(cdr 1)").is_err());
}

fn pair_car(value: &SrsValue) -> SrsValue {
    match value {
        SrsValue::Pair(cell) => cell.borrow().0.clone(),
        other => panic!("expected pair, got {:?}", other),
    }
}

fn pair_cdr(value: &SrsValue) -> SrsValue {
    match value {
        SrsValue::Pair(cell) => cell.borrow().1.clone(),
        other => panic!("expected pair, got {:?}", other),
    }
}

#[test]
fn quote_returns_symbol_unevaluated() {
    assert!(matches!(ok("(quote x)"), SrsValue::Symbol(s) if s == "x"));
}

#[test]
fn quote_shorthand_returns_symbol_unevaluated() {
    assert!(matches!(ok("'x"), SrsValue::Symbol(s) if s == "x"));
}

#[test]
fn quote_returns_list_unevaluated() {
    let v = ok("'(1 2 3)");
    assert!(matches!(pair_car(&v), SrsValue::Integer(1)));
    let rest = pair_cdr(&v);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(2)));
}

#[test]
fn quote_does_not_evaluate_operator_looking_symbol() {
    // '(+ 1 2) must stay the list (+ 1 2), not evaluate to 3.
    let v = ok("'(+ 1 2)");
    assert!(matches!(pair_car(&v), SrsValue::Symbol(s) if s == "+"));
}

#[test]
fn quote_no_args_fails() {
    assert!(eval_src("(quote)").is_err());
}

#[test]
fn quote_too_many_args_fails() {
    assert!(eval_src("(quote 1 2)").is_err());
}

#[test]
fn quasiquote_without_unquote_behaves_like_quote() {
    let v = ok("`(1 2 3)");
    assert!(matches!(pair_car(&v), SrsValue::Integer(1)));
}

#[test]
fn quasiquote_evaluates_unquoted_expr() {
    let v = ok("(define x 5) `(a ,x c)");
    let rest = pair_cdr(&v);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(5)));
}

#[test]
fn quasiquote_evaluates_unquoted_computation() {
    let v = ok("`(1 ,(+ 1 1) 3)");
    let rest = pair_cdr(&v);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(2)));
}

#[test]
fn quasiquote_splices_unquote_splicing() {
    let v = ok("(define xs (cons 2 (cons 3 '()))) `(1 ,@xs 4)");
    assert!(matches!(pair_car(&v), SrsValue::Integer(1)));
    let rest = pair_cdr(&v);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(2)));
    let rest = pair_cdr(&rest);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(3)));
    let rest = pair_cdr(&rest);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(4)));
    assert!(matches!(pair_cdr(&rest), SrsValue::Nil));
}

#[test]
fn quasiquote_shorthand_matches_form() {
    let v1 = ok("`(a ,(+ 1 2))");
    let v2 = ok("(quasiquote (a (unquote (+ 1 2))))");
    assert!(matches!(pair_car(&v1), SrsValue::Symbol(s) if s == "a"));
    assert!(matches!(pair_car(&v2), SrsValue::Symbol(s) if s == "a"));
    assert!(matches!(pair_car(&pair_cdr(&v1)), SrsValue::Integer(3)));
    assert!(matches!(pair_car(&pair_cdr(&v2)), SrsValue::Integer(3)));
}

#[test]
fn nested_quasiquote_defers_inner_unquote() {
    // At depth 2, the inner `,x` is not evaluated; the whole thing stays
    // quoted data: (quasiquote (unquote x)).
    let v = ok("(define x 1) `(a `(b ,x))");
    let inner = pair_car(&pair_cdr(&v));
    assert!(matches!(pair_car(&inner), SrsValue::Symbol(s) if s == "quasiquote"));
}

#[test]
fn quasiquote_no_args_fails() {
    assert!(eval_src("(quasiquote)").is_err());
}

#[test]
fn quasiquote_too_many_args_fails() {
    assert!(eval_src("(quasiquote 1 2)").is_err());
}

#[test]
fn apply_native_with_list_args() {
    assert!(matches!(ok("(apply + '(1 2 3))"), SrsValue::Integer(6)));
}

#[test]
fn apply_with_leading_args_and_list() {
    assert!(matches!(ok("(apply + 1 2 '(3 4))"), SrsValue::Integer(10)));
}

#[test]
fn apply_with_lambda() {
    assert!(matches!(
        ok("(apply (lambda (a b) (* a b)) '(6 7))"),
        SrsValue::Integer(42)
    ));
}

#[test]
fn apply_last_arg_not_a_list_fails() {
    assert!(eval_src("(apply + 1)").is_err());
}

#[test]
fn apply_no_args_fails() {
    assert!(eval_src("(apply)").is_err());
}

#[test]
fn apply_one_arg_fails() {
    assert!(eval_src("(apply +)").is_err());
}

#[test]
fn map_single_list() {
    let v = ok("(map (lambda (x) (* x x)) '(1 2 3))");
    assert!(matches!(pair_car(&v), SrsValue::Integer(1)));
    let rest = pair_cdr(&v);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(4)));
    let rest = pair_cdr(&rest);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(9)));
    assert!(matches!(pair_cdr(&rest), SrsValue::Nil));
}

#[test]
fn map_multiple_lists() {
    let v = ok("(map + '(1 2 3) '(10 20 30))");
    assert!(matches!(pair_car(&v), SrsValue::Integer(11)));
    let rest = pair_cdr(&v);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(22)));
    let rest = pair_cdr(&rest);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(33)));
    assert!(matches!(pair_cdr(&rest), SrsValue::Nil));
}

#[test]
fn map_stops_at_shortest_list() {
    let v = ok("(map + '(1 2 3) '(10 20))");
    assert!(matches!(pair_car(&v), SrsValue::Integer(11)));
    let rest = pair_cdr(&v);
    assert!(matches!(pair_car(&rest), SrsValue::Integer(22)));
    assert!(matches!(pair_cdr(&rest), SrsValue::Nil));
}

#[test]
fn map_empty_list_returns_empty_list() {
    assert!(matches!(ok("(map + '())"), SrsValue::Nil));
}

#[test]
fn map_no_args_fails() {
    assert!(eval_src("(map)").is_err());
}

#[test]
fn map_one_arg_fails() {
    assert!(eval_src("(map +)").is_err());
}

fn write_temp_scm(contents: &str) -> String {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!(
        "srs_load_test_{}_{}.scm",
        std::process::id(),
        contents.len()
    ));
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn load_defines_bindings_in_current_env() {
    let path = write_temp_scm("(define square (lambda (x) (* x x)))");
    let src = format!("(load \"{}\") (square 6)", path);
    assert!(matches!(ok(&src), SrsValue::Integer(36)));
    std::fs::remove_file(path).ok();
}

#[test]
fn load_returns_value_of_last_expression() {
    let path = write_temp_scm("(+ 1 2) (+ 3 4)");
    let src = format!("(load \"{}\")", path);
    assert!(matches!(ok(&src), SrsValue::Integer(7)));
    std::fs::remove_file(path).ok();
}

#[test]
fn load_missing_file_fails() {
    assert!(eval_src("(load \"/nonexistent/path/to/file.scm\")").is_err());
}

#[test]
fn load_no_args_fails() {
    assert!(eval_src("(load)").is_err());
}

#[test]
fn load_too_many_args_fails() {
    assert!(eval_src("(load \"a\" \"b\")").is_err());
}

#[test]
fn load_non_string_path_fails() {
    assert!(eval_src("(load 42)").is_err());
}
