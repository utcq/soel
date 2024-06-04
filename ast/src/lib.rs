/*
@ Author:  Unity
@ Date:    2024-06-01
@ Target:  To implement the AST for the language
@ Status:  In progress

Expr {
    Add/Sub/Mul/Div/Pow [left, right]
    Neg [term]
    Decl [name, type, value]
    Assign [name, source]
    Call [name, args]
    If [cond, then, else]
    While [cond, body]
    For [init, cond, step, body]
    Parameter [name, type]
    Function [name, args, body]
}
*/

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Spanned<T>(pub core::ops::Range<usize>, pub T);

impl<T> Spanned<T> {
    pub fn map<B, F>(self, f: F) -> Spanned<B>
    where
        F: FnOnce(T) -> B,
    {
        Spanned(self.0.clone(), f(self.1))
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(Spanned<i32>),
    Ident(Spanned<String>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Decl(String, Type, Box<Expr>),
    Assign(String, Box<Expr>),
    Call(String, Vec<Expr>),
    Block(Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
    Return(Box<Expr>),
    Break,
    Continue,
    Function(String, String, Vec<(String, String)>, Box<Expr>),
    Empty,
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,

    Other(String),
}

#[derive(Debug, Clone)]
pub struct Ast {
    pub root: Vec<Expr>,
}
