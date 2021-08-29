extern crate lox;
use lox::*;

#[test]
fn run_arithmetic() {
    let source = "1.2 + 3.8 == 5";
    let mut sc = Scanner::new(source);

    let tok = sc.scan_token();
    assert_eq!(TokenType::Number, tok.typ);
    assert_eq!("1.2", tok.source);
    let tok = sc.scan_token();
    assert_eq!(TokenType::Plus, tok.typ);
    assert_eq!("+", tok.source);
    let tok = sc.scan_token();
    assert_eq!(TokenType::Number, tok.typ);
    assert_eq!("3.8", tok.source);
    let tok = sc.scan_token();
    assert_eq!(TokenType::EqualEqual, tok.typ);
    assert_eq!("==", tok.source);
    let tok = sc.scan_token();
    assert_eq!(TokenType::Number, tok.typ);
    assert_eq!("5", tok.source);
}

