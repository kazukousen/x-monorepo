use crate::scanner::Scanner;
use crate::chunk::{Chunk, OpCode, Debug};
use crate::token::{Token, TokenType};
use crate::value::Value;

use std::collections::HashMap;
use std::ops::Add;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl From<i32> for Precedence {
    fn from(i: i32) -> Self {
        match i {
            0 => Precedence::None,
            1 => Precedence::Assignment,
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Term,
            7 => Precedence::Factor,
            8 => Precedence::Unary,
            9 => Precedence::Call,
            10 => Precedence::Primary,
            _ => unreachable!(),
        }
    }
}

impl Add<i32> for &Precedence {
    type Output = Precedence;

    fn add(self, other: i32) -> Precedence {
        Precedence::from(self.clone() as i32 + other)
    }
}

type ParseFn<'r> = fn(&mut Compiler<'r>) -> ();

struct ParseRule<'r> {
    prefix: Option<ParseFn<'r>>,
    infix: Option<ParseFn<'r>>,
    precedence: Precedence,
}

impl<'r> ParseRule<'r> {
    fn new(prefix: Option<ParseFn<'r>>, infix: Option<ParseFn<'r>>, precedence: Precedence) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

pub struct Compiler<'a> {
    compiling_chunk: Chunk,
    scanner: Scanner<'a>,
    parser: Parser<'a>,
    parse_rules: HashMap<TokenType, ParseRule<'a>>,
}

#[derive(Default)]
struct Parser<'a> {
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panicked: bool,
}

macro_rules! parse_rules {
    ( $( $typ:ident => $prefix:expr, $infix:expr, $precedence:ident );*; ) => {
        {
            let mut map = ::std::collections::HashMap::new();
            $(
                map.insert(
                    TokenType::$typ,
                    ParseRule::new($prefix, $infix, Precedence::$precedence),
                );
            )*
            map
        }
    }
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Self {
            compiling_chunk: Chunk::new(),
            scanner: Scanner::new(""),
            parser: Parser::default(),
            parse_rules: parse_rules![
                LeftParen => Some(Compiler::grouping), None, None;
                RightParen => None, None, None;
                Plus => None, Some(Compiler::binary), Term;
                Minus => Some(Compiler::unary), Some(Compiler::binary), Term;
                Star => None, Some(Compiler::binary), Term;
                Slash => None, Some(Compiler::binary), Term;
                Number => Some(Compiler::number), None, None;
                True => Some(Compiler::literal), None, None;
                False => Some(Compiler::literal), None, None;
                Nil => Some(Compiler::literal), None, None;
                Bang => Some(Compiler::unary), None, None;
                BangEqual => None, Some(Compiler::binary), Equality;
                Equal => None, None, None;
                EqualEqual => None, Some(Compiler::binary), Equality;
                Greater => None, Some(Compiler::binary), Comparison;
                GreaterEqual => None, Some(Compiler::binary), Comparison;
                Less => None, Some(Compiler::binary), Comparison;
                LessEqual => None, Some(Compiler::binary), Comparison;
                Eof => None, None, None;
            ],
        }
    }


    pub fn compile(&mut self, source: &'a str) -> Option<Chunk> {
        self.scanner = Scanner::new(source);

        self.parser.had_error = false;
        self.parser.panicked = false;

        self.advance();
        self.expression();
        // after we compile the expression, we should bet at the end of the source,
        // so we check for the sentinel EOF token.
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();

        if self.parser.had_error {
            None
        } else {
            let chunk = std::mem::replace(&mut self.compiling_chunk, Chunk::new());
            Some(chunk)
        }
    }

    // consume is similar to advance() in that it reads the next token.
    // but it also validates that the token has an expected type.
    // if not, it reports an error.
    fn consume(&mut self, typ: TokenType, msg: &str) {
        if self.parser.current.typ == typ {
            self.advance();
            return;
        }

        self.error_at_current(msg);
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            self.parser.current = self.scanner.scan_token();
            if self.parser.current.typ != TokenType::Error {
                break;
            }
            self.error_at_current(self.parser.current.source);
        }
    }

    fn error_at_current(&mut self, msg: &str) {
        self.error_at(self.parser.current.clone(), msg)
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.parser.previous.clone(), msg)
    }

    fn error_at(&mut self, tok: Token, msg: &str) {
        if self.parser.panicked {
            // while the panicked flag is set,
            // we simply suppress any other errors that get detected.
            return;
        }

        self.parser.panicked = true;

        eprint!("[line {}] Error", tok.line);

        match tok.typ {
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error => {} // noop
            _=> eprint!(" at '{}'", &tok.source),
        }

        eprintln!(": {}", msg);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if !self.parser.had_error {
            self.compiling_chunk.disassemble("code");
        }
    }

    fn emit_constant(&mut self, v: f64) {
        let idx = self.compiling_chunk.add_constant(Value::new_number(v));
        self.emit(OpCode::Constant(idx));
    }

    fn emit_return(&mut self) {
        self.emit(OpCode::Return)
    }

    fn emit(&mut self, op: OpCode) {
        self.compiling_chunk.add_instruction(op, self.parser.previous.line)
    }

    // number literals
    // e.g. 123
    fn number(&mut self) {
        let v: f64 = self.parser.previous.source
            .parse().expect("Compiler tried to parse to number");

        self.emit_constant(v);
    }

    // parentheses for grouping
    // e.g. (123)
    fn grouping(&mut self) {
        // we assume the initial '(' has already been consumed.
        // so we recursively call back into expression() between the parentheses.
        self.expression();
        // then parse the closing ')' at the end.
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    // unary negation
    // e.g. -123
    fn unary(&mut self) {
        // remember the operator.
        let typ = self.parser.previous.typ;

        // compile the operand.
        self.parse_precedence(Precedence::Unary);

        // emit the operator instruction.
        match typ {
            TokenType::Minus => self.emit(OpCode::Negate),
            TokenType::Bang => self.emit(OpCode::Not),
            _ => unreachable!(),
        }
    }

    // e.g. 123 + 456
    fn binary(&mut self) {
        // remember the operator.
        let typ = self.parser.previous.typ;

        // compile the right operand.
        let precedence = &self.get_rule(&typ).precedence + 1;
        self.parse_precedence(precedence);

        // emit the operator instruction.
        match typ {
            TokenType::Plus => self.emit(OpCode::Add),
            TokenType::Minus => self.emit(OpCode::Subtract),
            TokenType::Star => self.emit(OpCode::Multiply),
            TokenType::Slash => self.emit(OpCode::Divide),
            TokenType::BangEqual => {
                self.emit(OpCode::Equal);
                self.emit(OpCode::Not);
            },
            TokenType::EqualEqual => self.emit(OpCode::Equal),
            TokenType::Greater => self.emit(OpCode::Greater),
            TokenType::GreaterEqual => {
                self.emit(OpCode::Less);
                self.emit(OpCode::Not);
            },
            TokenType::Less => self.emit(OpCode::Less),
            TokenType::LessEqual => {
                self.emit(OpCode::Greater);
                self.emit(OpCode::Not);
            },
            _ => unreachable!(),
        }
    }

    // e.g. true
    fn literal(&mut self) {
        let typ = self.parser.previous.typ;
        match typ {
            TokenType::True => self.emit(OpCode::True),
            TokenType::False => self.emit(OpCode::False),
            TokenType::Nil => self.emit(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn get_rule(&self, typ: &TokenType) -> &ParseRule<'a> {
        &self.parse_rules[typ]
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(&self.parser.previous.typ).prefix;

        match prefix_rule {
            Some(rule) => {
                rule(self);

                while precedence <= self.get_rule(&self.parser.current.typ).precedence {
                    self.advance();
                    let infix_rule = &self
                        .get_rule(&self.parser.previous.typ)
                        .infix
                        .expect("Expect infix");

                    infix_rule(self);
                }
            },
            None => self.error("Expect expression"),
        }
    }
}
