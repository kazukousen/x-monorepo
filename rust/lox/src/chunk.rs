use crate::value::Value;

pub enum OpCode {
    Return,
    Constant(usize),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
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
