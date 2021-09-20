extern crate lox;
use lox::*;

#[test]
fn run_arithmetic() {
    let mut compiler = Compiler::new();
    let source = r#"
print -5 + (6 - 2);
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
}

#[test]
fn run_comparison() {
    let mut compiler = Compiler::new();
    let source = r#"
print !(5 - 4 > 3 * 2 == !nil);
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
}

#[test]
fn run_string() {
    let mut compiler = Compiler::new();
    let source = r#"
print "foo" + "bar" == "foobar";
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
}

#[test]
fn run_print() {
    let mut compiler = Compiler::new();
    let source = r#"
print "foobar";
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
}
