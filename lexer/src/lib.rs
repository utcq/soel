use std::iter::Enumerate;

pub enum LexerError {
    InvalidToken,
    InvalidSymbol
}

pub type Span = std::ops::Range<usize>;
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Spanned<T>(Span, pub T);

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(Spanned<i32>),
    Identifier(Spanned<String>),
    String(Spanned<String>),

    Return(Span),
    Function(Span),
    Var(Span),
    Asm(Span),
    If(Span),
    Then(Span),
    Else(Span),
    Namespace(Span),
    Here(Span),

    Plus(Span),
    Minus(Span),
    Mul(Span),
    Div(Span),
    Pow(Span),
    Increment(Span),
    Decrease(Span),

    Semicolon(Span),
    Colon(Span),
    Dollar(Span),

    Eqq(Span),
    Eq(Span),
    Not(Span),
    NotEq(Span),

    LParen(Span),
    RParen(Span),
    LBrace(Span),
    RBrace(Span),
    LBracket(Span),
    RBracket(Span),
    Comma(Span),
    Dot(Span),

    Greater(Span),
    Less(Span),

    Comment(Span),
}

const KEYWRD_MAP: &[(&str, fn(Span) -> Token)] = &[
    ("return", Token::Return),
    ("function", Token::Function),
    ("var", Token::Var),
    ("asm", Token::Asm),
    ("if", Token::If),
    ("then", Token::Then),
    ("else", Token::Else),
    ("namespace", Token::Namespace),
    ("here", Token::Here),
];

const OPERATOR_MAP: &[(&str, fn(Span) -> Token)] = &[
    ("+", Token::Plus),
    ("-", Token::Minus),
    ("*", Token::Mul),
    ("/", Token::Div),
    ("**", Token::Pow),
    ("++", Token::Increment),
    ("--", Token::Decrease),
    (";", Token::Semicolon),
    (":", Token::Colon),
    ("$", Token::Dollar),
    ("=", Token::Eq),
    ("==", Token::Eqq),
    ("!", Token::Not),
    ("!=", Token::NotEq),
    ("(", Token::LParen),
    (")", Token::RParen),
    ("{", Token::LBrace),
    ("}", Token::RBrace),
    ("[", Token::LBracket),
    ("]", Token::RBracket),
    (",", Token::Comma),
    (".", Token::Dot),
    (">", Token::Greater),
    ("<", Token::Less),
    ("//", Token::Comment),
];

struct Lexer<'a> {
    input: Enumerate<std::str::Chars<'a>>,
    tokens: Vec<Token>,
    current: (usize, char),
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Lexer<'a> {
        Lexer {
            input: src.chars().enumerate(),
            tokens: Vec::new(),
            current: (0, '\0'),
        }
    }

    fn advance(&mut self) {
        self.current = self.input.next().unwrap_or((0, '\0'));
    }

    fn process_digit(&mut self) -> Result<(), LexerError> {
        let mut span: (usize, usize) = (self.current.0, self.current.0);
        let mut strep = String::new();
        while self.current.1 == '0'
            || self.current.1 == 'b'
            || self.current.1 == 'x'
            || self.current.1.is_digit(16)
        {
            strep.push(self.current.1);
            span.1 += 1;
            self.advance();
        }
        let res;
        if strep.starts_with("0x") {
            res = i32::from_str_radix(strep.trim_start_matches("0x"), 16).unwrap();
        } else if strep.starts_with("0b") {
            res = i32::from_str_radix(strep.trim_start_matches("0b"), 2).unwrap();
        } else {
            res = strep.parse().unwrap();
        }
        self.tokens
            .push(Token::Number(Spanned(span.0..span.1, res)));
        Ok(())
    }

    fn process_identifier(&mut self) -> Result<(), LexerError> {
        let mut span: (usize, usize) = (self.current.0, self.current.0);
        let mut strep = String::new();
        while self.current.1.is_alphanumeric() {
            strep.push(self.current.1);
            span.1 += 1;
            self.advance();
        }

        for (kwrd, token) in KEYWRD_MAP.iter() {
            if kwrd == &strep {
                self.tokens.push(token(span.0..span.1));
                return Ok(());
            }
        }

        self.tokens
            .push(Token::Identifier(Spanned(span.0..span.1, strep)));
        Ok(())
    }

    fn process_string(&mut self) -> Result<(), LexerError> {
        let mut span: (usize, usize) = (self.current.0, self.current.0);
        let mut strep = String::new();
        self.advance();
        while self.current.1 != '"' {
            strep.push(self.current.1);
            span.1 += 1;
            self.advance();
        }
        span.1 += 1;
        self.advance();
        self.tokens
            .push(Token::String(Spanned(span.0..span.1, strep)));
        Ok(())
    }

    fn check_sym(&mut self, strep: String) -> Option<&fn(Span) -> Token> {
        for (sym, token) in OPERATOR_MAP.iter() {
            if sym == &strep {
                return Some(token);
            }
        }
        return None;
    }

    fn process_symbol(&mut self) -> Result<(), LexerError> {
        let mut span: (usize, usize) = (self.current.0, self.current.0);
        let mut strep = String::new();
        while self
            .check_sym(strep.clone() + &self.current.1.to_string())
            .is_some()
        {
            strep.push(self.current.1);
            span.1 += 1;
            self.advance();
        }
        let _token = self
            .check_sym(strep.clone());
        match _token {
            Some(token) => {
                let tko = token(span.0..span.1);
                match tko {
                    Token::Comment(_) => {
                        while self.current.1 != '\n' {
                            self.advance();
                        }
                    }
                    _ => {}
                }
                self.tokens.push(tko);
            }
            None => {
                return Err(LexerError::InvalidSymbol);
            }
        }
        Ok(())
    }

    pub fn process(&mut self) -> Result<(), LexerError> {
        self.advance();
        while self.current.1 != '\0' {
            match self.current.1 {
                '0'..='9' => { self.process_digit()?; },
                'a'..='z' | 'A'..='Z' => { self.process_identifier()?; },
                '"' => { self.process_string()?; },
                '+'..='/'
                | '{'
                | '}'
                | '['
                | ']'
                | '('
                | ')'
                | '='
                | ';'
                | ':'
                | '>'
                | '<'
                | '!'
                | '$' => { self.process_symbol()?; } ,

                ' ' | '\n' | '\t' => {
                    self.advance();
                }
                _ => {
                    /* TODO: Err Handler */
                    return Err(LexerError::InvalidToken);
                }
            }
        }
        Ok(())
    }
}

pub fn lex(src: &str) -> Vec<Token> {
    let mut l = Lexer::new(src);
    let _ = l.process();
    return l.tokens;
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn lexer_test0() {
        let input = read_to_string("../syntax/syntax0.se").unwrap();
        let result = lex(&input);
        result.into_iter().for_each(|token| println!("{token:?}"))
        /*let expected = vec![
            Token::Return,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Number(10),
            Token::Semicolon,
        ];
        let result = lexer(input);
        assert_eq!(result, expected);*/
    }
}