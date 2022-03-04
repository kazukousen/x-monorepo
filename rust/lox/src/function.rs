use crate::chunk::Chunk;
use crate::Value;

#[derive(Eq, PartialEq)]
pub enum FunctionType {
    Function,
    Script,
}

#[derive(Copy, Clone)]
pub struct NativeFn(pub fn(&[Value]) -> Value);

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
    pub func_id: usize,
}

impl Closure {
    pub fn new(func_id: usize) -> Self {
        Self { func_id }
    }
}

#[derive(Default)]
pub struct Closures {
    closures: Vec<Closure>,
}

impl Closures {
    pub fn lookup(&self, id: usize) -> &Closure {
        &self.closures[id]
    }

    pub fn store(&mut self, closure: Closure) -> usize {
        self.closures.push(closure);
        self.closures.len() - 1
    }
}

pub struct Function {
    pub chunk: Chunk,
    pub name: Option<String>,
}

impl Function {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            name: None,
        }
    }
}

#[derive(Default)]
pub struct Functions {
    functions: Vec<Function>,
}

impl Functions {
    pub fn lookup(&self, id: usize) -> &Function {
        &self.functions[id]
    }

    pub fn store(&mut self, function: Function) -> usize {
        self.functions.push(function);
        self.functions.len() - 1
    }
}
