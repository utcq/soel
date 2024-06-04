use crate::arch::avr::asm_writer::*;

use ast::{Ast, Expr};

const R24: u32 = 1 << 2; // R24 - R27
const R18: u32 = 2 << 2; // R18 - R23
const R16: u32 = 4 << 2; // R16 - R18  LIMITED TO SHORT INT FOR SMALL OPS
const EMPTY: u32 = 0 << 2; // No register

pub enum BackendError {
    AssemblerError,
    RanOutOfRegisters,
    UnsupportedBinaryOperation,
    UnsupportedValue,
    CannotResolveFunction,
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
    target_register: Registers,
}

pub struct AVRBackend<'a> {
    nodes: &'a [Expr],
    assm: AVRWriter,
    ctx: Context,
}

impl<'a> AVRBackend<'a> {
    pub fn new(source: &'a [Expr]) -> Self {
        AVRBackend {
            nodes: source,
            assm: AVRWriter::new(),
            ctx: Context {
                functions: Vec::new(),
                locals: Vec::new(),
                text: 0,
                data: 0,
                stack_offset: 0,
                used_regs: EMPTY,
                target_register: Registers::R0,
            },
        }
    }

    fn emit_function(
        &mut self,
        name: &str,
        ret: &str,
        args: &[(String, String)],
        body: &Expr,
    ) -> Result<(), BackendError> {
        let addr = self.assm.create_label(name);

        self.ctx.functions.push(Function {
            name: name.into(),
            ret: ret.into(),
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
            self.assm.pop(Registers::R0);
        }

        // Fix sync problems by adding 1*locals clock cycles
        for i in 0..self.ctx.locals.len() {
            self.assm.append_after("rcall .+0".to_string(), 1 + i);
        }

        self.assm.function_epilogue();
        Ok(())
    }

    fn reserve_single(&mut self) -> Registers {
        if self.ctx.used_regs & R24 == 0 {
            self.ctx.used_regs |= R24;
            Registers::R24
        } else if self.ctx.used_regs & R18 == 0 {
            self.ctx.used_regs |= R18;
            Registers::R18
        } else if self.ctx.used_regs & R16 == 0 {
            self.ctx.used_regs |= R16;
            Registers::R16
        } else {
            self.ctx.used_regs |= R18;
            Registers::R18
        }
    }

    fn load_constant(&mut self, val: i16) -> Result<u16, BackendError> {
        let dest: Registers;
        if self.ctx.target_register == Registers::R0 {
            dest = self.reserve_single();
        } else {
            dest = self.ctx.target_register;
        }

        self.assm.ldi(dest, val & 0xff);
        self.assm.ldi(dest.add(1), (val >> 8) & 0xff);
        Ok(2)
    }

    fn emit_moffset(&mut self, offset: u16, size: u16) -> Result<(), BackendError> {
        let dest: Registers;
        if self.ctx.target_register == Registers::R0 {
            dest = self.reserve_single();
        } else {
            dest = self.ctx.target_register;
        }

        for i in offset + 1..offset + size + 1 {
            self.assm
                .ldd(dest.add((i - offset - 1) as u8), Registers::Y, i)
        }

        Ok(())
    }

    fn load_variable(&mut self, name: String) -> Result<u16, BackendError> {
        for var in self.ctx.locals.clone().iter() {
            if var.name == name {
                self.emit_moffset(var.stack_offset, var.size)?;
                return Ok(var.size);
            }
        }
        Err(BackendError::AssemblerError)
    }

    fn emit_binop(&mut self, expr: &Expr, lhs: &Expr, rhs: &Expr) -> Result<u16, BackendError> {
        let rcouple: (Registers, Registers);

        let lhs_size = self.emit_expression(lhs, false, Registers::R0)?;
        let rhs_size = self.emit_expression(rhs, false, Registers::R0)?;

        if self.ctx.used_regs & R24 != 0
            && self.ctx.used_regs & R18 != 0
            && self.ctx.used_regs & R16 == 0
        {
            self.ctx.used_regs &= !R18; // Set R18 as free bc result is stored in R24
            rcouple = (Registers::R24, Registers::R18);
        } else if self.ctx.used_regs & R24 != 0
            && self.ctx.used_regs & R18 != 0
            && self.ctx.used_regs & R16 != 0
        {
            self.ctx.used_regs &= !R16; // Set R16 as free bc result is stored in R18
            rcouple = (Registers::R18, Registers::R16);
        } else {
            return Err(BackendError::RanOutOfRegisters);
        }

        match expr {
            Expr::Add(_, _) => {
                self.assm.add(rcouple.0, rcouple.1);
                for i in 1..lhs_size {
                    self.assm
                        .adc(rcouple.0.add(i as u8), rcouple.1.add(i as u8));
                }
            }
            Expr::Sub(_, _) => {}
            _ => return Err(BackendError::UnsupportedBinaryOperation),
        }

        Ok(lhs_size.max(rhs_size))
    }

    fn resolve_function(&self, name: &str) -> Result<&Function, BackendError> {
        for func in self.ctx.functions.iter() {
            if func.name == name {
                return Ok(func);
            }
        }
        Err(BackendError::CannotResolveFunction)
    }

    fn emit_call(&mut self, name: &str, args: &Vec<Expr>) -> Result<u16, BackendError> {
        let func = self.resolve_function(&name)?;
        let ret_size = self.resolve_size(&func.ret)?;
        let mut current_reg = Registers::R16;
        let mut arg_size;

        for arg in args {
            arg_size = self.emit_expression(arg, true, current_reg)?;
            for o in 0..arg_size {
                if current_reg != Registers::R24 && self.ctx.target_register == Registers::R0 {
                    self.assm
                        .mov(current_reg.add(o as u8), Registers::R24.add((o) as u8));
                }
            }
            current_reg = current_reg.add(arg_size as u8);
        }
        Ok(ret_size)
    }

    fn emit_expression(&mut self, expr: &Expr, root: bool, target_register: Registers) -> Result<u16, BackendError> {
        // Root is to identify if the expression is the root of the operation tree
        self.ctx.target_register = target_register;
        if root {
            self.ctx.used_regs = EMPTY;
        }
        match expr {
            Expr::Number(value) => self.load_constant(value.clone() as i16),
            Expr::Var(name) => self.load_variable(name.to_string()),
            Expr::Add(lhs, rhs) | Expr::Sub(lhs, rhs) => self.emit_binop(expr, lhs, rhs),
            Expr::Call(name, args) => self.emit_call(name, args),
            _ => Err(BackendError::UnsupportedValue),
        }
    }

    fn resolve_size(&self, ty: &str) -> Result<u16, BackendError> {
        match ty {
            "char" => Ok(1),
            "int" => Ok(2),
            "long" => Ok(4),
            _ => Err(BackendError::AssemblerError),
        }
    }

    fn emit_declaration(&mut self, name: &str, ty: &str, value: &Expr) -> Result<(), BackendError> {
        let size = self.resolve_size(ty)?;

        self.ctx.locals.push(Variable {
            name: name.into(),
            size,
            stack_offset: self.ctx.stack_offset,
        });

        self.emit_expression(value, true, Registers::R24)?;

        let new_offset = self.ctx.stack_offset + 1;

        for i in new_offset..new_offset + size {
            self.assm.std(
                Registers::Y,
                i,
                Registers::R24.add((i - self.ctx.stack_offset - 1) as u8),
            )
        }

        self.ctx.stack_offset += size;

        Ok(())
    }

    fn emit_return(&mut self, expr: &Expr) -> Result<(), BackendError> {
        self.emit_expression(expr, true, Registers::R24)?;
        Ok(())
    }

    fn emit_statement(&mut self, stat: &Expr) -> Result<(), BackendError> {
        //println!("{:?}", stat);
        match stat {
            Expr::Decl(name, ty, value) => self.emit_declaration(name, ty, value),
            Expr::Return(expr) => self.emit_return(expr),
            _ => {
                self.emit_expression(stat, true, Registers::R0)?;
                Ok(())
            },
        }
    }

    pub fn process(&mut self) -> Result<(), BackendError> {
        self.assm.new_global("main");

        self.ctx.data = self.assm.create_section(".data");
        self.ctx.text = self.assm.create_section(".text");

        self.assm.select_section(self.ctx.text);

        for node in self.nodes {
            match node {
                Expr::Function(name, ret, args, body) => {
                    self.emit_function(name, ret, args, body)?;
                }
                _ => return Err(BackendError::UnsupportedValue),
            }
        }
        println!("{}", self.assm.repr());
        Ok(())
    }
}

pub fn compile(source: Ast) {
    let mut seb = AVRBackend::new(&source.root);
    let _ = seb.process();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_test0() {
        let ast = Ast {
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
                    Expr::Call("main".to_string(), vec![
                        Expr::Number(10)
                    ]),
                     /*Expr::Decl(
                        "x".to_string(),
                        "int".to_string(),
                        Box::new(Expr::Number(3)),
                    ),
                    Expr::Decl(
                        "y".to_string(),
                        "int".to_string(),
                        Box::new(Expr::Number(4)),
                    ),
                    Expr::Return(Box::new(Expr::Add(
                        Box::new(Expr::Add(
                            Box::new(Expr::Number(3)),
                            Box::new(Expr::Var("y".to_string())),
                        )),
                        Box::new(Expr::Add(
                            Box::new(Expr::Number(2)),
                            Box::new(Expr::Var("x".to_string())),
                        )),
                    ))),*/
                ])),
            )],
        };

        compile(ast);
    }
}
