use libsrs::interpretor::evaluator::Evaluator;
use libsrs::interpretor::lexical_analyzer::get_lexemes;
use libsrs::interpretor::translator::translate_all;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args.get(1).cloned().unwrap_or_else(|| "(+ 2 3)".to_string());

    println!("input: {}", program);

    let lexemes = match get_lexemes(&program) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("lexer error: {}", e);
            return;
        }
    };

    let values = match translate_all(lexemes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("parser error: {}", e);
            return;
        }
    };

    let evaluator = Evaluator::new();
    for value in values {
        match evaluator.eval(&value) {
            Ok(result) => println!("= {}", result),
            Err(e) => eprintln!("eval error: {}", e),
        }
    }
}
