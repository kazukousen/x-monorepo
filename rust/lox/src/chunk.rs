use crate::value::Value;

pub enum OpCode {
    Return,
    Print,
    Pop,
    GetGlobal(usize),
    SetGlobal(usize),
    DefineGlobal(usize),
    Constant(usize),
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Not,
}

pub struct Chunk {
    pub instructions: Vec<OpCode>,
    pub values: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            values: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.values.push(value);
        self.values.len() - 1
    }

    pub fn add_instruction(&mut self, op: OpCode, line: usize) {
        self.instructions.push(op);
        self.lines.push(line);
    }
}

pub trait Debug {
    fn disassemble(&self, name: &str);
}

impl Debug for Chunk {
    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        for i in 0..self.instructions.len() {
            disassemble_instruction(self, i)
        }
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) {
    print!("{:04} ", offset);
    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:04} ", chunk.lines[offset]);
    }

    if let Some(op) = chunk.instructions.get(offset) {
        match op {
            OpCode::Return => simple_instruction("OP_RETURN"),
            OpCode::Print => simple_instruction("OP_PRINT"),
            OpCode::Pop => simple_instruction("OP_POP"),
            OpCode::GetGlobal(index) => constant_instruction("OP_GET_GLOBAL", chunk, *index),
            OpCode::SetGlobal(index) => constant_instruction("OP_SET_GLOBAL", chunk, *index),
            OpCode::DefineGlobal(index) => constant_instruction("OP_DEFINE_GLOBAL", chunk, *index),
            OpCode::Constant(index) => constant_instruction( "OP_CONSTANT", chunk, *index),
            OpCode::Negate => simple_instruction("OP_NEGATE"),
            OpCode::Add => simple_instruction("OP_ADD"),
            OpCode::Subtract => simple_instruction("OP_SUBSTRACT"),
            OpCode::Multiply => simple_instruction("OP_MULTIPLY"),
            OpCode::Divide => simple_instruction("OP_DIVIDE"),
            OpCode::Nil => simple_instruction("OP_NIL"),
            OpCode::True => simple_instruction("OP_TRUE"),
            OpCode::False => simple_instruction("OP_FALSE"),
            OpCode::Equal => simple_instruction("OP_EQUAL"),
            OpCode::Greater => simple_instruction("OP_GREATER"),
            OpCode::Less => simple_instruction("OP_LESS"),
            OpCode::Not => simple_instruction("OP_NOT"),
        }
    }
}

fn simple_instruction(name: &str) {
    println!("{}", name);
}

fn constant_instruction(name: &str, chunk: &Chunk, index: usize) {
    let value = &chunk.values[index];
    println!("{} {:04} {:.2}", name, index, value);
}
