use crate::chunk::{Chunk, OpCode::*};
use crate::value::Value;
use crate::OpCode;

#[derive(Debug, Eq, PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM<'a> {
    chunk: &'a Chunk,
    pc: usize,
    pub stack: Vec<Value>,
}

macro_rules! binary_op {
    ( $vm:ident, $op:tt ) => {
        {
            if !$vm.peek(0).is_number()
                || !$vm.peek(1).is_number() {
                    eprintln!("Operand must be a number.");
                    return InterpretResult::RuntimeError;
            }

            let (b, a) = ($vm.pop().as_number(), $vm.pop().as_number());
            $vm.push(Value::new_number(a $op b));
        }
    };
}

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            chunk,
            pc: 0,
            stack: vec![],
        }
    }

    // dispatch instructions
    pub fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = &self.chunk.instructions[self.pc];
            self.pc = self.pc + 1;

            match instruction {
                Return => return InterpretResult::Ok,
                Constant(index) => {
                    let v = self.chunk.values[*index].clone();
                    self.push(v);
                }
                Nil => self.push(Value::new_nil()),
                True => self.push(Value::new_bool(true)),
                False => self.push(Value::new_bool(false)),
                Negate => {
                    if !self.peek(0).is_number() {
                        eprintln!("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let v = self.pop();
                    self.push(Value::new_number(-v.as_number()));
                }
                Add => binary_op!(self, +),
                Subtract => binary_op!(self, -),
                Multiply => binary_op!(self, *),
                Divide => binary_op!(self, /),
                Not => {
                    if !self.peek(0).is_bool()
                        && !self.peek(0).is_nil() {
                        eprintln!("Operand must be a bool or nil.");
                        return InterpretResult::RuntimeError;
                    }
                    let v = self.pop();
                    self.push(Value::new_bool(v.is_falsy()));
                }
            }
        }
    }

    fn push(&mut self, v: Value) {
        self.stack.push(v);
    }

    fn pop(&mut self) -> Value {
        match self.stack.pop() {
            Some(v) => return v,
            _ => panic!("VM tried to get value from empty stack"),
        }
    }

    fn peek(&self, distance: usize) -> &Value {
        match self.stack.get(self.stack.len() - (distance + 1)) {
            Some(v) => v,
            None => panic!("VM tried to peek value from stack"),
        }
    }
}




