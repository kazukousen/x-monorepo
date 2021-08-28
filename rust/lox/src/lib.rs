mod chunk;
mod vm;

pub use chunk::{Chunk, OpCode};
pub use vm::{InterpretResult, VM};
