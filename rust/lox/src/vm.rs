use crate::chunk::OpCode;
use crate::function::{Closure, Closures, Functions, NativeFn};
use crate::value::Value;
use crate::{Allocator, Parser};
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

fn native_clock(_args: &[Value]) -> Value {
    Value::new_number(1234_f64)
}

fn native_max(args: &[Value]) -> Value {
    if args[0].as_number() > args[1].as_number() {
        args[0].clone()
    } else {
        args[1].clone()
    }
}

#[derive(Copy, Clone)]
struct CallFrame {
    func_id: usize,
    closure_id: usize,
    ip: usize,
    slot: usize,
}

impl CallFrame {
    pub fn new(func_id: usize, closure_id: usize) -> Self {
        Self {
            func_id,
            closure_id,
            ip: 0,
            slot: 0,
        }
    }
}

pub struct VM {
    frames: Vec<CallFrame>,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            frames: vec![],
            stack: vec![],
            globals: Default::default(),
        };

        vm.define_native("clock".to_string(), NativeFn(native_clock));
        vm.define_native("max".to_string(), NativeFn(native_max));

        vm
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
        self.globals.insert(name, Value::new_native_fn(native));
    }
}

pub struct Store {
    pub allocator: Allocator,
    pub functions: Functions,
    pub closures: Closures,
}

impl Store {
    pub fn new() -> Self {
        Self {
            allocator: Default::default(),
            functions: Default::default(),
            closures: Default::default(),
        }
    }

    pub fn interpret(&mut self, src: &str, vm: &mut VM) -> InterpretResult {
        let mut parser = Parser::new(&mut self.functions, &mut self.allocator);

        let func_id = match parser.compile(src) {
            Ok(func_id) => func_id,
            Err(msg) => return InterpretResult::CompileError(msg),
        };

        vm.push(Value::new_function(func_id));

        let closure = Closure::new(func_id);
        let closure_id = self.closures.store(closure);

        vm.frames.push(CallFrame::new(func_id, closure_id));
        let ret = self.run(vm);
        println!("== VM ==");
        println!("== globals ==");
        for (k, v) in &vm.globals {
            println!("{:?}: {:?}", k, v);
        }
        ret
    }

    // dispatch instructions
    fn run(&mut self, vm: &mut VM) -> InterpretResult {
        let mut frame = vm.frames.pop().unwrap();
        let closure = self.closures.lookup(frame.closure_id);
        let mut chunk = &self.functions.lookup(closure.func_id).chunk;
        loop {
            let instruction = &chunk.instructions[frame.ip];
            {
                for value in vm.stack.iter() {
                    print!("[{}]", value);
                }
                println!();
            }
            frame.ip += 1;

            match instruction {
                OpCode::Return => {
                    let value = vm.pop();
                    match vm.frames.pop() {
                        Some(f) => {
                            vm.stack.truncate(frame.slot);
                            vm.push(value);
                            frame = f;
                            chunk = &self.functions.lookup(frame.func_id).chunk;
                        }
                        None => {
                            return InterpretResult::Ok;
                        }
                    }
                }
                OpCode::Print => {
                    print!("{}\n", vm.pop());
                }
                OpCode::JumpIfFalse(offset) => {
                    if vm.peek(0).is_falsy() {
                        frame.ip += *offset;
                    }
                }
                OpCode::Jump(offset) => {
                    frame.ip += *offset;
                }
                OpCode::Loop(offset) => {
                    frame.ip -= *offset;
                }
                OpCode::Pop => {
                    vm.pop(); // discard the result
                }
                OpCode::GetGlobal(index) => {
                    let str_id = chunk.values[*index].as_string();
                    let name = self.allocator.deref(str_id);
                    let v = match vm.globals.get(name) {
                        Some(v) => v.clone(),
                        None => {
                            eprintln!("Undefined global variable: '{}'.", name);
                            return InterpretResult::RuntimeError;
                        }
                    };
                    vm.push(v);
                }
                OpCode::SetGlobal(index) => {
                    let str_id = chunk.values[*index].as_string();
                    let name = self.allocator.deref(str_id).clone();
                    match vm.globals.get(&name) {
                        Some(_) => {
                            vm.globals.insert(name, vm.peek(0).clone());
                        }
                        None => {
                            vm.globals.remove(&name);
                            eprintln!("Undefined global variable: '{}'.", name);
                            return InterpretResult::RuntimeError;
                        }
                    }
                }
                OpCode::DefineGlobal(index) => {
                    let str_id = chunk.values[*index].as_string();
                    let name = self.allocator.deref(str_id).clone();
                    vm.globals.insert(name, vm.peek(0).clone());
                    vm.pop();
                }
                OpCode::GetLocal(index) => {
                    let v = vm.get(*index + frame.slot).clone();
                    vm.push(v);
                }
                OpCode::SetLocal(index) => {
                    vm.stack.insert(*index + frame.slot, vm.peek(0).clone());
                }
                OpCode::Constant(index) => {
                    let v = chunk.values[*index].clone();
                    vm.push(v);
                }
                OpCode::Call(arg_num) => {
                    let callee = vm.peek(*arg_num);

                    if callee.is_closure() {
                        vm.frames.push(frame.clone());
                        frame = self.call(vm, arg_num);
                        chunk = &self.functions.lookup(frame.func_id).chunk;
                    } else if callee.is_native_fn() {
                        self.call_native_fn(vm, arg_num);
                    } else {
                        eprintln!("Operand must be a closure or native function.");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Closure(index) => {
                    let v = chunk.values[*index].clone();
                    if !v.is_fun() {
                        eprintln!("Value must be a function.");
                        return InterpretResult::RuntimeError;
                    }

                    let closure = Closure::new(*v.as_fun());
                    let closure_id = self.closures.store(closure);
                    vm.push(Value::new_closure(closure_id));
                }
                OpCode::Nil => vm.push(Value::new_nil()),
                OpCode::True => vm.push(Value::new_bool(true)),
                OpCode::False => vm.push(Value::new_bool(false)),
                OpCode::Equal => {
                    let (b, a) = (vm.pop(), vm.pop());
                    vm.push(Value::new_bool(b == a));
                }
                OpCode::Greater => binary_op!(vm, Value::new_bool, >),
                OpCode::Less => binary_op!(vm, Value::new_bool, <),
                OpCode::Add => {
                    if vm.peek(0).is_number() && vm.peek(1).is_number() {
                        // numerical

                        let (b, a) = (vm.pop().as_number(), vm.pop().as_number());
                        vm.push(Value::new_number(a + b));
                    } else if vm.peek(0).is_string() && vm.peek(1).is_string() {
                        // string

                        let (b, a) = (vm.pop(), vm.pop());
                        let (b, a) = (b.as_string(), a.as_string());
                        let (b, a) = (self.allocator.deref(b), self.allocator.deref(a));
                        let concat_str_id = self.allocator.new_string(format!("{}{}", a, b));
                        vm.push(Value::new_string(concat_str_id));
                    } else {
                        eprintln!(
                            "L:{:?}: Operand must be numbers or strings.",
                            chunk.lines[frame.ip - 1]
                        );
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::Subtract => binary_op!(vm, Value::new_number, -),
                OpCode::Multiply => binary_op!(vm, Value::new_number, *),
                OpCode::Divide => binary_op!(vm, Value::new_number, /),
                OpCode::Negate => {
                    if !vm.peek(0).is_number() {
                        eprintln!("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }
                    let v = vm.pop();
                    vm.push(Value::new_number(-v.as_number()));
                }
                OpCode::Not => {
                    if !vm.peek(0).is_bool() && !vm.peek(0).is_nil() {
                        eprintln!("Operand must be a bool or nil.");
                        return InterpretResult::RuntimeError;
                    }
                    let v = vm.pop();
                    vm.push(Value::new_bool(v.is_falsy()));
                }
            }
        }
    }

    fn call(&self, vm: &VM, arg_num: &usize) -> CallFrame {
        let callee_id = vm.peek(*arg_num).as_closure();
        let callee = self.closures.lookup(*callee_id);
        let mut new_frame = CallFrame::new(callee.func_id, *callee_id);
        new_frame.slot = vm.stack.len() - *arg_num - 1;
        new_frame
    }

    fn call_native_fn(&self, vm: &mut VM, arg_num: &usize) {
        let f = vm.peek(*arg_num).as_native_fn();
        let result = f.0(&vm.stack[vm.stack.len() - *arg_num..]);
        vm.push(result);
    }
}
