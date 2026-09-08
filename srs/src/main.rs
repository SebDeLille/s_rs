use std::io::{self, Write};

use libsrs::interpretor::evaluator::{eval, global_env};
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::reader::read_all;
use libsrs::types::core::SrsValue;

fn main() {
    let env = global_env();
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("srs> ");
        io::stdout().flush().ok();

        input.clear();
        let bytes_read = match stdin.read_line(&mut input) {
            Ok(n) => n,
            Err(err) => {
                eprintln!("error: {}", err);
                continue;
            }
        };

        // EOF (Ctrl-D)
        if bytes_read == 0 {
            println!();
            break;
        }

        let line = input.trim();
        if line.is_empty() {
            continue;
        }
        if line == "(exit)" || line == ",exit" || line == ",quit" {
            break;
        }

        let lexemes = match get_lexemes(line) {
            Ok(lexemes) => lexemes,
            Err(err) => {
                eprintln!("lex error: {}", err);
                continue;
            }
        };

        let values = match read_all(lexemes) {
            Ok(values) => values,
            Err(err) => {
                eprintln!("read error: {}", err);
                continue;
            }
        };

        for value in &values {
            match eval(value, &env) {
                Ok(SrsValue::Unspecified) => {}
                Ok(result) => println!("{}", result),
                Err(err) => {
                    eprintln!("eval error: {}", err);
                    break;
                }
            }
        }
    }
}
