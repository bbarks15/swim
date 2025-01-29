use logos::Logos;
use swim_parser::lexer::Token;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        std::process::exit(1);
    }

    let set_file = std::fs::read_to_string(&args[1]).unwrap();

    let mut lexer = Token::lexer(&set_file);

    while let Some(token) = lexer.next() {
        println!("{:?}: {:?}", token, lexer.slice());
    }
}
