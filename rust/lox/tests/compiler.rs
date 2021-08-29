extern crate lox;
use lox::*;

#[test]
fn run_arithmetic() {
    let mut compiler = Compiler::new();
    let source = r#"
-5 + (6 - 2)
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
    assert_eq!(-1_f64, vm.stack[0].as_number());
}

#[test]
fn run_comparison() {
    let mut compiler = Compiler::new();
    let source = r#"
!(5 - 4 > 3 * 2 == !nil)
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
    assert_eq!(true, vm.stack[0].as_bool());
}
