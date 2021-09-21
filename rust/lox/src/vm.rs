use crate::chunk::{Chunk, OpCode};
use crate::value::Value;
use std::collections::HashMap;

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
    pub globals: HashMap<String, Value>,
}

macro_rules! binary_op {
    ( $vm:ident, $constructor:expr, $op:tt ) => {
        {
            if !$vm.peek(0).is_number()
                || !$vm.peek(1).is_number() {
                    eprintln!("Operand must be numbers.");
                    return InterpretResult::RuntimeError;
            }

            let (b, a) = ($vm.pop().as_number(), $vm.pop().as_number());
            $vm.push($constructor(a $op b));
        }
    };
}

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            chunk,
            pc: 0,
            stack: vec![],
            globals: Default::default(),
        }
    }

    // dispatch instructions
    pub fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = &self.chunk.instructions[self.pc];
            self.pc = self.pc + 1;

            match instruction {
                OpCode::Return => return InterpretResult::Ok,
                OpCode::Print => {
                    print!("{}\n", self.pop());
                }
                OpCode::Pop => {
                    self.pop(); // discard the result
                }
                OpCode::GetGlobal(index) => {
                    let name = self.chunk.values[*index].as_string().clone();
                    match self.globals.get(&name) {
                        Some(v) => {
                            self.push(v.clone());
                        },
                        None => {
                            eprintln!("Undefined global variable: '{}'.", name);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::DefineGlobal(index) => {
                    let name = self.chunk.values[*index].as_string().clone();
                    self.globals.insert(name, self.peek(0).clone());
                    self.pop();
                }
                OpCode::Constant(index) => {
                    let v = self.chunk.values[*index].clone();
                    self.push(v);
                }
                OpCode::Nil => self.push(Value::new_nil()),
                OpCode::True => self.push(Value::new_bool(true)),
                OpCode::False => self.push(Value::new_bool(false)),
                OpCode::Equal => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::new_bool(b == a));
                }
                OpCode::Greater => binary_op!(self, Value::new_bool, >),
                OpCode::Less => binary_op!(self, Value::new_bool, <),
                OpCode::Add => {
                    if self.peek(0).is_number() && self.peek(1).is_number() {
                        // numerical

                        let (b, a) = (self.pop().as_number(), self.pop().as_number());
                        self.push(Value::new_number(a + b));

                    } else if self.peek(0).is_string() && self.peek(1).is_string() {
                        // string

                        let (b, a) = (self.pop(), self.pop());
                        let (b, a) = (b.as_string(), a.as_string());
                        self.push(Value::new_string(format!("{}{}", a, b)));
                    } else {
                        eprintln!("Operand must be numbers or strings.");
                        return InterpretResult::RuntimeError;
                    }
                },
                OpCode::Subtract => binary_op!(self, Value::new_number, -),
                OpCode::Multiply => binary_op!(self, Value::new_number, *),
                OpCode::Divide => binary_op!(self, Value::new_number, /),
                OpCode::Negate => {
                    if !self.peek(0).is_number() {
                        eprintln!("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let v = self.pop();
                    self.push(Value::new_number(-v.as_number()));
                }
                OpCode::Not => {
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
            None => panic!("VM tried to peek value out of bounds stack"),
        }
    }
}




