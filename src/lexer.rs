use logos::Logos;

#[derive(Clone, Debug, Logos, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
pub enum Token<'source> {
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<u32>().ok())]
    Number(u32),

    #[token("x", priority = 4)]
    Times,

    #[token("{")]
    BraceOpen,

    #[token("}")]
    BraceClose,

    #[token("m", priority = 4)]
    Meters,

    #[token("km", priority = 4)]
    Kilometers,

    #[regex(r"[a-zA-Z][a-zA-Z.-]*", |lex| lex.slice(), priority = 2)]
    Word(&'source str),

    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token(",")]
    Comma,

    #[token("@")]
    At,

    #[regex("s", priority = 3)]
    Seconds,

    #[regex(r"[0-9]+:[0-9]+s?", |lex| lex.slice())]
    Time(&'source str),

    #[regex(r"#[^\n]*", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    Comment,
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_basic_tokens() {
        let mut lex = Token::lexer("100");

        assert_eq!(lex.next(), Some(Ok(Token::Number(100))));
        assert_eq!(lex.span(), 0..3);
        assert_eq!(lex.slice(), "100");

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_complex_statements() {
        let mut lex = Token::lexer("100m freestyle @1:30");

        assert_eq!(lex.next(), Some(Ok(Token::Number(100))));
        assert_eq!(lex.span(), 0..3);
        assert_eq!(lex.slice(), "100");

        assert_eq!(lex.next(), Some(Ok(Token::Meters)));
        assert_eq!(lex.span(), 3..4);
        assert_eq!(lex.slice(), "m");

        assert_eq!(lex.next(), Some(Ok(Token::Word("freestyle"))));
        assert_eq!(lex.span(), 5..14);
        assert_eq!(lex.slice(), "freestyle");

        assert_eq!(lex.next(), Some(Ok(Token::At)));
        assert_eq!(lex.span(), 15..16);
        assert_eq!(lex.slice(), "@");

        assert_eq!(lex.next(), Some(Ok(Token::Time("1:30"))));
        assert_eq!(lex.span(), 16..20);
        assert_eq!(lex.slice(), "1:30");

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_modifiers() {
        let mut lex = Token::lexer("butterfly(drill, kick)");

        assert_eq!(lex.next(), Some(Ok(Token::Word("butterfly"))));
        assert_eq!(lex.span(), 0..9);
        assert_eq!(lex.slice(), "butterfly");

        assert_eq!(lex.next(), Some(Ok(Token::ParenOpen)));
        assert_eq!(lex.span(), 9..10);
        assert_eq!(lex.slice(), "(");

        assert_eq!(lex.next(), Some(Ok(Token::Word("drill"))));
        assert_eq!(lex.span(), 10..15);
        assert_eq!(lex.slice(), "drill");

        assert_eq!(lex.next(), Some(Ok(Token::Comma)));
        assert_eq!(lex.span(), 15..16);
        assert_eq!(lex.slice(), ",");

        assert_eq!(lex.next(), Some(Ok(Token::Word("kick"))));
        assert_eq!(lex.span(), 17..21);
        assert_eq!(lex.slice(), "kick");

        assert_eq!(lex.next(), Some(Ok(Token::ParenClose)));
        assert_eq!(lex.span(), 21..22);
        assert_eq!(lex.slice(), ")");

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_intervals() {
        let mut lex = Token::lexer("@30s");

        assert_eq!(lex.next(), Some(Ok(Token::At)));
        assert_eq!(lex.span(), 0..1);
        assert_eq!(lex.slice(), "@");

        assert_eq!(lex.next(), Some(Ok(Token::Number(30))));
        assert_eq!(lex.span(), 1..3);
        assert_eq!(lex.slice(), "30");
        assert_eq!(lex.next(), Some(Ok(Token::Seconds)));
        assert_eq!(lex.span(), 3..4);
        assert_eq!(lex.slice(), "s");

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_comments() {
        let mut lex = Token::lexer("100m # comment");

        assert_eq!(lex.next(), Some(Ok(Token::Number(100))));
        assert_eq!(lex.span(), 0..3);
        assert_eq!(lex.slice(), "100");

        assert_eq!(lex.next(), Some(Ok(Token::Meters)));
        assert_eq!(lex.span(), 3..4);
        assert_eq!(lex.slice(), "m");

        assert_eq!(lex.next(), None);
    }
}
