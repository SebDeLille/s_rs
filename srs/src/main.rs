use std::io::{self, Write};
use std::rc::Rc;

use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
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
    let lexemes = get_lexemes(line).map_err(|e| e.to_string())?;
    let values = read_all(lexemes).map_err(|e| e.to_string())?;

    for value in &values {
        match eval(value, env) {
            Ok(SrsValue::Unspecified) => {}
            Ok(result) => println!("{}", result),
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(())
}
