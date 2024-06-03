#[cfg(test)]
mod tests {
    use ast::{Ast, Expr};

    use super::*;
    #[test]
    fn test_add_ast() {
        let tast = Ast {
            /*
            int main() {
                int x = 10;
                int y = 20;
                return x + y;
            }
            */
            root: vec![Expr::Function(
                "main".to_string(),
                "int".to_string(),
                vec![],
                Box::new(Expr::Block(vec![
                    Expr::Decl(
                        "x".to_string(),
                        "int".to_string(),
                        Box::new(Expr::Number(5)),
                    ),
                    Expr::Decl(
                        "y".to_string(),
                        "int".to_string(),
                        Box::new(Expr::Number(8)),
                    ),
                    /*Expr::Return(Box::new(
                        Expr::Add(
                            Box::new(Expr::Var("x".to_string())),
                            Box::new(Expr::Var("y".to_string())),
                        ),
                    ))*/
                ])),
            )],
        };
    }
}
