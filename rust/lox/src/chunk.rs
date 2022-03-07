use crate::value::Value;
use crate::Reference;

#[derive(Copy, Clone)]
pub enum OpCode {
    Return,
    Print,
    JumpIfFalse(usize),
    Jump(usize),
    Loop(usize),
    Pop,
    GetGlobal(usize),
    SetGlobal(usize),
    DefineGlobal(usize),
    GetLocal(usize),
    SetLocal(usize),
    Constant(usize),
    Call(usize),
    Closure(usize),
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

    pub fn read_string(&self, index: usize) -> &Reference<String> {
        self.values[index].as_string()
    }
}

pub trait Debug {
    fn disassemble(&self, name: &str);
}

impl Debug for Chunk {
    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        println!("==== instructions ====");
        for i in 0..self.instructions.len() {
            disassemble_instruction(self, i)
        }
        println!("==== values ====");
        for i in 0..self.values.len() {
            println!("{}: {:?}", i, &self.values[i]);
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
            OpCode::JumpIfFalse(offset) => byte_instruction("OP_JUMP_IF_FALSE", *offset),
            OpCode::Jump(offset) => byte_instruction("OP_JUMP", *offset),
            OpCode::Loop(offset) => byte_instruction("OP_LOOP", *offset),
            OpCode::Pop => simple_instruction("OP_POP"),
            OpCode::GetGlobal(index) => constant_instruction("OP_GET_GLOBAL", chunk, *index),
            OpCode::SetGlobal(index) => constant_instruction("OP_SET_GLOBAL", chunk, *index),
            OpCode::DefineGlobal(index) => constant_instruction("OP_DEFINE_GLOBAL", chunk, *index),
            OpCode::GetLocal(index) => byte_instruction("OP_GET_LOCAL", *index),
            OpCode::SetLocal(index) => byte_instruction("OP_SET_LOCAL", *index),
            OpCode::Constant(index) => constant_instruction("OP_CONSTANT", chunk, *index),
            OpCode::Call(arg_num) => byte_instruction("OP_CALL", *arg_num),
            OpCode::Closure(_) => simple_instruction("OP_CLOSURE"),
            OpCode::Negate => simple_instruction("OP_NEGATE"),
            OpCode::Add => simple_instruction("OP_ADD"),
            OpCode::Subtract => simple_instruction("OP_SUBTRACT"),
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

fn byte_instruction(name: &str, index: usize) {
    println!("{} {:04}", name, index);
}
