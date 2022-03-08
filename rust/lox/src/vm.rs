use crate::allocator::Table;
use crate::chunk::OpCode;
use crate::function::{Closure, NativeFn};
use crate::value::Value;
use crate::{Allocator, Chunk, Parser, Reference};
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError(String),
    RuntimeError,
}

macro_rules! binary_op {
    ( $vm:ident, $constructor:expr, $op:tt ) => {
        {

            match ($vm.pop(), $vm.pop()) {
                (Value::Number(b), Value::Number(a)) => {
                    $vm.push($constructor(a $op b));
                }
                _ => {
                    eprintln!("Operand must be numbers.");
                    return InterpretResult::RuntimeError;
                }
            }
        }
    };
}

fn native_clock(_: &Allocator, _args: &[Value]) -> Value {
    Value::Number(1234_f64)
}

fn native_max(_: &Allocator, args: &[Value]) -> Value {
    if let Value::Number(a) = args[0] {
        if let Value::Number(b) = args[1] {
            return if a > b {
                args[0].clone()
            } else {
                args[1].clone()
            };
        }
    }

    panic!("panic: Operand must be numbers.");
}

fn native_panic(allocator: &Allocator, args: &[Value]) -> Value {
    let arg = args[0];
    let s = if let Value::String(s) = arg {
        allocator.deref(&s)
    } else {
        "unknown"
    };
    panic!("panic: {}", s);
}

#[derive(Copy, Clone)]
pub struct CallFrame {
    pub closure_id: Reference<Closure>,
    ip: usize,
    slot: usize,
}

impl CallFrame {
    pub fn new(closure_id: Reference<Closure>) -> Self {
        Self {
            closure_id,
            ip: 0,
            slot: 0,
        }
    }
}

pub struct VM {
    pub frames: Vec<CallFrame>,
    pub stack: Vec<Value>,
    pub globals: Table,
    pub allocator: Allocator,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            frames: vec![],
            stack: vec![],
            globals: Default::default(),
            allocator: Default::default(),
        };

        vm.define_native("clock".to_string(), NativeFn(native_clock));
        vm.define_native("max".to_string(), NativeFn(native_max));
        vm.define_native("panic".to_string(), NativeFn(native_panic));

        vm
    }

    pub fn interpret(&mut self, src: &str) -> InterpretResult {
        let mut parser = Parser::new(&mut self.allocator);

        let func_id = match parser.compile(src) {
            Ok(func_id) => func_id,
            Err(msg) => return InterpretResult::CompileError(msg),
        };

        self.push(Value::Function(func_id));
        let closure_id = self.allocator.alloc(Closure::new(func_id));
        self.frames.push(CallFrame::new(closure_id));

        let ret = self.run();

        println!("== VM ==");
        println!("== globals ==");
        for (k, v) in &self.globals {
            println!("{:?}: {:?}", k, v);
        }

        ret
    }

    // dispatch instructions
    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.current_chunk().instructions[self.current_frame().ip];
            {
                print!("id: {} ", self.current_frame().closure_id);
                for value in self.stack.iter() {
                    print!("[{}]", value);
                }
                println!();
            }
            self.current_frame_mut().ip += 1;

            match instruction {
                OpCode::Return => {
                    let value = self.pop();
                    let frame = self.frames.pop().unwrap();

                    if self.frames.is_empty() {
                        return InterpretResult::Ok;
                    }

                    self.stack.truncate(frame.slot);
                    self.push(value);
                }
                OpCode::Print => {
                    print!("{}\n", self.pop());
                }
                OpCode::JumpIfFalse(offset) => {
                    if self.peek(0).is_falsy() {
                        self.current_frame_mut().ip += offset;
                    }
                }
                OpCode::Jump(offset) => {
                    self.current_frame_mut().ip += offset;
                }
                OpCode::Loop(offset) => {
                    self.current_frame_mut().ip -= offset;
                }
                OpCode::Pop => {
                    self.pop(); // discard the result
                }
                OpCode::GetGlobal(index) => {
                    let str_id = self.current_chunk().read_string(index);
                    let v = match self.globals.get(str_id) {
                        Some(v) => v.clone(),
                        None => {
                            eprintln!("Undefined global variable: '{}'.", str_id);
                            return InterpretResult::RuntimeError;
                        }
                    };
                    self.push(v);
                }
                OpCode::SetGlobal(index) => {
                    let str_id = self.current_chunk().read_string(index);
                    match self.globals.get(str_id) {
                        Some(_) => {
                            self.globals.insert(str_id.clone(), self.peek(0).clone());
                        }
                        None => {
                            let str_id = str_id.clone();
                            self.globals.remove(&str_id);
                            eprintln!("Undefined global variable: '{}'.", str_id);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::DefineGlobal(index) => {
                    let str_id = self.current_chunk().read_string(index);
                    self.globals.insert(str_id.clone(), self.peek(0).clone());
                    self.pop();
                }
                OpCode::GetLocal(index) => {
                    let v = self.get(index + self.current_frame().slot).clone();
                    self.push(v);
                }
                OpCode::SetLocal(index) => {
                    self.stack
                        .insert(index + self.current_frame().slot, self.peek(0).clone());
                }
                OpCode::Constant(index) => {
                    let v = self.current_chunk().values[index].clone();
                    self.push(v);
                }
                OpCode::Call(arg_num) => {
                    let callee = self.peek(arg_num);

                    match callee {
                        Value::Closure(_) => {
                            self.frames.push(self.call(arg_num));
                        }
                        Value::NativeFn(_) => {
                            self.call_native_fn(arg_num);
                        }
                        _ => {
                            eprintln!("Operand must be a closure or native function.");
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Closure(index) => {
                    let func_id = match self.current_chunk().values[index].clone() {
                        Value::Function(func_id) => func_id,
                        _ => {
                            eprintln!("Value must be a function.");
                            return InterpretResult::RuntimeError;
                        }
                    };

                    let closure = Closure::new(func_id);
                    let closure_id = self.allocator.alloc(closure);
                    self.push(Value::Closure(closure_id));
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::Equal => {
                    let (b, a) = (self.pop(), self.pop());
                    self.push(Value::Bool(b == a));
                }
                OpCode::Greater => binary_op!(self, Value::Bool, >),
                OpCode::Less => binary_op!(self, Value::Bool, <),
                OpCode::Add => {
                    match (self.pop(), self.pop()) {
                        (Value::Number(b), Value::Number(a)) => {
                            // numerical
                            self.push(Value::Number(a + b));
                        }
                        (Value::String(ref b), Value::String(ref a)) => {
                            // string
                            let b = self.allocator.deref(b);
                            let a = self.allocator.deref(a);
                            let concat_str_id = self.allocator.new_string(format!("{}{}", a, b));
                            self.push(Value::String(concat_str_id));
                        }
                        _ => {
                            let frame = self.current_frame();
                            let chunk = self.current_chunk();
                            eprintln!(
                                "L:{:?}: Operand must be numbers or strings.",
                                chunk.lines[frame.ip - 1]
                            );
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::Subtract => binary_op!(self, Value::Number, -),
                OpCode::Multiply => binary_op!(self, Value::Number, *),
                OpCode::Divide => binary_op!(self, Value::Number, /),
                OpCode::Negate => match self.pop() {
                    Value::Number(v) => {
                        self.push(Value::Number(-v));
                    }
                    _ => {
                        eprintln!("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                },
                OpCode::Not => {
                    let v = self.pop();
                    match v {
                        Value::Bool(_) | Value::Nil => {
                            self.push(Value::Bool(v.is_falsy()));
                        }
                        _ => {
                            eprintln!("Operand must be a number.");
                            return InterpretResult::RuntimeError;
                        }
                    }
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
        self.get(self.stack.len() - (distance + 1))
    }

    fn get(&self, index: usize) -> &Value {
        match self.stack.get(index) {
            Some(v) => v,
            None => panic!("VM tried to peek value out of bounds stack: {}", index),
        }
    }

    fn define_native(&mut self, name: String, native: NativeFn) {
        let name = self.allocator.new_string(name);
        self.globals.insert(name, Value::NativeFn(native));
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn current_chunk(&self) -> &Chunk {
        let frame = self.current_frame();
        let closure = self.allocator.deref(&frame.closure_id);
        let function = self.allocator.deref(&closure.func_id);
        &function.chunk
    }

    fn call(&self, arg_num: usize) -> CallFrame {
        if let Value::Closure(callee_id) = self.peek(arg_num) {
            let mut new_frame = CallFrame::new(*callee_id);
            new_frame.slot = self.stack.len() - arg_num - 1;
            return new_frame;
        }
        panic!("unreachable")
    }

    fn call_native_fn(&mut self, arg_num: usize) {
        if let Value::NativeFn(f) = self.peek(arg_num) {
            let result = f.0(&self.allocator, &self.stack[self.stack.len() - arg_num..]);
            self.push(result);
            return;
        }
        panic!("unreachable")
    }
}
