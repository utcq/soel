use std::{
    iter::{Enumerate, Peekable},
    ops::Deref,
};

use logos::Logos;
mod helpers {

    use logos::Lexer;
    use num_traits::Num;

    use super::{LexerErrorKind, Token};

    pub fn parse_number<N: Num>(lexer: &Lexer<'_, Token>) -> Result<N, LexerErrorKind> {
        let slice = lexer.slice();

        N::from_str_radix(slice, 10).map_err(|_| LexerErrorKind::NumberTooBig)
    }

    pub fn unquote_str(lexer: &Lexer<'_, Token>) -> String {
        let input = lexer.slice();

        let start = 1;
        let end = input.len() - 1;

        (input[start..end]).into()
    }
}
pub struct LexerError(LexerErrorKind);

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LexerErrorKind {
    #[default]
    UnknownSymbol,
    NumberTooBig,
    EndOfInput,
}

pub type Span = std::ops::Range<usize>;

#[derive(Debug)]
pub struct Spanned<T>(Span, T);

#[derive(Debug, Logos)]
#[logos(error = LexerErrorKind)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[regex("[0-9]+", helpers::parse_number)]
    Number(i32),

    #[regex("[a-zA-Z]([a-zA-Z0-9]|_)*", |lexer| lexer.slice().to_string())]
    Identifier(String),

    #[regex(r#""(\\[\\"]|[^"])*""#, helpers::unquote_str)]
    String(String),

    #[token("return")]
    Return,
    #[token("var")]
    Var,
    #[token("asm")]
    Asm,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("namespace")]
    Namespace,
    #[token("here")]
    Here,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("**")]
    Pow,
    #[token("++")]
    Increment,
    #[token("--")]
    Decrease,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("$")]
    Dollar,
    #[token("==")]
    Eqq,
    #[token("=")]
    Eq,
    #[token("!")]
    Not,
    #[token("!=")]
    NotEq,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(">")]
    Greater,
    #[token("<")]
    Less,
    #[regex("//(.*)\n")]
    Comment,
}
