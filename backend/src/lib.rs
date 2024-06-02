use ast::{Ast, Expr};
use swriter::AVRWriter;
pub enum BackendError {
    AssemblerError,
}

struct Function {
    name: String,
    ret: String,
    args: Vec<String>,
    address: usize,
}

struct Variable {
    name: String,
    var_type: String,
    size: u16,
    stack_lower: u16,
    stack_higher: u16,
}
struct Context {
    functions: Vec<Function>,
    locals: Vec<Variable>,
    text: usize,
    data: usize,
    stack_offset: u16,
}
pub struct Seb<'a> {
    nodes: &'a Vec<Expr>,
    assm: AVRWriter,
    ctx: Context,
}

impl<'a> Seb<'a> {
    pub fn new(source: &'a Vec<Expr>) -> Self {
        Seb {
            nodes: source,
            assm: AVRWriter::new(),
            ctx: Context {
                functions: Vec::new(),
                locals: Vec::new(),
                text: 0,
                data: 0,
                stack_offset: 0,
            },
        }
    }

    fn emit_function(&mut self, name: &String, ret: &String, args: &Vec<(String, String)>, body: &Expr) -> Result<(), BackendError> {
        let addr = self.assm.create_label(name);
        
        self.ctx.functions.push(Function {
            name: name.clone(),
            ret: ret.clone(),
            args: args.iter().map(|(_, ty)| ty.clone()).collect(),
            address: addr,
        });
        self.assm.select_label(addr);

        self.assm.function_prologue();

        match body {
            Expr::Block(stats) => {
                for stat in stats {
                    self.emit_statement(stat)?;
                }
            }
            _ => {
                // Declaration
            }
        }

        for _ in 0..self.ctx.stack_offset {
            self.assm.POP(swriter::Registers::R0);
        }

        // Fix sync problems by adding 1*locals clock cycles
        for i in 0..self.ctx.locals.len() {
            self.assm.append_after("rcall .+0".to_string(), 1+i);
        }

        self.assm.function_epilogue();
        Ok(())
    }

    fn load_constant(&mut self, val: i32, first:bool) -> Result<u16, BackendError> {
        let dests: (swriter::Registers, swriter::Registers);
        if !first { dests = (swriter::Registers::R18, swriter::Registers::R19); }
        else { dests = (swriter::Registers::R24, swriter::Registers::R25);}

        self.assm.LDI(dests.0, val & 0xff);
        self.assm.LDI(dests.1, (val >> 8) & 0xff);
        Ok(16)
    }

    fn load_variable(&mut self, name: String, first: bool) -> Result<u16, BackendError> {
        let dests: (swriter::Registers, swriter::Registers);
        if !first { dests = (swriter::Registers::R18, swriter::Registers::R19); }
        else { dests = (swriter::Registers::R24, swriter::Registers::R25);}
        for var in self.ctx.locals.iter() {
            if var.name == name {
                self.assm.LDD(dests.0, swriter::Registers::Y, var.stack_lower);
                self.assm.LDD(dests.1, swriter::Registers::Y, var.stack_higher);
                return Ok(var.size);
            }
        }
        Err(BackendError::AssemblerError)
    }

    fn emit_binop(&mut self, expr: &Expr, lhs: &Expr, rhs: &Expr, first: bool) -> Result<u16, BackendError> {
        let lhs_size = self.emit_expression(lhs, !first)?;
        let rhs_size = self.emit_expression(rhs, first)?;

        match expr {
            Expr::Add(_, _) => {
                self.assm.ADD(swriter::Registers::R24, swriter::Registers::R18);
                self.assm.ADC(swriter::Registers::R25, swriter::Registers::R19);
            }
            Expr::Sub(_, _) => {
                //self.assm.SUB(swriter::Registers::R24, swriter::Registers::R18);
                //self.assm.SBC(swriter::Registers::R25, swriter::Registers::R19);
            }
            _ => { todo!("Unsupported node type") }
        }

        Ok(lhs_size.max(rhs_size))
    }

    fn emit_expression(&mut self, expr: &Expr, first: bool) -> Result<u16, BackendError> {
        match expr {
            Expr::Number(value) => {
                return self.load_constant(value.clone(), first);
            }
            Expr::Var(name) => {
                return self.load_variable(name.to_string(), first);
            }
            Expr::Add(lhs, rhs) | Expr::Sub(lhs, rhs) => {
                return self.emit_binop(expr, lhs, rhs, first);
            }
            _ => { todo!("Unsupported node type") },
        }
        Ok(0)
    }

    fn resolve_size(&self, ty: &String) -> Result<u16, BackendError> {
        match ty.as_str() {
            "int" => Ok(2),
            _ => todo!("Unsupported type"),
        }
    }

    fn emit_declaration(&mut self, name: &String, ty: &String, value: &Expr) -> Result<(), BackendError> {
        let size = self.resolve_size(ty)?;
        let lower = std::cmp::max(size/2, 0);
        self.ctx.locals.push(Variable {
            name: name.clone(),
            var_type: ty.clone(),
            size,
            stack_lower: self.ctx.stack_offset+lower,
            stack_higher: self.ctx.stack_offset+size,
        });
        self.emit_expression(value, true)?;
        self.assm.STD(swriter::Registers::Y, self.ctx.stack_offset+size, swriter::Registers::R25);
        self.assm.STD(swriter::Registers::Y, self.ctx.stack_offset+lower, swriter::Registers::R24);
        self.ctx.stack_offset += size;

        Ok(())
    }

    fn emit_return(&mut self, expr: &Expr) -> Result<(), BackendError> {
        self.emit_expression(expr, true)?;
        Ok(())
    }

    fn emit_statement(&mut self, stat: &Expr) -> Result<(), BackendError> {
        println!("{:?}", stat);
        match stat {
            Expr::Decl(name, ty, value) => { self.emit_declaration(name, ty, value)?; }
            Expr::Return(expr) => { self.emit_return(expr)?; }
            _ => {}, //{todo!("Unsupported node type");}
        }
        Ok(())
    }

    pub fn process(&mut self) -> Result<(), BackendError> {
        self.assm.new_global("main");
        self.ctx.data = self.assm.create_section(".data");
        self.ctx.text = self.assm.create_section(".text");
        self.assm.select_section(self.ctx.text);
        for node in self.nodes {
            match node {
                Expr::Function(name,ret,args,body) => {
                    self.emit_function(name, ret, args, body)?;
                }
                _ => todo!("Unsupported node type"),
            }
        }
        println!("{}", self.assm.repr());
        Ok(())
    }
}


pub fn compile(source: Ast) {
    let mut seb = Seb::new(&source.root);
    let _ = seb.process();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_test0() {
        let tast = Ast {
            /*
            int main() {
                int x = 10;
                int y = 20;
                return x + y;
            }
            */
            root: vec![
                Expr::Function("main".to_string(), "int".to_string(), vec![], Box::new(Expr::Block(vec![
                    Expr::Decl("x".to_string(), "int".to_string(), Box::new(Expr::Number(5))),
                    Expr::Decl("y".to_string(), "int".to_string(), Box::new(Expr::Number(8))),
                    Expr::Return(Box::new(
                        Expr::Add(
                            Box::new(Expr::Var("x".to_string())),
                            Box::new(Expr::Var("y".to_string())),
                        ),
                    ))
                ])))
            ]
        };
        compile(tast);
    }
}
