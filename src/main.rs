use clap::Parser;

use logos::Logos;
use swim_parser::lexer::Token;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    set: String,
}

fn main() {
    let args = Args::parse();

    let set_file = std::fs::read_to_string(args.set).unwrap();

    let mut lexer = Token::lexer(&set_file);

    while let Some(token) = lexer.next() {
        println!("{:?}: {:?}", token, lexer.slice());
    }
}
