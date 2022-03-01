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

type ParseFn<'r> = fn(&mut Parser<'r>, can_assign: bool) -> Result<(), String>;

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
    chunk: Chunk,
    locals: Vec<Local<'a>>,
    scope_depth: usize,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }
}

pub struct Parser<'a> {
    compiler: Compiler<'a>,
    tokens: Vec<Token<'a>>,
    token_pos: usize,
    parse_rules: HashMap<TokenType, ParseRule<'a>>,
}

#[derive(Default)]
struct Local<'a> {
    name: &'a str,
    depth: usize,
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

impl<'a> Parser<'a> {
    pub fn new() -> Self {
        Self {
            compiler: Compiler::new(),
            tokens: Vec::new(),
            token_pos: 0,
            parse_rules: parse_rules![
                LeftParen => Some(Parser::grouping), None, None;
                RightParen => None, None, None;
                Plus => None, Some(Parser::binary), Term;
                Minus => Some(Parser::unary), Some(Parser::binary), Term;
                Star => None, Some(Parser::binary), Term;
                Slash => None, Some(Parser::binary), Term;
                SemiColon => None, None, None;
                Identifier => Some(Parser::variable), None, None;
                String => Some(Parser::string), None, None;
                Number => Some(Parser::number), None, None;
                And => None, Some(Parser::and), And;
                Or => None, Some(Parser::or), Or;
                True => Some(Parser::literal), None, None;
                False => Some(Parser::literal), None, None;
                Nil => Some(Parser::literal), None, None;
                Print => None, None, None;
                Bang => Some(Parser::unary), None, None;
                BangEqual => None, Some(Parser::binary), Equality;
                Equal => None, None, None;
                EqualEqual => None, Some(Parser::binary), Equality;
                Greater => None, Some(Parser::binary), Comparison;
                GreaterEqual => None, Some(Parser::binary), Comparison;
                Less => None, Some(Parser::binary), Comparison;
                LessEqual => None, Some(Parser::binary), Comparison;
                Eof => None, None, None;
            ],
        }
    }


    pub fn compile(&mut self, source: &'a str) -> Result<Chunk, String> {

        match Scanner::new(source).scan_tokens() {
            Ok(tokens) => {
                self.tokens = tokens;
            }
            Err(e) => {
                return Err(e)
            },
        }

        while !self.advance_if_matched(TokenType::Eof) {
            self.declaration()?;
        }
        self.end_compiler();

        let chunk = std::mem::replace(&mut self.compiler.chunk, Chunk::new());
        Ok(chunk)
    }

    fn advance_if_matched(&mut self, typ: TokenType) -> bool {
        if self.current().typ == typ {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        self.token_pos += 1;
    }

    fn current(&self) -> &Token<'a> {
        &self.tokens[self.token_pos]
    }

    fn previous(&self) -> &Token<'a> {
        &self.tokens[self.token_pos - 1]
    }

    /*
    program -> declaration* EOF ;
    declaration -> classDecl | funDecl | varDecl | statement ;
    statement -> exprStmt | forStmt | ifStmt | printStmt | returnStmt | whileStmt | block ;
     */

    fn declaration(&mut self) -> Result<(), String> {
        if self.advance_if_matched(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    // ```
    // "var" IDENTIFIER ("=" expression)? ";" ;
    // ```
    fn var_declaration(&mut self) -> Result<(), String> {
        self.consume(TokenType::Identifier, "Expect variable name")?;
        let name = self.previous().source;
        if self.compiler.scope_depth > 0 {
            self.compiler.locals.push(Local{ name, depth: 0 });
        }

        if self.advance_if_matched(TokenType::Equal) {
            self.expression()?;
        } else {
            self.emit(OpCode::Nil);
        }

        self.consume(TokenType::SemiColon, "Expect ';' after value declaration.")?;

        if self.compiler.scope_depth > 0 {
            self.compiler.locals.last_mut().expect("Expect locals exist one more").depth = self.compiler.scope_depth;
            return Ok(());
        }

        let global = self.identifier_constant(name);
        self.emit(OpCode::DefineGlobal(global));

        Ok(())
    }

    fn identifier_constant(&mut self, name: &'a str) -> usize {
        let name = name.to_string();
        let idx = self.compiler.chunk.add_constant(Value::new_string(name));
        return idx;
    }

    /*
    statement -> exprStmt | forStmt | ifStmt | printStmt | returnStmt | whileStmt | block ;
     */
    fn statement(&mut self) -> Result<(), String> {
        if self.advance_if_matched(TokenType::Print) {
            self.print_statement()
        } else if self.advance_if_matched(TokenType::If) {
            self.if_statement()
        } else if self.advance_if_matched(TokenType::While) {
            self.while_statement()
        } else if self.advance_if_matched(TokenType::For) {
            self.for_statement()
        } else if self.advance_if_matched(TokenType::LeftBrace) {
            self.begin_scope();
            self.block()?;
            self.end_scope();
            Ok(())
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<(), String> {
        self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after print statement.")?;
        self.emit(OpCode::Print);

        Ok(())
    }

    fn if_statement(&mut self) -> Result<(), String> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition of 'if'.")?;

        let then_pos = self.emit_jump(
            /* Set a place holder for now, and patch it later */
            OpCode::JumpIfFalse(0)
        );
        self.emit(OpCode::Pop);
        self.statement()?;
        let else_pos = self.emit_jump(
            /* Set a place holder for now, and patch it later */
            OpCode::Jump(0)
        );
        self.patch_jump(then_pos);
        self.emit(OpCode::Pop);
        if self.advance_if_matched(TokenType::Else) {
            self.statement()?;
        }
        self.patch_jump(else_pos);

        Ok(())
    }

    fn while_statement(&mut self) -> Result<(), String> {
        let start_pos = self.compiler.chunk.instructions.len() - 1;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition of 'while'.")?;

        let exit_pos = self.emit_jump(
            /* set a placeholder for now, patch it later. */
            OpCode::JumpIfFalse(0)
        );

        self.emit(OpCode::Pop);
        self.statement()?;

        // back immediately to a start position
        self.emit_loop(start_pos);

        self.patch_jump(exit_pos);
        self.emit(OpCode::Pop);

        Ok(())
    }

    /*
    forStmt -> "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
     */
    fn for_statement(&mut self) -> Result<(), String> {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for' .")?;

        // initializer clause
        if self.advance_if_matched(TokenType::SemiColon) {
            // no initializer.
        } else if self.advance_if_matched(TokenType::Var) {
            self.var_declaration()?;
        } else {
            self.expression_statement()?;
        }

        let cond_pos = self.compiler.chunk.instructions.len() - 1;

        // condition expression
        let maybe_exit_pos = match self.advance_if_matched(TokenType::SemiColon) {
            true => None, // the condition is omitted
            false => {
                self.expression()?;
                self.consume(TokenType::SemiColon, "Expect ';'.")?;
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
                let increment_pos = self.compiler.chunk.instructions.len() - 1;

                self.expression()?;
                self.emit(OpCode::Pop);
                self.consume(TokenType::RightParen, "Expect ')' after clause of 'for' .")?;

                self.emit_loop(cond_pos);
                self.patch_jump(body_pos);

                increment_pos
            }
        };

        // body statement
        self.statement()?;
        self.emit_loop(back_pos);

        maybe_exit_pos.map(|exit_pos| {
            self.patch_jump(exit_pos);
            self.emit(OpCode::Pop);
        });

        self.end_scope();

        Ok(())
    }

    // back to a start position
    fn emit_loop(&mut self, start_pos: usize) {
        let offset = self.compiler.chunk.instructions.len() - start_pos;
        self.emit(OpCode::Loop(offset));
    }

    fn expression_statement(&mut self) -> Result<(), String> {
        self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after expression statement.")?;
        self.emit(OpCode::Pop);

        Ok(())
    }

    fn block(&mut self) -> Result<(), String> {
        while self.current().typ != TokenType::RightBrace && self.current().typ != TokenType::Eof {
            self.declaration()?;
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;
        while self.compiler.locals.len() > 0 && self.compiler.locals.last().unwrap().depth > self.compiler.scope_depth {
            // discard local variables.
            self.compiler.locals.pop();
            self.emit(OpCode::Pop);
        }
    }

    // consume is similar to advance() in that it reads the next token.
    // but it also validates that the token has an expected type.
    // if not, it reports an error.
    fn consume(&mut self, typ: TokenType, msg: &str) -> Result<(), String> {
        if self.current().typ == typ {
            self.advance();
            return Ok(());
        }

        Err(format!("[Line {}] Error: {}", self.current().line, msg))
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        self.compiler.chunk.disassemble("code");
    }

    fn emit_constant(&mut self, v: Value) {
        let idx = self.compiler.chunk.add_constant(v);
        self.emit(OpCode::Constant(idx));
    }

    fn emit_return(&mut self) {
        self.emit(OpCode::Return)
    }

    fn emit(&mut self, op: OpCode) {
        self.compiler.chunk.add_instruction(op, self.previous().line)
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit(op);
        self.compiler.chunk.instructions.len() - 1
    }

    // patches the instruction at 'pos' to replace the offset value for the jump
    fn patch_jump(&mut self, pos: usize) {
        let offset = self.compiler.chunk.instructions.len() - 1 - pos;
        match self.compiler.chunk.instructions.get(pos).unwrap() {
            OpCode::JumpIfFalse(_) => {
                self.compiler.chunk.instructions[pos] = OpCode::JumpIfFalse(offset);
            }
            OpCode::Jump(_) => {
                self.compiler.chunk.instructions[pos] = OpCode::Jump(offset);
            }
            _ => unreachable!()
        }
        return
    }

    // number literals
    // e.g. 123
    fn number(&mut self, _: bool) -> Result<(), String> {
        let v: f64 = self.previous().source
            .parse().expect("Compiler tried to parse to number");

        self.emit_constant(Value::new_number(v));

        Ok(())
    }

    // parentheses for grouping
    // e.g. (123)
    fn grouping(&mut self, _: bool) -> Result<(), String> {
        // we assume the initial '(' has already been consumed.
        // so we recursively call back into expression() between the parentheses.
        self.expression()?;
        // then parse the closing ')' at the end.
        self.consume(TokenType::RightParen, "Expect ')' after expression.")
    }

    // unary negation
    // e.g. -123
    fn unary(&mut self, _: bool) -> Result<(), String> {
        // remember the operator.
        let typ = self.previous().typ;

        // compile the operand.
        self.parse_precedence(Precedence::Unary)?;

        // emit the operator instruction.
        match typ {
            TokenType::Minus => self.emit(OpCode::Negate),
            TokenType::Bang => self.emit(OpCode::Not),
            _ => unreachable!(),
        }

        Ok(())
    }

    // e.g. 123 + 456
    fn binary(&mut self, _: bool) -> Result<(), String> {
        // remember the operator.
        let typ = self.previous().typ;

        // compile the right operand.
        let precedence = &self.get_rule(&typ).precedence + 1;
        self.parse_precedence(precedence)?;

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

        Ok(())
    }

    // e.g. true
    fn literal(&mut self, _: bool) -> Result<(), String> {
        let typ = self.previous().typ;
        match typ {
            TokenType::True => self.emit(OpCode::True),
            TokenType::False => self.emit(OpCode::False),
            TokenType::Nil => self.emit(OpCode::Nil),
            _ => unreachable!(),
        }

        Ok(())
    }

    fn string(&mut self, _: bool) -> Result<(), String> {
        // trim quotes
        let s = &self.previous().source[1..=self.previous().source.len()-2];
        let s = s.to_string();
        self.emit_constant(Value::new_string(s));

        Ok(())
    }

    fn and(&mut self, _: bool) -> Result<(), String> {
        let pos = self.emit_jump(
            /* set a placeholder for now, and patch it later. */
            OpCode::JumpIfFalse(0)
        );

        self.emit(OpCode::Pop);

        // compile the right operand.
        self.parse_precedence(Precedence::And)?;

        self.patch_jump(pos);

        Ok(())
    }

    fn or(&mut self, _: bool) -> Result<(), String> {
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
        self.parse_precedence(Precedence::Or)?;
        self.patch_jump(end_pos);

        Ok(())
    }

    fn variable(&mut self, can_assign: bool) -> Result<(), String> {
        let name = self.previous().source;
        let (set_op, get_op) = match self.resolve_local(name)? {
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
            self.expression()?;
            self.emit(set_op);
        } else {
            self.emit(get_op);
        }

        Ok(())
    }

    fn resolve_local(&mut self, name: &'a str) -> Result<Option<usize>, String> {
        for (i, local) in self.compiler.locals.iter().enumerate() {
            if local.name == name {
                if local.depth == 0 {
                    return Err("Can't read local variable in its own initializer.".to_string());
                }
                return Ok(Some(self.compiler.locals.len() - 1 - i))
            }
        }
        Ok(None)
    }

    fn expression(&mut self) -> Result<(), String> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn get_rule(&self, typ: &TokenType) -> &ParseRule<'a> {
        &self.parse_rules.get(typ).expect(format!("no entry found for key: {}", typ).as_str())
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), String> {
        self.advance();
        let prefix_rule = self.get_rule(&self.previous().typ).prefix;

        match prefix_rule {
            Some(prefix_rule) => {
                let can_assign = precedence <= Precedence::Assignment;
                prefix_rule(self, can_assign)?;

                while precedence <= self.get_rule(&self.current().typ).precedence {
                    self.advance();
                    let infix_rule = &self
                        .get_rule(&self.previous().typ)
                        .infix
                        .expect("Expect infix");

                    infix_rule(self, can_assign)?;
                }

                if can_assign && self.advance_if_matched(TokenType::Equal) {
                    return Err(format!("[line {}] Error: Invalid assignment target.", self.previous().line));
                }

                Ok(())
            },
            None => Err(format!("[line {}] Error: Expect expression", self.previous().line))
        }
    }
}
