use std::io::{self, BufRead, Write};

use libsrs::interpretor::evaluator::Evaluator;
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::translator::translate_all;
use libsrs::types::core::SrsValue;
use libsrs::types::error::SrsError;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let result = if let Some(program) = args.get(1) {
        run_one_shot(program)
    } else {
        run_repl()
    };

    match result {
        Ok(Some(value)) => {
            println!("= {}", value);
            std::process::exit(0);
        }
        Ok(None) => std::process::exit(0),
        Err(SrsError::Exit(code)) => std::process::exit(code as i32),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_one_shot(input: &str) -> Result<Option<SrsValue>, SrsError> {
    evaluate(input)
}

fn run_repl() -> Result<Option<SrsValue>, SrsError> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    let mut lines = stdin.lock().lines();

    loop {
        stdout_lock.flush().ok();
        let line = match lines.next() {
            Some(Ok(line)) => line,
            Some(Err(_)) => return Ok(None),
            None => return Ok(None),
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "exit" || trimmed == "quit" {
            return Ok(None);
        }

        match evaluate(trimmed) {
            Ok(Some(value)) => println!("= {}", value),
            Ok(None) => {}
            Err(SrsError::Exit(code)) => return Err(SrsError::Exit(code)),
            Err(e) => eprintln!("error: {}", e),
        }
    }
}

fn evaluate(input: &str) -> Result<Option<SrsValue>, SrsError> {
    let lexemes = get_lexemes(input)?;
    let values = translate_all(lexemes)?;
    if values.is_empty() {
        return Ok(None);
    }

    let evaluator = Evaluator::new();
    let mut last: Option<SrsValue> = None;
    for value in values {
        match evaluator.eval(&value) {
            Ok(result) => last = Some(result),
            Err(SrsError::Exit(code)) => return Err(SrsError::Exit(code)),
            Err(e) => return Err(e),
        }
    }
    Ok(last)
}
