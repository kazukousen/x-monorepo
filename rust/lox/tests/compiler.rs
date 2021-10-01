extern crate lox;
use lox::*;

#[test]
fn run_arithmetic() {
    let mut compiler = Compiler::new();
    let source = r#"
-5 + (6 - 2);
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
!(5 - 4 > 3 * 2 == !nil);
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
"foo" + "bar" == "foobar";
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

#[test]
fn run_global() {
    let mut compiler = Compiler::new();
    let source = r#"
var beverage = "cafe au lait";
var breakfast = "beignets with " + beverage;
print breakfast;

var mut_foo = "foo";
mut_foo = "updated foo!";
print mut_foo;
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
    assert_eq!("beignets with cafe au lait", vm.globals.get("breakfast")
        .expect("no such key").as_string().clone());
    assert_eq!("updated foo!", vm.globals.get("mut_foo")
        .expect("no such key").as_string().clone());
}

#[test]
fn run_local() {
    let mut compiler = Compiler::new();
    let source = r#"
var global = "globalized";
{
    var local = "localized";
    print local + " and " + global;
}
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
    assert_eq!("globalized", vm.globals.get("global")
        .expect("no such key").as_string().clone());
}

#[test]
fn run_if() {
    let mut compiler = Compiler::new();
    let source = r#"
var global = "globalized";
var falsy = false;
if (global == "globalized") {
    global = "localized";
} else {
    global = "foo";
}
if (global != "localized") {
    falsy = false;
} else {
    falsy = true;
}
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
    assert_eq!("localized", vm.globals.get("global")
        .expect("no such key").as_string().clone());
    assert_eq!(true, vm.globals.get("falsy")
        .expect("no such key").as_bool().clone());
}
#[test]
fn run_logical_operators() {
    let mut compiler = Compiler::new();
    let source = r#"
var global = "globalized";
var local = "localized";
var and_exp_true = ( global == "globalized" and local == "localized" );
var and_exp_false = ( global != "globalized" and local == "localized" );
var or_exp_true = ( global == "globalized" or local != "localized" );
var or_exp_false = ( global != "globalized" or local != "localized" );
"#;
    let chunk = compiler.compile(source);
    assert_eq!(true, chunk.is_some());

    let chunk = chunk.unwrap();
    let mut vm = VM::new(&chunk);
    assert_eq!(InterpretResult::Ok, vm.run());
    assert_eq!(true, vm.globals.get("and_exp_true")
        .expect("no such key").as_bool().clone());
    assert_eq!(false, vm.globals.get("and_exp_false")
        .expect("no such key").as_bool().clone());
    assert_eq!(true, vm.globals.get("or_exp_true")
        .expect("no such key").as_bool().clone());
    assert_eq!(false, vm.globals.get("or_exp_false")
        .expect("no such key").as_bool().clone());
}
