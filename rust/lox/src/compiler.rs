use crate::chunk::{Debug, OpCode};
use crate::function::{Function, FunctionType, Functions};
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::value::Value;

use std::collections::HashMap;
use std::mem;
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
    fn new(
        prefix: Option<ParseFn<'r>>,
        infix: Option<ParseFn<'r>>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

pub struct Compiler<'a> {
    locals: Vec<Local<'a>>,
    scope_depth: usize,
    function: Function,
    func_type: FunctionType,
    enclosing: Option<Box<Compiler<'a>>>,
}

impl<'a> Compiler<'a> {
    pub fn new(kind: FunctionType) -> Box<Self> {
        let mut compiler = Self {
            locals: Vec::new(),
            scope_depth: 0,
            function: Function::new(),
            func_type: kind,
            enclosing: None,
        };

        compiler.locals.push(Local { name: "", depth: 0 });

        Box::new(compiler)
    }
}

pub struct Parser<'a> {
    compiler: Box<Compiler<'a>>,
    tokens: Vec<Token<'a>>,
    functions: &'a mut Functions,
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
    pub fn new(functions: &'a mut Functions) -> Self {
        Self {
            compiler: Compiler::new(FunctionType::Script),
            functions,
            tokens: Vec::new(),
            token_pos: 0,
            parse_rules: parse_rules![
                LeftParen => Some(Parser::grouping), Some(Parser::call), Call;
                RightParen => None, None, None;
                Plus => None, Some(Parser::binary), Term;
                Minus => Some(Parser::unary), Some(Parser::binary), Term;
                Star => None, Some(Parser::binary), Term;
                Slash => None, Some(Parser::binary), Term;
                SemiColon => None, None, None;
                Comma => None, None, None;
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

    pub fn compile(&mut self, source: &'a str) -> Result<usize, String> {
        match Scanner::new(source).scan_tokens() {
            Ok(tokens) => {
                self.tokens = tokens;
            }
            Err(e) => return Err(e),
        }

        while !self.advance_if_matched(TokenType::Eof) {
            self.declaration()?;
        }
        self.end_compiler();

        let function = std::mem::replace(&mut self.compiler.function, Function::new());
        let func_id = self.functions.store(function);
        Ok(func_id)
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
        if self.advance_if_matched(TokenType::Fun) {
            self.fun_declaration()
        } else if self.advance_if_matched(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn parse_identifier(&mut self) -> &'a str {
        let name = self.previous().source;
        if self.compiler.scope_depth > 0 {
            self.compiler.locals.push(Local { name, depth: 0 });
        }
        name
    }

    fn define_variable(&mut self, name: &'a str) {
        if self.compiler.scope_depth > 0 {
            self.compiler
                .locals
                .last_mut()
                .expect("Expect locals exist one more")
                .depth = self.compiler.scope_depth;
            return;
        }

        let global = self.identifier_constant(name);
        self.emit(OpCode::DefineGlobal(global));
    }

    // ```
    // "fun" IDENTIFIER "(" ")" "{" blockStmt
    // ```
    fn fun_declaration(&mut self) -> Result<(), String> {
        self.consume(TokenType::Identifier, "Expect function name")?;
        let name = self.parse_identifier();

        self.function(name, FunctionType::Function)?;

        self.define_variable(name);

        Ok(())
    }

    fn push_compiler(&mut self, name: &str, kind: FunctionType) {
        let new_compiler = Compiler::new(kind);
        let old_compiler = mem::replace(&mut self.compiler, new_compiler);
        self.compiler.enclosing = Some(old_compiler);
        self.compiler.function.name = Some(name.to_string());
    }

    fn pop_compiler(&mut self) -> Function {
        self.end_compiler();

        let function = match self.compiler.enclosing.take() {
            Some(enclosing) => {
                let compiler = mem::replace(&mut self.compiler, enclosing);
                compiler.function
            }
            None => panic!("Cannot find an enclosing compiler."),
        };

        function
    }

    fn function(&mut self, name: &str, kind: FunctionType) -> Result<(), String> {
        self.push_compiler(name, kind);
        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.")?;

        if !self.advance_if_matched(TokenType::RightParen) {
            loop {
                self.advance();
                let param = self.parse_identifier();
                self.define_variable(param);

                if !self.advance_if_matched(TokenType::Comma) {
                    break;
                }
            }
            self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        }
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.")?;
        self.block()?;

        let function = self.pop_compiler();
        let func_id = self.functions.store(function);
        let index = self.make_constant(Value::new_function(func_id));
        self.emit(OpCode::Closure(index));

        Ok(())
    }

    // ```
    // "var" IDENTIFIER ("=" expression)? ";" ;
    // ```
    fn var_declaration(&mut self) -> Result<(), String> {
        self.consume(TokenType::Identifier, "Expect variable name")?;
        let name = self.parse_identifier();

        if self.advance_if_matched(TokenType::Equal) {
            self.expression()?;
        } else {
            self.emit(OpCode::Nil);
        }

        self.consume(TokenType::SemiColon, "Expect ';' after value declaration.")?;

        self.define_variable(name);

        Ok(())
    }

    fn identifier_constant(&mut self, name: &'a str) -> usize {
        let name = name.to_string();
        let idx = self.make_constant(Value::new_string(name));
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
        } else if self.advance_if_matched(TokenType::Return) {
            self.return_statement()
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
            OpCode::JumpIfFalse(0),
        );
        self.emit(OpCode::Pop);
        self.statement()?;
        let else_pos = self.emit_jump(
            /* Set a place holder for now, and patch it later */
            OpCode::Jump(0),
        );
        self.patch_jump(then_pos);
        self.emit(OpCode::Pop);
        if self.advance_if_matched(TokenType::Else) {
            self.statement()?;
        }
        self.patch_jump(else_pos);

        Ok(())
    }

    fn return_statement(&mut self) -> Result<(), String> {
        if self.compiler.func_type == FunctionType::Script {
            return Err(format!(
                "[Line {}] Error: Cannot return from top-level code.",
                self.current().line
            ));
        }

        if self.advance_if_matched(TokenType::SemiColon) {
            self.emit_return();
        } else {
            self.expression()?;
            self.consume(TokenType::SemiColon, "Expect ';' after return value.")?;
            self.emit(OpCode::Return);
        }

        Ok(())
    }

    fn while_statement(&mut self) -> Result<(), String> {
        let start_pos = self.compiler.function.chunk.instructions.len() - 1;
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        self.expression()?;
        self.consume(
            TokenType::RightParen,
            "Expect ')' after condition of 'while'.",
        )?;

        let exit_pos = self.emit_jump(
            /* set a placeholder for now, patch it later. */
            OpCode::JumpIfFalse(0),
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

        let cond_pos = self.compiler.function.chunk.instructions.len() - 1;

        // condition expression
        let maybe_exit_pos = match self.advance_if_matched(TokenType::SemiColon) {
            true => None, // the condition is omitted
            false => {
                self.expression()?;
                self.consume(TokenType::SemiColon, "Expect ';'.")?;
                // if the condition is false, jump out of the loop.
                let pos = self.emit_jump(
                    /* set a placeholder for now, patch it later. */
                    OpCode::JumpIfFalse(0),
                );
                self.emit(OpCode::Pop);
                Some(pos)
            }
        };

        // increment expression
        let back_pos = match self.advance_if_matched(TokenType::RightParen) {
            true => {
                // if the increment expression is omit
                cond_pos
            }
            false => {
                let body_pos = self.emit_jump(
                    /* set a placeholder for now, patch it later. */
                    OpCode::Jump(0),
                );
                let increment_pos = self.compiler.function.chunk.instructions.len() - 1;

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
        let offset = self.compiler.function.chunk.instructions.len() - start_pos;
        self.emit(OpCode::Loop(offset));
    }

    fn expression_statement(&mut self) -> Result<(), String> {
        self.expression()?;
        self.consume(
            TokenType::SemiColon,
            "Expect ';' after expression statement.",
        )?;
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
        while self.compiler.locals.len() > 0
            && self.compiler.locals.last().unwrap().depth > self.compiler.scope_depth
        {
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
        let name = match &self.compiler.function.name {
            Some(name) => name.to_string(),
            None => "code".to_string(),
        };
        self.compiler.function.chunk.disassemble(&name);
    }

    fn make_constant(&mut self, v: Value) -> usize {
        self.compiler.function.chunk.add_constant(v)
    }

    fn emit_constant(&mut self, v: Value) {
        let idx = self.make_constant(v);
        self.emit(OpCode::Constant(idx));
    }

    fn emit_return(&mut self) {
        self.emit(OpCode::Nil);
        self.emit(OpCode::Return);
    }

    fn emit(&mut self, op: OpCode) {
        self.compiler
            .function
            .chunk
            .add_instruction(op, self.previous().line)
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.emit(op);
        self.compiler.function.chunk.instructions.len() - 1
    }

    // patches the instruction at 'pos' to replace the offset value for the jump
    fn patch_jump(&mut self, pos: usize) {
        let offset = self.compiler.function.chunk.instructions.len() - 1 - pos;
        match self.compiler.function.chunk.instructions.get(pos).unwrap() {
            OpCode::JumpIfFalse(_) => {
                self.compiler.function.chunk.instructions[pos] = OpCode::JumpIfFalse(offset);
            }
            OpCode::Jump(_) => {
                self.compiler.function.chunk.instructions[pos] = OpCode::Jump(offset);
            }
            _ => unreachable!(),
        }
        return;
    }

    // number literals
    // e.g. 123
    fn number(&mut self, _: bool) -> Result<(), String> {
        let v: f64 = self
            .previous()
            .source
            .parse()
            .expect("Compiler tried to parse to number");

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
            }
            TokenType::EqualEqual => self.emit(OpCode::Equal),
            TokenType::Greater => self.emit(OpCode::Greater),
            TokenType::GreaterEqual => {
                self.emit(OpCode::Less);
                self.emit(OpCode::Not);
            }
            TokenType::Less => self.emit(OpCode::Less),
            TokenType::LessEqual => {
                self.emit(OpCode::Greater);
                self.emit(OpCode::Not);
            }
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
        let s = &self.previous().source[1..=self.previous().source.len() - 2];
        let s = s.to_string();
        self.emit_constant(Value::new_string(s));

        Ok(())
    }

    fn and(&mut self, _: bool) -> Result<(), String> {
        let pos = self.emit_jump(
            /* set a placeholder for now, and patch it later. */
            OpCode::JumpIfFalse(0),
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
            OpCode::JumpIfFalse(0),
        );
        let end_pos = self.emit_jump(
            /* set a placeholder for now, and patch it later. */
            OpCode::Jump(0),
        );

        self.patch_jump(else_pos);
        self.emit(OpCode::Pop);

        // compile the right operand.
        self.parse_precedence(Precedence::Or)?;
        self.patch_jump(end_pos);

        Ok(())
    }

    fn call(&mut self, _: bool) -> Result<(), String> {
        let mut arg_count = 0;
        if !self.advance_if_matched(TokenType::RightParen) {
            loop {
                self.expression()?;
                arg_count += 1;
                if !self.advance_if_matched(TokenType::Comma) {
                    break;
                }
            }
            self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        }

        self.emit(OpCode::Call(arg_count));

        Ok(())
    }

    fn variable(&mut self, can_assign: bool) -> Result<(), String> {
        let name = self.previous().source;

        let (set_op, get_op) = if let Some(idx) = self.resolve_local(&self.compiler, name)? {
            // in current scope
            (OpCode::SetLocal(idx), OpCode::GetLocal(idx))
        // } else if let Some(idx) = self.resolve_upvalue(name) {
        // (OpCode::SetLocal(idx), OpCode::GetLocal(idx))
        } else {
            // global
            let idx = self.identifier_constant(name);
            (OpCode::SetGlobal(idx), OpCode::GetGlobal(idx))
        };

        if can_assign && self.advance_if_matched(TokenType::Equal) {
            self.expression()?;
            self.emit(set_op);
        } else {
            self.emit(get_op);
        }

        Ok(())
    }

    fn resolve_local(
        &self,
        compiler: &Box<Compiler<'a>>,
        name: &'a str,
    ) -> Result<Option<usize>, String> {
        for (i, local) in compiler.locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth == 0 {
                    return Err("Can't read local variable in its own initializer.".to_string());
                }
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    fn resolve_upvalue(&mut self, name: &'a str) -> Result<Option<usize>, String> {
        match &self.compiler.enclosing {
            Some(enclosing) => self.resolve_local(enclosing, name),
            None => Ok(None),
        }
    }

    fn expression(&mut self) -> Result<(), String> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn get_rule(&self, typ: &TokenType) -> &ParseRule<'a> {
        &self
            .parse_rules
            .get(typ)
            .expect(format!("no entry found for key: {}", typ).as_str())
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
                    return Err(format!(
                        "[line {}] Error: Invalid assignment target.",
                        self.previous().line
                    ));
                }

                Ok(())
            }
            None => Err(format!(
                "[line {}] Error: Expect expression",
                self.previous().line
            )),
        }
    }
}
