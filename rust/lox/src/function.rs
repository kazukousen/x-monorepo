use crate::chunk::Chunk;

pub enum FunctionType {
    Function,
    Script,
}

pub struct Function {
    params_num: u8,
    pub chunk: Chunk,
    name: Option<String>,
}

impl Function {
    pub fn new() -> Self {
        Self {
            params_num: 0,
            chunk: Chunk::new(),
            name: None,
        }
    }
}
