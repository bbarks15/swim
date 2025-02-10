use crate::ast::*;
use crate::lexer::Token;
use logos::Logos;

pub struct Parser<'source> {
    tokens: Vec<Result<Token<'source>, ()>>,
    current: usize,
}

impl<'source> Parser<'source> {
    pub fn new(input: &'source str) -> Self {
        let lexer = Token::lexer(input);
        Self {
            tokens: lexer.collect(),
            current: 0,
        }
    }

    fn peek(&self) -> Option<&Result<Token<'source>, ()>> {
        self.tokens.get(self.current)
    }

    fn next(&mut self) -> Option<Result<Token<'source>, ()>> {
        if self.current >= self.tokens.len() {
            return None;
        }

        let token = self.tokens[self.current].clone();
        self.current += 1;
        Some(token)
    }

    fn peek_nth(&self, n: usize) -> Option<&Result<Token<'source>, ()>> {
        self.tokens.get(self.current + n)
    }

    pub fn parse(&mut self) -> Result<Workout, String> {
        let mut sets = Vec::new();

        while self.peek().is_some() {
            sets.push(self.parse_set()?);
        }

        Ok(Workout { sets })
    }

    fn parse_set(&mut self) -> Result<Set, String> {
        match self.peek() {
            Some(Ok(Token::Number(_))) => match self.peek_nth(1) {
                Some(Ok(Token::Times)) => self.parse_repetition(),
                _ => self.parse_statement().map(Set::Statement),
            },
            Some(Ok(Token::BraceOpen)) => self.parse_block(),
            _ => Err("Expected number or '{'".to_string()),
        }
    }

    fn parse_repetition(&mut self) -> Result<Set, String> {
        let count = match self.next() {
            Some(Ok(Token::Number(n))) => n,
            _ => return Err("Expected number for repetition count".to_string()),
        };

        match self.next() {
            Some(Ok(Token::Times)) => (),
            _ => return Err("Expected 'x' after repetition count".to_string()),
        }

        let set = Box::new(self.parse_set()?);
        Ok(Set::Repetition { count, set })
    }

    fn parse_block(&mut self) -> Result<Set, String> {
        match self.next() {
            Some(Ok(Token::BraceOpen)) => (),
            _ => return Err("Expected '{'".to_string()),
        }

        let mut sets = Vec::new();

        loop {
            match self.peek() {
                Some(Ok(Token::BraceClose)) => {
                    self.next(); // Consume closing brace
                    break;
                }
                Some(_) => sets.push(self.parse_set()?),
                None => return Err("Unexpected end of input in block".to_string()),
            }
        }

        Ok(Set::Block { sets })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let distance = self.parse_distance()?;
        let stroke = self.parse_stroke()?;
        let interval = self.parse_interval()?;

        Ok(Statement {
            distance,
            stroke,
            interval,
        })
    }

    fn parse_distance(&mut self) -> Result<Distance, String> {
        let value = match self.next() {
            Some(Ok(Token::Number(n))) => n,
            _ => return Err("Expected number for distance".to_string()),
        };

        let unit = match self.next() {
            Some(Ok(Token::Meters)) => DistanceUnit::Meters,
            Some(Ok(Token::Kilometers)) => DistanceUnit::Kilometers,
            _ => return Err("Expected 'm' or 'km' for distance unit".to_string()),
        };

        Ok(Distance { value, unit })
    }

    fn parse_stroke(&mut self) -> Result<Stroke, String> {
        let name = match self.next() {
            Some(Ok(Token::Word(word))) => word.to_string(),
            _ => return Err("Expected stroke name".to_string()),
        };

        let mut modifiers = Vec::new();

        if let Some(Ok(Token::ParenOpen)) = self.peek() {
            self.next(); // Consume '('

            loop {
                match self.next() {
                    Some(Ok(Token::Word(word))) => modifiers.push(word.to_string()),
                    _ => return Err("Expected modifier in parentheses".to_string()),
                }

                match self.peek() {
                    Some(Ok(Token::Comma)) => {
                        self.next(); // Consume comma
                        continue;
                    }
                    Some(Ok(Token::ParenClose)) => {
                        self.next(); // Consume ')'
                        break;
                    }
                    _ => return Err("Expected ',' or ')' after modifier".to_string()),
                }
            }
        }

        Ok(Stroke { name, modifiers })
    }

    fn parse_interval(&mut self) -> Result<Option<Interval>, String> {
        match self.peek() {
            Some(Ok(Token::At)) => {
                self.next(); // Consume '@'
                match self.next() {
                    Some(Ok(Token::Number(n))) => {
                        match self.peek() {
                            Some(Ok(Token::Seconds)) => {
                                self.next(); // Consume 's'
                                Ok(Some(Interval::Seconds(n)))
                            }
                            _ => Ok(Some(Interval::Seconds(n))),
                        }
                    }
                    Some(Ok(Token::Time(time))) => {
                        let parts: Vec<&str> = time.split(':').collect();
                        if parts.len() != 2 {
                            return Err("Invalid time format".to_string());
                        }

                        let minutes = parts[0]
                            .parse::<u32>()
                            .map_err(|_| "Invalid minutes".to_string())?;

                        let seconds = parts[1]
                            .trim_end_matches('s')
                            .parse::<u32>()
                            .map_err(|_| "Invalid seconds".to_string())?;

                        Ok(Some(Interval::MinutesSeconds { minutes, seconds }))
                    }
                    _ => Err("Expected number or time after '@'".to_string()),
                }
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_statement() {
        let input = "100m fly @ 1:30";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        assert_eq!(workout.sets.len(), 1);
        match &workout.sets[0] {
            Set::Statement(stmt) => {
                assert_eq!(stmt.distance.value, 100);
                assert_eq!(stmt.distance.unit, DistanceUnit::Meters);
                assert_eq!(stmt.stroke.name, "fly");
                assert!(stmt.stroke.modifiers.is_empty());
                assert_eq!(
                    stmt.interval,
                    Some(Interval::MinutesSeconds {
                        minutes: 1,
                        seconds: 30
                    })
                );
            }
            _ => panic!("Expected Statement"),
        }
    }

    #[test]
    fn test_simple_repetition() {
        let input = "4x50m free @ 60s";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        assert_eq!(workout.sets.len(), 1);
        match &workout.sets[0] {
            Set::Repetition { count, set } => {
                assert_eq!(*count, 4);
                match &**set {
                    Set::Statement(stmt) => {
                        assert_eq!(stmt.distance.value, 50);
                        assert_eq!(stmt.stroke.name, "free");
                        assert_eq!(stmt.interval, Some(Interval::Seconds(60)));
                    }
                    _ => panic!("Expected Statement inside Repetition"),
                }
            }
            _ => panic!("Expected Repetition"),
        }
    }

    #[test]
    fn test_stroke_with_modifiers() {
        let input = "100m butterfly(drill, kick) @ 30s";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        match &workout.sets[0] {
            Set::Statement(stmt) => {
                assert_eq!(stmt.stroke.name, "butterfly");
                assert_eq!(stmt.stroke.modifiers, vec!["drill", "kick"]);
            }
            _ => panic!("Expected Statement"),
        }
    }

    #[test]
    fn test_simple_block() {
        let input = "{\n  25m choice (easy) @ 60s\n  50m free @ 60s\n}";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        match &workout.sets[0] {
            Set::Block { sets } => {
                assert_eq!(sets.len(), 2);
                match &sets[0] {
                    Set::Statement(stmt) => {
                        assert_eq!(stmt.distance.value, 25);
                        assert_eq!(stmt.stroke.name, "choice");
                        assert_eq!(stmt.stroke.modifiers, vec!["easy"]);
                    }
                    _ => panic!("Expected Statement"),
                }
            }
            _ => panic!("Expected Block"),
        }
    }

    #[test]
    fn test_complex_workout() {
        let input = "\
            1x100m fly @ 1:30\n\
            50m fly @ 60s\n\
            4x {\n\
              25m choice (easy) @ 60s\n\
              12x50m free @ 60s\n\
            }\n\
            12x {\n\
              75m free (easy) @ 60s\n\
              12x50m free (descend) @ 60s\n\
            }";

        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();

        assert_eq!(workout.sets.len(), 4);

        // Test first set: 1x100m fly @ 1:30
        match &workout.sets[0] {
            Set::Repetition { count, set } => {
                assert_eq!(*count, 1);
                match &**set {
                    Set::Statement(stmt) => {
                        assert_eq!(stmt.distance.value, 100);
                        assert_eq!(stmt.stroke.name, "fly");
                        assert_eq!(
                            stmt.interval,
                            Some(Interval::MinutesSeconds {
                                minutes: 1,
                                seconds: 30
                            })
                        );
                    }
                    _ => panic!("Expected Statement"),
                }
            }
            _ => panic!("Expected Repetition"),
        }

        // Test fourth set: complex nested block
        match &workout.sets[3] {
            Set::Repetition { count, set } => {
                assert_eq!(*count, 12);
                match &**set {
                    Set::Block { sets } => {
                        assert_eq!(sets.len(), 2);
                        // Test nested repetition
                        match &sets[1] {
                            Set::Repetition { count, set } => {
                                assert_eq!(*count, 12);
                                match &**set {
                                    Set::Statement(stmt) => {
                                        assert_eq!(stmt.stroke.modifiers, vec!["descend"]);
                                    }
                                    _ => panic!("Expected Statement"),
                                }
                            }
                            _ => panic!("Expected Repetition"),
                        }
                    }
                    _ => panic!("Expected Block"),
                }
            }
            _ => panic!("Expected Repetition"),
        }
    }

    #[test]
    fn test_error_handling() {
        // Test missing distance unit
        let input = "100 fly @ 30s";
        let mut parser = Parser::new(input);
        assert!(parser.parse().is_err());

        // Test invalid interval format
        let input = "100m fly @ 1:xyz";
        let mut parser = Parser::new(input);
        assert!(parser.parse().is_err());

        // Test unclosed block
        let input = "4x {\n  50m free @ 30s\n";
        let mut parser = Parser::new(input);
        assert!(parser.parse().is_err());

        // Test invalid repetition format
        let input = "4 50m free @ 30s";
        let mut parser = Parser::new(input);
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_interval_formats() {
        // Test seconds format
        let input = "100m free @ 45s";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();
        match &workout.sets[0] {
            Set::Statement(stmt) => {
                assert_eq!(stmt.interval, Some(Interval::Seconds(45)));
            }
            _ => panic!("Expected Statement"),
        }

        // Test minutes:seconds format
        let input = "100m free @ 1:30";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();
        match &workout.sets[0] {
            Set::Statement(stmt) => {
                assert_eq!(
                    stmt.interval,
                    Some(Interval::MinutesSeconds {
                        minutes: 1,
                        seconds: 30
                    })
                );
            }
            _ => panic!("Expected Statement"),
        }
    }

    #[test]
    fn test_distance_units() {
        // Test meters
        let input = "100m free @ 30s";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();
        match &workout.sets[0] {
            Set::Statement(stmt) => {
                assert_eq!(stmt.distance.unit, DistanceUnit::Meters);
            }
            _ => panic!("Expected Statement"),
        }

        // Test kilometers
        let input = "1km free @ 30s";
        let mut parser = Parser::new(input);
        let workout = parser.parse().unwrap();
        match &workout.sets[0] {
            Set::Statement(stmt) => {
                assert_eq!(stmt.distance.unit, DistanceUnit::Kilometers);
            }
            _ => panic!("Expected Statement"),
        }
    }
}
