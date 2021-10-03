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

type ParseFn<'r> = fn(&mut Compiler<'r>, can_assign: bool) -> ();

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
    locals: Vec<Local<'a>>,
    scope_depth: usize,
}

#[derive(Default)]
struct Local<'a> {
    name: &'a str,
    depth: usize,
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
                SemiColon => None, None, None;
                Identifier => Some(Compiler::variable), None, None;
                String => Some(Compiler::string), None, None;
                Number => Some(Compiler::number), None, None;
                And => None, Some(Compiler::and), And;
                Or => None, Some(Compiler::or), Or;
                True => Some(Compiler::literal), None, None;
                False => Some(Compiler::literal), None, None;
                Nil => Some(Compiler::literal), None, None;
                Print => None, None, None;
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
            locals: Vec::new(),
            scope_depth: 0
        }
    }


    pub fn compile(&mut self, source: &'a str) -> Option<Chunk> {
        self.scanner = Scanner::new(source);

        self.parser.had_error = false;
        self.parser.panicked = false;

        self.advance();
        while !self.advance_if_matched(TokenType::Eof) {
            self.declaration();
        }
        self.end_compiler();

        if self.parser.had_error {
            None
        } else {
            let chunk = std::mem::replace(&mut self.compiling_chunk, Chunk::new());
            Some(chunk)
        }
    }

    fn advance_if_matched(&mut self, typ: TokenType) -> bool {
        if self.parser.current.typ == typ {
            self.advance();
            true
        } else {
            false
        }
    }

    /*
    program -> declaration* EOF ;
    declaration -> classDecl | funDecl | varDecl | statement ;
    statement -> exprStmt | forStmt | ifStmt | printStmt | returnStmt | whileStmt | block ;
     */

    fn declaration(&mut self) {
        if self.advance_if_matched(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panicked {
            self.synchronize();
        }
    }

    // ```
    // "var" IDENTIFIER ("=" expression)? ";" ;
    // ```
    fn var_declaration(&mut self) {
        self.consume(TokenType::Identifier, "Expect variable name");
        let name = self.parser.previous.source;
        if self.scope_depth > 0 {
            self.locals.push(Local{ name, depth: 0 });
        }

        if self.advance_if_matched(TokenType::Equal) {
            self.expression();
        } else {
            self.emit(OpCode::Nil);
        }

        self.consume(TokenType::SemiColon, "Expect ';' after value declaration.");

        if self.scope_depth > 0 {
            self.locals.last_mut().expect("Expect locals exist one more").depth = self.scope_depth;
            return;
        }

        let global = self.identifier_constant(name);
        self.emit(OpCode::DefineGlobal(global));
    }

    fn identifier_constant(&mut self, name: &'a str) -> usize {
        let name = name.to_string();
        let idx = self.compiling_chunk.add_constant(Value::new_string(name));
        return idx;
    }

    fn synchronize(&mut self) {
        self.parser.panicked = false;
        while self.parser.current.typ == TokenType::Eof {
            if self.parser.previous.typ == TokenType::SemiColon {
                return;
            }
            match self.parser.current.typ {
                TokenType::Class | TokenType::Fun | TokenType::Var |
                TokenType::For | TokenType::If | TokenType::While |
                TokenType::Print | TokenType::Return
                => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    /*
    statement -> exprStmt | forStmt | ifStmt | printStmt | returnStmt | whileStmt | block ;
     */
    fn statement(&mut self) {
        if self.advance_if_matched(TokenType::Print) {
            self.print_statement();
        } else if self.advance_if_matched(TokenType::If) {
            self.if_statement();
        } else if self.advance_if_matched(TokenType::While) {
            self.while_statement();
        } else if self.advance_if_matched(TokenType::For) {
            self.for_statement();
        } else if self.advance_if_matched(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after print statement.");
        self.emit(OpCode::Print);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition of 'if'.");

        let then_pos = self.emit_jump(
            /* Set a place holder for now, and patch it later */
            OpCode::JumpIfFalse(0)
        );
        self.emit(OpCode::Pop);
        self.statement();
        let else_pos = self.emit_jump(
            /* Set a place holder for now, and patch it later */
            OpCode::Jump(0)
        );
        self.patch_jump(then_pos);
        self.emit(OpCode::Pop);
        if self.advance_if_matched(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_pos);
    }

    fn while_statement(&mut self) {
        let start_pos = self.compiling_chunk.instructions.len() - 1;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition of 'while'.");

        let exit_pos = self.emit_jump(
            /* set a placeholder for now, patch it later. */
            OpCode::JumpIfFalse(0)
        );

        self.emit(OpCode::Pop);
        self.statement();

        // back immediately to a start position
        self.emit_loop(start_pos);

        self.patch_jump(exit_pos);
        self.emit(OpCode::Pop);
    }

    /*
    forStmt -> "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
     */
    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for' .");

        // initializer clause
        if self.advance_if_matched(TokenType::SemiColon) {
            // no initializer.
        } else if self.advance_if_matched(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let cond_pos = self.compiling_chunk.instructions.len() - 1;

        // condition expression
        let maybe_exit_pos = match self.advance_if_matched(TokenType::SemiColon) {
            true => None, // the condition is omitted
            false => {
                self.expression();
                self.consume(TokenType::SemiColon, "Expect ';'.");
                // if the condition is false, jump out of the loop.
                let pos = self.emit_jump(
                    /* set a placeholder for now, patch it later. */
                    OpCode::JumpIfFalse(0)
                );
                self.emit(OpCode::Pop);
                Some(pos)
            }
        };

        // increment expression
        let back_pos =  match self.advance_if_matched(TokenType::RightParen) {
            true => {
                // if the increment expression is omit
                cond_pos
            }
            false => {
                let body_pos = self.emit_jump(
                    /* set a placeholder for now, patch it later. */
                    OpCode::Jump(0)
                );
                let increment_pos = self.compiling_chunk.instructions.len() - 1;

                self.expression();
                self.emit(OpCode::Pop);
                self.consume(TokenType::RightParen, "Expect ')' after clause of 'for' .");

                self.emit_loop(cond_pos);
                self.patch_jump(body_pos);

                increment_pos
            }
        };

        // body statement
        self.statement();
        self.emit_loop(back_pos);

        maybe_exit_pos.map(|exit_pos| {
            self.patch_jump(exit_pos);
            self.emit(OpCode::Pop);
        });

        self.end_scope();
    }

    // back to a start position
    fn emit_loop(&mut self, start_pos: usize) {
        let offset = self.compiling_chunk.instructions.len() - start_pos;
        self.emit(OpCode::Loop(offset));
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SemiColon, "Expect ';' after expression statement.");
        self.emit(OpCode::Pop);
    }

    fn block(&mut self) {
        while self.parser.current.typ != TokenType::RightBrace && self.parser.current.typ != TokenType::Eof {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while self.locals.len() > 0 && self.locals.last().unwrap().depth > self.scope_depth {
            // discard local variables.
            self.locals.pop();
            self.emit(OpCode::Pop);
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
                println!("type: {}", self.parser.current.typ);
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

    fn emit_constant(&mut self, v: Value) {
        let idx = self.compiling_chunk.add_constant(v);
        self.emit(OpCode::Constant(idx));
    }

    fn emit_return(&mut self) {
        self.emit(OpCode::Return)
    }

    fn emit(&mut self, op: OpCode) {
        self.compiling_chunk.add_instruction(op, self.parser.previous.line)
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit(op);
        self.compiling_chunk.instructions.len() - 1
    }

    // patches the instruction at 'pos' to replace the offset value for the jump
    fn patch_jump(&mut self, pos: usize) {
        let offset = self.compiling_chunk.instructions.len() - 1 - pos;
        match self.compiling_chunk.instructions.get(pos).unwrap() {
            OpCode::JumpIfFalse(_) => {
                self.compiling_chunk.instructions[pos] = OpCode::JumpIfFalse(offset);
            }
            OpCode::Jump(_) => {
                self.compiling_chunk.instructions[pos] = OpCode::Jump(offset);
            }
            _ => unreachable!()
        }
        return
    }

    // number literals
    // e.g. 123
    fn number(&mut self, _: bool) {
        let v: f64 = self.parser.previous.source
            .parse().expect("Compiler tried to parse to number");

        self.emit_constant(Value::new_number(v));
    }

    // parentheses for grouping
    // e.g. (123)
    fn grouping(&mut self, _: bool) {
        // we assume the initial '(' has already been consumed.
        // so we recursively call back into expression() between the parentheses.
        self.expression();
        // then parse the closing ')' at the end.
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    // unary negation
    // e.g. -123
    fn unary(&mut self, _: bool) {
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
    fn binary(&mut self, _: bool) {
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
    fn literal(&mut self, _: bool) {
        let typ = self.parser.previous.typ;
        match typ {
            TokenType::True => self.emit(OpCode::True),
            TokenType::False => self.emit(OpCode::False),
            TokenType::Nil => self.emit(OpCode::Nil),
            _ => unreachable!(),
        }
    }

    fn string(&mut self, _: bool) {
        // trim quotes
        let s = &self.parser.previous.source[1..=self.parser.previous.source.len()-2];
        let s = s.to_string();
        self.emit_constant(Value::new_string(s));
    }

    fn and(&mut self, _: bool) {
        let pos = self.emit_jump(
            /* set a placeholder for now, and patch it later. */
            OpCode::JumpIfFalse(0)
        );

        self.emit(OpCode::Pop);

        // compile the right operand.
        self.parse_precedence(Precedence::And);

        self.patch_jump(pos);
    }

    fn or(&mut self, _: bool) {
        let else_pos = self.emit_jump(
            /* set a placeholder for now, and patch it later. */
            OpCode::JumpIfFalse(0)
        );
        let end_pos = self.emit_jump(
            /* set a placeholder for now, and patch it later. */
            OpCode::Jump(0)
        );

        self.patch_jump(else_pos);
        self.emit(OpCode::Pop);

        // compile the right operand.
        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_pos);
    }

    fn variable(&mut self, can_assign: bool) {
        let name = self.parser.previous.source;
        let (set_op, get_op) = match self.resolve_local(name) {
            Some(idx) => {
                // in current scope
                (OpCode::SetLocal(idx), OpCode::GetLocal(idx))
            }
            None => {
                // global
                let idx = self.identifier_constant(name);
                (OpCode::SetGlobal(idx), OpCode::GetGlobal(idx))
            }
        };

        if can_assign && self.advance_if_matched(TokenType::Equal) {
            self.expression();
            self.emit(set_op);
        } else {
            self.emit(get_op);
        }
    }

    fn resolve_local(&mut self, name: &'a str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate() {
            if local.name == name {
                if local.depth == 0 {
                    self.error("Can't read local variable in its own initializer.");
                }
                return Some(self.locals.len() - 1 - i)
            }
        }
        None
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn get_rule(&self, typ: &TokenType) -> &ParseRule<'a> {
        &self.parse_rules.get(typ).expect(format!("no entry found for key: {}", typ).as_str())
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(&self.parser.previous.typ).prefix;

        match prefix_rule {
            Some(prefix_rule) => {
                let can_assign = precedence <= Precedence::Assignment;
                prefix_rule(self, can_assign);

                while precedence <= self.get_rule(&self.parser.current.typ).precedence {
                    self.advance();
                    let infix_rule = &self
                        .get_rule(&self.parser.previous.typ)
                        .infix
                        .expect("Expect infix");

                    infix_rule(self, can_assign);
                }

                if can_assign && self.advance_if_matched(TokenType::Equal) {
                    self.error("Invalid assignment target.");
                }
            },
            None => self.error("Expect expression"),
        }
    }
}
