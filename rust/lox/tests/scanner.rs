extern crate lox;
use lox::*;

#[test]
fn run_arithmetic() {
    let source = "1.2 + 3.8 == 5";
    let mut sc = Scanner::new(source);

    let tok = sc.scan_token();
    assert_eq!(TokenType::Number, tok);
}

