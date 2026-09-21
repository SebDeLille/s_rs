use std::env;
use std::io::{self, Write};
use std::process;
use std::rc::Rc;

use libsrs::interpretor::evaluator::{EvalError, EvalErrorKind, global_env_with_frontend};
use libsrs::interpretor::repl::{EvalOutcome, eval_source};
use libsrs::interpretor::startup::load_startup_scripts;
use libsrs::types::core::{Env, SrsValue};

fn main() {
    let env = global_env_with_frontend("cli");
    load_startup_scripts(&env);

    let args: Vec<String> = env::args().skip(1).collect();
    if let Some(path) = args.first() {
        run_file(path, &env);
        return;
    }

    run_repl(&env);
}

/// Reads and evaluates the given source file, then exits.
fn run_file(path: &str, env: &Rc<Env>) {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("erreur: {}: {}", path, e);
            process::exit(1);
        }
    };

    match eval_source(&source, env) {
        Ok(EvalOutcome::Done(_)) => {}
        Ok(EvalOutcome::Incomplete) => {
            eprintln!("erreur: {}: unexpected end of input", path);
            process::exit(1);
        }
        Err(EvalError {
            kind: EvalErrorKind::Exit(code),
        }) => {
            process::exit(code);
        }
        Err(e) => {
            eprintln!("erreur: {}: {}", path, e);
            process::exit(1);
        }
    }
}

fn run_repl(env: &Rc<Env>) {
    println!("srs REPL - Ctrl+D pour quitter");
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

        match eval_line(line, &env) {
            Ok(()) => {}
            Err(EvalError {
                kind: EvalErrorKind::Exit(code),
            }) => {
                process::exit(code);
            }
            Err(e) => {
                eprintln!("erreur: {e}");
            }
        }
    }
}

fn eval_line(line: &str, env: &Rc<Env>) -> Result<(), EvalError> {
    match eval_source(line, env)? {
        // The CLI evaluates one line at a time and doesn't support
        // multi-line input: an incomplete form is just reported as an
        // error, same as before this pipeline was factored out.
        EvalOutcome::Incomplete => Err(EvalError {
            kind: EvalErrorKind::Native("unexpected end of input".to_string()),
        }),
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
