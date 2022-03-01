extern crate lox;
use lox::*;

#[test]
fn run_arithmetic() {
    let mut function = Function::new();
    let i = function.chunk.add_constant(Value::new_number(1.2));
    function.chunk.add_instruction(OpCode::Constant(i), 1);
    let i = function.chunk.add_constant(Value::new_number(3.4));
    function.chunk.add_instruction(OpCode::Constant(i), 1);
    function.chunk.add_instruction(OpCode::Add, 1);
    let i = function.chunk.add_constant(Value::new_number(4.6));
    function.chunk.add_instruction(OpCode::Constant(i), 1);
    function.chunk.add_instruction(OpCode::Divide, 1);
    function.chunk.add_instruction(OpCode::Negate, 1);

    function.chunk.add_instruction(OpCode::Return, 2);

    let mut vm = VM::new(function);
    assert_eq!(InterpretResult::Ok, vm.run());
    assert_eq!(-1_f64, vm.stack[0].as_number()); // - (1.2 + 3.4) / 4.6
}
