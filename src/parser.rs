use logos::Lexer;

use crate::{ast::Workout, lexer::Token};

pub struct Parser<'source> {
    lexer: Lexer<'source, Token<'source>>,
    errors: Vec<String>,
}

impl<'source> Parser<'source> {
    pub fn new(lexer: Lexer<'source, Token<'source>>) -> Self {
        Self {
            lexer,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Workout, Vec<String>> {
        let workout = Workout { sets: vec![] };

        Ok(workout)
    }
}
