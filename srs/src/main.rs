use libsrs::interpretor::lexical_analyzer::get_lexemes;

fn main() {
    let lexemes = get_lexemes("(+ 1 2)").unwrap();
    println!("{:?}", lexemes);
}
