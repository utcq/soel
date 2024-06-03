use ast::{Ast, Expr};
use swriter::AVRWriter;

const R24: u32 = 1 << 2; // R24 - R27
const R18: u32 = 2 << 2; // R18 - R23
const R16: u32 = 4 << 2; // R16 - R18  LIMITED TO SHORT INT FOR SMALL OPS
const EMPTY: u32 = 0 << 2; // No register

pub enum BackendError {
    AssemblerError,
}

struct Function {
    name: String,
    ret: String,
    args: Vec<String>,
    address: u16,
}

#[derive(Clone)]
struct Variable {
    name: String,
    var_type: String,
    size: u16,
    stack_offset: u16,
}
struct Context {
    functions: Vec<Function>,
    locals: Vec<Variable>,
    text: u16,
    data: u16,
    stack_offset: u16,
    used_regs: u32,
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
                used_regs: EMPTY,
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

    fn reserve_single(&mut self) -> swriter::Registers {
        if self.ctx.used_regs & R24 == 0 {
            self.ctx.used_regs |= R24;
            return swriter::Registers::R24;
        }
        else if self.ctx.used_regs & R18 == 0 {
            self.ctx.used_regs |= R18;
            return swriter::Registers::R18;
        }
        else if self.ctx.used_regs & R16 == 0 {
            self.ctx.used_regs |= R16;
            return swriter::Registers::R16;
        }
        else {
            self.ctx.used_regs |= R18;
            return swriter::Registers::R18;
        }
    }

    fn load_constant(&mut self, val: i16) -> Result<u16, BackendError> {
        let dest = self.reserve_single();
        reports::realtime_logger::loading_const(val, dest);

        self.assm.LDI(dest, val & 0xff);
        self.assm.LDI(dest.add(1), (val >> 8) & 0xff);
        Ok(2)
    }

    fn emit_moffset(&mut self, offset:u16, size:u16, name: &String) -> Result<(), BackendError> {
        let dest = self.reserve_single();
        reports::realtime_logger::loading_var(name.clone(), offset, dest);
        for i in offset+1..offset+size+1 {
            self.assm.LDD(
                dest.add((i - offset - 1) as u8),
                swriter::Registers::Y,
                i
            )
        }

        Ok(())
    }

    fn load_variable(&mut self, name: String) -> Result<u16, BackendError> {
        for var in self.ctx.locals.clone().iter() {
            if var.name == name {
                self.emit_moffset(var.stack_offset, var.size, &var.name)?;
                return Ok(var.size);
            }
        }
        Err(BackendError::AssemblerError)
    }

    fn emit_binop(&mut self, expr: &Expr, lhs: &Expr, rhs: &Expr) -> Result<u16, BackendError> {
        let rcouple: (swriter::Registers, swriter::Registers);
        let lhs_size = self.emit_expression(lhs, false)?;
        let rhs_size = self.emit_expression(rhs, false)?;
        if self.ctx.used_regs & R24 != 0 && self.ctx.used_regs & R18 != 0 && self.ctx.used_regs & R16 == 0 {
            self.ctx.used_regs &= !R18; // Set R18 as free bc result is stored in R24
            rcouple = (swriter::Registers::R24, swriter::Registers::R18);
        }
        else if self.ctx.used_regs & R24 != 0 && self.ctx.used_regs & R18 != 0 && self.ctx.used_regs & R16 != 0 {
            self.ctx.used_regs &= !R16; // Set R16 as free bc result is stored in R18
            rcouple = (swriter::Registers::R18, swriter::Registers::R16);
        } else {
            todo!("No free registers");
        }

        match expr {
            Expr::Add(_, _) => {
                self.assm.ADD(rcouple.0, rcouple.1);
                for i in 1..lhs_size {
                    self.assm.ADC(rcouple.0.add(i as u8), rcouple.1.add(i as u8));
                }
            }
            Expr::Sub(_, _) => {}
            _ => { todo!("Unsupported node type") }
        }

        Ok(lhs_size.max(rhs_size))
    }

    fn emit_expression(&mut self, expr: &Expr, root: bool) -> Result<u16, BackendError> {
        // Root is to identify if the expression is the root of the operation tree
        if root { self.ctx.used_regs = EMPTY; }
        match expr {
            Expr::Number(value) => {
                return self.load_constant(value.clone() as i16);
            }
            Expr::Var(name) => {
                return self.load_variable(name.to_string());
            }
            Expr::Add(lhs, rhs) | Expr::Sub(lhs, rhs) => {
                self.emit_binop(expr, lhs, rhs)
            }
            _ => { todo!("Unsupported node type") },
        }
    }

    fn resolve_size(&self, ty: &String) -> Result<u16, BackendError> {
        match ty.as_str() {
            "char" => Ok(1),
            "int" => Ok(2),
            "long" => Ok(4),
            _ => Err(BackendError::AssemblerError),
        }
    }

    fn emit_declaration(&mut self, name: &String, ty: &String, value: &Expr) -> Result<(), BackendError> {
        let size = self.resolve_size(ty)?;
        self.ctx.locals.push(Variable {
            name: name.clone(),
            var_type: ty.clone(),
            size,
            stack_offset: self.ctx.stack_offset,
        });
        self.emit_expression(value, true)?;
        for i in  self.ctx.stack_offset+1 .. self.ctx.stack_offset+size+1 {
            self.assm.STD(
                swriter::Registers::Y,
                i,
                swriter::Registers::R24.add( (i - self.ctx.stack_offset - 1) as u8)
            )
        }
        self.ctx.stack_offset += size;

        Ok(())
    }

    fn emit_return(&mut self, expr: &Expr) -> Result<(), BackendError> {
        self.emit_expression(expr, true)?;
        Ok(())
    }

    fn emit_statement(&mut self, stat: &Expr) -> Result<(), BackendError> {
        //println!("{:?}", stat);
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
        println!("\n\n\n\n{}", self.assm.repr());
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
                    Expr::Decl("x".to_string(), "int".to_string(), Box::new(Expr::Number(3))),
                    Expr::Decl("y".to_string(), "int".to_string(), Box::new(Expr::Number(4))),
                    Expr::Return(
                        Box::new(Expr::Add(
                            Box::new(Expr::Add(
                                Box::new(Expr::Number(3)),
                                Box::new(Expr::Var("y".to_string())),
                            )),
                            Box::new(Expr::Add(
                                Box::new(Expr::Number(2)),
                                Box::new(Expr::Var("x".to_string())),
                            )),
                        )),
                    ),
                ])))
            ]
        };
        compile(tast);
    }
}
