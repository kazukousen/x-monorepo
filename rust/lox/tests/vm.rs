extern crate lox;
use lox::*;

#[test]
fn run_arithmetic() {
    let mut chunk = Chunk::new();
    let i = chunk.add_constant(1.2);
    chunk.add_instruction(OpCode::Constant(i), 1);
    let i = chunk.add_constant(3.4);
    chunk.add_instruction(OpCode::Constant(i), 1);
    chunk.add_instruction(OpCode::Add, 1);
    let i = chunk.add_constant(4.6);
    chunk.add_instruction(OpCode::Constant(i), 1);
    chunk.add_instruction(OpCode::Divide, 1);
    chunk.add_instruction(OpCode::Negate, 1);

    chunk.add_instruction(OpCode::Return, 2);

    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
    assert_eq!(-1_f64, vm.stack[0]); // - (1.2 + 3.4) / 4.6
}

