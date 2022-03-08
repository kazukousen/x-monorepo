use crate::chunk::Chunk;
use crate::{Allocator, Reference, Value};

#[derive(Eq, PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

#[derive(Copy, Clone)]
pub struct NativeFn(pub fn(&Allocator, &[Value]) -> Value);

impl std::fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<fn>")
    }
}

impl PartialEq for NativeFn {
    fn eq(&self, _other: &Self) -> bool {
        return false;
    }
}

pub struct Closure {
    pub func_id: Reference<Function>,
}

impl std::fmt::Debug for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<closure {}>", self.func_id)
    }
}

impl Closure {
    pub fn new(func_id: Reference<Function>) -> Self {
        Self { func_id }
    }
}

pub struct Function {
    pub chunk: Chunk,
    pub name: Reference<String>,
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

impl Function {
    pub fn new(name: Reference<String>) -> Self {
        Self {
            chunk: Chunk::new(),
            name,
        }
    }
}
