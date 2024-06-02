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

#[derive(Debug)]
pub enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Var(String),
    Decl(String, String, Box<Expr>),
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

#[derive(Debug)]
pub struct Ast {
    pub root: Vec<Expr>,
}

const RESULTADD: &str = "Ast { root: [Assign(\"x\", Number(10)), Assign(\"y\", Number(20)), Return(Add(Var(\"x\"), Var(\"y\")))] }";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_ast() {
        let tast = Ast {
            root: vec![
                Expr::Assign("x".to_string(), Box::new(Expr::Number(10))),
                Expr::Assign("y".to_string(), Box::new(Expr::Number(20))),
                Expr::Return(Box::new(
                    Expr::Add(
                        Box::new(Expr::Var("x".to_string())),
                        Box::new(Expr::Var("y".to_string())),
                    ),
                ))
            ]
        };
        assert_eq!(format!("{:?}", tast).as_str(), RESULTADD);
    }
}
