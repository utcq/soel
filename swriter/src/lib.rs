#[derive(Debug)]
pub enum Registers {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    R16,
    R17,
    R18,
    R19,
    R20,
    R21,
    R22,
    R23,
    R24,
    R25,
    R26,
    R27,
    R28,
    R29,
    R30,
    R31,
    Y,
}

struct Label {
    name: String,
    instructions: Vec<String>,
}

struct Section {
    name: String,
    data: Vec<Label>,
}

pub struct AVRWriter {
    sections: Vec<Section>,
    globals: Vec<String>,

    section: usize,
    label: usize
}

#[allow(non_snake_case, dead_code)]
impl AVRWriter {
    pub fn new() -> Self {
        AVRWriter {
            sections: Vec::new(),
            globals: Vec::new(),
            section: 0,
            label: 0,
        }
    }

    pub fn create_section(&mut self, name: &str) -> usize {
        let section = Section {
            name: name.to_string(),
            data: Vec::new(),
        };
        self.sections.push(section);
        return self.sections.len() - 1;
    }

    pub fn select_section(&mut self, section: usize) {
        self.section = section;
    }

    pub fn create_label(&mut self, name: &str) -> usize {
        let label = Label {
            name: name.to_string(),
            instructions: Vec::new(),
        };
        self.sections[self.section].data.push(label);
        return self.sections[self.section].data.len() - 1;
    }

    pub fn select_label(&mut self, label: usize) {
        self.label = label;
    }

    pub fn new_global(&mut self, name: &str) {
        self.globals.push(name.to_string());
    }

    pub fn append_instruction(&mut self, instruction: String) {
        self.sections[self.section].data[self.label].instructions.push(instruction);
    }

    pub fn insert_instruction(&mut self, instruction: String, index: usize) {
        self.sections[self.section].data[self.label].instructions.insert(index, instruction);
    }

    pub fn append_after(&mut self, instruction: String, index: usize) {
        self.insert_instruction(instruction, index + 1);
    }

    pub fn repr(&self) -> String {
        let mut repr = String::new();
        for global in &self.globals {
            repr.push_str(&format!(".global {}\n", global));
        }
        for section in &self.sections {
            repr.push_str(&format!(".section {}\n", section.name));
            for label in &section.data {
                repr.push_str(&format!("{}:\n", label.name));
                for instruction in &label.instructions {
                    repr.push_str(&format!("    {}\n", instruction));
                }
            }
        }
        return repr;
    }

    pub fn PUSH(&mut self, register: Registers) {
        self.append_instruction(format!("push {:?}", register));
    }

    pub fn IN(&mut self, register: Registers, port: i32) {
        self.append_instruction(format!("in {:?}, {}", register, port));
    }

    pub fn LDI(&mut self, register: Registers, value: i32) {
        self.append_instruction(format!("ldi {:?}, {}", register, value));
    }

    pub fn RET(&mut self) {
        self.append_instruction("ret".to_string());
    }

    pub fn POP(&mut self, register: Registers) {
        self.append_instruction(format!("pop {:?}", register));
    }

    pub fn STD(&mut self, register: Registers, offset: u16, base: Registers) {
        self.append_instruction(format!("std {:?}+{}, {:?}", register, offset, base));
    }   

    pub fn LDD(&mut self, register: Registers, base: Registers, offset: u16) {
        self.append_instruction(format!("ldd {:?}, {:?}+{}", register, base, offset));
    }

    pub fn ADD(&mut self, dest: Registers, source: Registers) {
        self.append_instruction(format!("add {:?}, {:?}", dest, source));
    }

    pub fn ADC(&mut self, dest: Registers, source: Registers) {
        self.append_instruction(format!("adc {:?}, {:?}", dest, source));
    }

    pub fn function_prologue(&mut self) {
        self.PUSH(Registers::R28);
        self.PUSH(Registers::R29);
        self.IN(Registers::R28, 0x3D);
        self.IN(Registers::R29, 0x3E);
    }

    pub fn function_epilogue(&mut self) {
        self.POP(Registers::R29);
        self.POP(Registers::R28);
        self.RET();
    }
}