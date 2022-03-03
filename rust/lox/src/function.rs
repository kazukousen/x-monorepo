use crate::chunk::Chunk;

#[derive(Eq, PartialEq)]
pub enum FunctionType {
    Function,
    Script,
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
