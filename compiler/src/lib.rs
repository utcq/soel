use ast::{Ast, Expr};

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

    }
}
