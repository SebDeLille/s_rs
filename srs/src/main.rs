use std::io::{self, Write};
use std::rc::Rc;

use libsrs::interpretor::evaluator::global_env;
use libsrs::interpretor::repl::{EvalOutcome, eval_source};
use libsrs::types::core::{Env, SrsValue};

fn main() {
    println!("srs REPL - Ctrl+D pour quitter");
    let env = global_env();
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("srs> ");
        io::stdout().flush().ok();

        input.clear();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("erreur de lecture: {e}");
                break;
            }
        }

        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        if let Err(e) = eval_line(line, &env) {
            eprintln!("erreur: {e}");
        }
    }
}

fn eval_line(line: &str, env: &Rc<Env>) -> Result<(), String> {
    match eval_source(line, env)? {
        // The CLI evaluates one line at a time and doesn't support
        // multi-line input: an incomplete form is just reported as an
        // error, same as before this pipeline was factored out.
        EvalOutcome::Incomplete => Err("unexpected end of input".to_string()),
        EvalOutcome::Done(values) => {
            for value in values {
                if !matches!(value, SrsValue::Unspecified) {
                    println!("{}", value);
                }
            }
            Ok(())
        }
    }
}
