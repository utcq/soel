use std::collections::VecDeque;

use logos::Logos;
use reports::{sourcemap::SourceKey, IntoReport, Level, Report, ReportContext};
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
pub struct LexerError(SourceKey, Span, LexerErrorKind);

impl IntoReport for LexerError {
    fn into_report(self) -> reports::Report {
        Report::new(
            Level::Error,
            self.1,
            self.0,
            "Lexer error",
            Some(match self.2 {
                LexerErrorKind::UnknownSymbol => "Unknown symbol",
                LexerErrorKind::NumberTooBig => "Number is too big to be lexed",
                LexerErrorKind::EndOfInput => "End of input found",
            }),
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LexerErrorKind {
    #[default]
    UnknownSymbol,
    NumberTooBig,
    EndOfInput,
}

pub type Span = std::ops::Range<usize>;

#[derive(Debug, Clone)]
pub struct Spanned<T>(Span, T);

#[derive(Debug, Clone, Logos)]
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

pub type SpannedToken = Spanned<Token>;

#[derive(Debug, Clone)]
pub struct TokenStream {
    tokens: VecDeque<SpannedToken>,
    index: usize,
}

impl TokenStream {
    pub fn with_vec_deque(tokens: impl Into<VecDeque<SpannedToken>>) -> Self {
        let tokens = tokens.into();
        Self { tokens, index: 0 }
    }

    pub fn with_iter(tokens: impl IntoIterator<Item = SpannedToken>) -> Self {
        let tokens = tokens.into_iter().collect::<VecDeque<_>>();
        Self { tokens, index: 0 }
    }

    pub fn skip_token(&mut self) -> Option<SpannedToken> {
        self.next()
    }

    pub fn skip_tokens(stream: &mut TokenStream, count: usize) -> Vec<SpannedToken> {
        let mut taken_tokens = vec![];

        for _ in 0..count {
            if let Some(token) = stream.next() {
                taken_tokens.push(token);
            } else {
                break;
            }
        }

        taken_tokens
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<SpannedToken> {
        if self.index < self.tokens.len() {
            let token = self.tokens[self.index].clone();

            self.index += 1;

            Some(token)
        } else {
            None
        }
    }

    #[must_use]
    pub fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.index)
    }

    pub fn take_while<P>(stream: &mut TokenStream, predicate: P) -> Vec<SpannedToken>
    where
        P: Fn(&SpannedToken) -> bool,
    {
        let mut taken_tokens = vec![];

        while let Some(token) = stream.peek() {
            if predicate(token) {
                if let Some(token) = stream.next() {
                    taken_tokens.push(token);
                }
            } else {
                break;
            }
        }

        taken_tokens
    }

    #[must_use]
    pub fn previous(&self) -> Option<SpannedToken> {
        if self.index > 0 {
            Some(self.tokens[self.index - 1].clone())
        } else {
            None
        }
    }
}

pub fn tokenize(report_ctx: &mut ReportContext, source_key: SourceKey, input: &str) -> TokenStream {
    let lexer = Token::lexer(input).spanned();
    let mut tokens = Vec::new();

    for (token, span) in lexer {
        match token {
            Ok(t) => tokens.push(Spanned(span, t)),
            Err(e) => {
                let error = LexerError(source_key, span, e);

                report_ctx.push(error.into_report());
            }
        }
    }

    TokenStream::with_vec_deque(tokens)
}
