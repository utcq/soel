
use lexer::Token;

pub enum ParserError {
    FailedTopLevel,
    FailedFunction,
    ConversionError,
    UnexpectedToken,
}

pub struct Parser {
    source: Vec<Token>,
    ast: ast::Ast,
}

impl Parser {
    pub fn new(source: Vec<Token>) -> Self {
        Self {
            source,
            ast: ast::Ast {
                root: Vec::new()
            },
        }
    }

    fn next(&mut self) -> Option<Token> {
        match self.source.pop() {
            Some(t) => Some(t),
            None => None,
        }
    }

    fn peek(&self) -> Result<&Token, ParserError> {
        match self.source.last() {
            Some(t) => Ok(t),
            None => Err(ParserError::UnexpectedToken),
        }
    }

    fn expect<T: Copy + std::convert::From<std::string::String> + std::convert::From<i32>> (&mut self, token: Token) -> Result<T, ParserError> {
        match self.next() {
            Some(t) => {
                if t == token {
                    match t {
                        Token::Number(n) => {
                            return Ok( n.1.into() )
                        }
                        Token::Identifier(s) => {
                            return Ok( s.1.into() )
                        }
                        _ => Err(ParserError::ConversionError),
                    }
                } else {
                    Err(ParserError::UnexpectedToken)
                }
            }
            None => Err(ParserError::UnexpectedToken),
        }
    }

    fn parse_function(&mut self) -> Result<(), ParserError> {
        let x: &str = &*self.expect(Token::Function(0..0))?;

        Ok(())
    }

    fn parse_toplevel(&mut self) -> Result<(), ParserError> {
        while let Some(tok) = self.next() {
            match tok {
                Token::Function(_) => {
                    self.parse_function();
                }
                _ => {
                    return Err(ParserError::FailedTopLevel);
                }
            }
        }

        Ok(())
    }

    pub fn process(&mut self) -> Result<(), ParserError> {
        self.parse_toplevel()
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_test0() {
        let src = std::fs::read_to_string("../syntax/syntax0.se").unwrap();
        let tokens = lexer::lex(&src);
        let mut parser = Parser::new(tokens);
        let _ = parser.process();
    }
}
