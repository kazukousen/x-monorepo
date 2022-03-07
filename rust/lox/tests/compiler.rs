extern crate lox;
use lox::*;

#[test]
fn run_arithmetic() {
    let source = r#"
-5 + (6 - 2);
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
}

#[test]
fn run_comparison() {
    let source = r#"
!(5 - 4 > 3 * 2 == !nil);
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
}

#[test]
fn run_string() {
    let source = r#"
"foo" + "bar" == "foobar";
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
}

#[test]
fn run_print() {
    let source = r#"
print "foobar";
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
}

#[test]
fn run_global() {
    let source = r#"
var beverage = "cafe au lait";
var breakfast = "beignets with " + beverage;
print breakfast;

var mut_foo = "foo";
mut_foo = "updated foo!";
print mut_foo;
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        "beignets with cafe au lait",
        vm.allocator.deref(
            vm.globals
                .get("breakfast")
                .expect("no such key")
                .as_string()
        ),
    );
    assert_eq!(
        "updated foo!",
        vm.allocator
            .deref(vm.globals.get("mut_foo").expect("no such key").as_string())
    );
}

#[test]
fn run_local() {
    let source = r#"
var global = "globalized";
var con = "";
{
    var local = "localized";
    con = local + " and " + global;
    print con;
}
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        "globalized",
        vm.allocator
            .deref(vm.globals.get("global").expect("no such key").as_string())
    );
    assert_eq!(
        "localized and globalized",
        vm.allocator
            .deref(vm.globals.get("con").expect("no such key").as_string())
    );
}

#[test]
fn run_if() {
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
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        "localized",
        vm.allocator
            .deref(vm.globals.get("global").expect("no such key").as_string()),
    );
    assert_eq!(
        true,
        vm.globals
            .get("falsy")
            .expect("no such key")
            .as_bool()
            .clone()
    );
}
#[test]
fn run_logical_operators() {
    let source = r#"
var global = "globalized";
var local = "localized";
var and_exp_true = ( global == "globalized" and local == "localized" );
var and_exp_false = ( global != "globalized" and local == "localized" );
var or_exp_true = ( global == "globalized" or local != "localized" );
var or_exp_false = ( global != "globalized" or local != "localized" );
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        true,
        vm.globals
            .get("and_exp_true")
            .expect("no such key")
            .as_bool()
            .clone()
    );
    assert_eq!(
        false,
        vm.globals
            .get("and_exp_false")
            .expect("no such key")
            .as_bool()
            .clone()
    );
    assert_eq!(
        true,
        vm.globals
            .get("or_exp_true")
            .expect("no such key")
            .as_bool()
            .clone()
    );
    assert_eq!(
        false,
        vm.globals
            .get("or_exp_false")
            .expect("no such key")
            .as_bool()
            .clone()
    );
}

#[test]
fn run_while() {
    let source = r#"
var retries = 5;
var cnt = 0;
while (cnt < retries) {
    cnt = cnt + 1;
}
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        5_f64,
        vm.globals
            .get("cnt")
            .expect("no such key")
            .as_number()
            .clone()
    );
}

#[test]
fn run_for() {
    let source = r#"
var a = 0;
for (var cnt = 0; cnt < 5; cnt = cnt + 1) {
    a = a + 1;
}

var b = 0;
var c = 0;
for (; b < 5; b = b + 1) {
    c = c + 1;
}

var d = 0;
for (;d < 5;) {
    d = d + 1;
}
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        5_f64,
        vm.globals
            .get("a")
            .expect("no such key")
            .as_number()
            .clone()
    );
    assert_eq!(
        5_f64,
        vm.globals
            .get("b")
            .expect("no such key")
            .as_number()
            .clone()
    );
    assert_eq!(
        5_f64,
        vm.globals
            .get("c")
            .expect("no such key")
            .as_number()
            .clone()
    );
    assert_eq!(
        5_f64,
        vm.globals
            .get("d")
            .expect("no such key")
            .as_number()
            .clone()
    );
}

#[test]
fn run_call_function() {
    let source = r#"
var c = 5;
fun foo() {
    var a = 1;
    var b = 2;

    c = a + b;
}
foo();
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        3_f64,
        vm.globals
            .get("c")
            .expect("no such key")
            .as_number()
            .clone()
    );
}

#[test]
fn run_call_function_with_args() {
    let source = r#"
var c = 5;
fun foo(a, b) {
    c = a + b;
}
foo(1, 2);
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        3_f64,
        vm.globals
            .get("c")
            .expect("no such key")
            .as_number()
            .clone()
    );
}

#[test]
fn run_call_function_with_args_and_returns() {
    let source = r#"
var c = 5;
fun foo(a, b) {
    return a + b;
}
c = foo(1, 2);
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        3_f64,
        vm.globals
            .get("c")
            .expect("no such key")
            .as_number()
            .clone()
    );
}

#[test]
fn run_call_function_early_return() {
    let source = r#"
var c = 5;
fun foo(a, b) {
    c = a + b;
    return;
    c = 1;
}
foo(1, 2);
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        3_f64,
        vm.globals
            .get("c")
            .expect("no such key")
            .as_number()
            .clone()
    );
}

#[test]
fn run_native_max_call() {
    let source = r#"
var a = 0;
a = max(1, 2);
"#;
    let mut vm = VM::new();
    assert_eq!(InterpretResult::Ok, vm.interpret(source));
    assert_eq!(
        2_f64,
        vm.globals
            .get("a")
            .expect("no such key")
            .as_number()
            .clone()
    );
}
